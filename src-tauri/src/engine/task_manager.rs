// 任务调度引擎
// 管理所有下载任务的生命周期：创建、排队、下载、暂停、完成、错误
// 通过全局信号量控制并发任务数，通过 channel 向 UI 推送状态更新

use crate::engine::bt::{BtConfig, BtEngine};
use crate::engine::ed2k::{Ed2kConfig, Ed2kEngine};
use crate::engine::hls::HlsEngine;
use crate::engine::http::HttpEngine;
use crate::storage::db::{Protocol, TaskParams, TaskState, TaskStatus};
use anyhow::Result;
use governor::{Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Semaphore};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

/// 任务状态更新事件（推送到前端）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskUpdateEvent {
    pub task_id: String,
    pub state: TaskState,
    pub downloaded: u64,
    pub total_size: u64,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub progress: f32,
    pub peers: u32,
    pub error: Option<String>,
    pub eta: Option<u64>,
}

/// 数据库持久化事件（用于保存进度到 DB）
#[derive(Debug, Clone)]
pub struct DbPersistEvent {
    pub task_id: String,
    pub state: TaskState,
    pub downloaded: u64,
    pub total_size: u64,
    pub error: Option<String>,
}

/// 任务句柄（用于控制正在运行的任务）
struct TaskHandle {
    /// 取消令牌（取消任务时触发）
    cancel_token: CancellationToken,
    /// 暂停信号发送端（true=暂停, false=恢复）
    pause_tx: watch::Sender<bool>,
    /// 任务参数（用于重试）
    url: String,
    save_path: String,
    file_name: String,
    proxy: Option<String>,
    speed_limit: Option<u64>,
    /// 协议类型
    protocol: Protocol,
    /// 是否处于暂停状态
    paused: bool,
}

/// 任务调度管理器
pub struct TaskManager {
    /// 全局并发信号量（控制同时下载的任务数）
    semaphore: Arc<Semaphore>,
    /// 运行中的任务句柄映射
    handles: HashMap<String, TaskHandle>,
    /// HTTP 下载引擎
    http_engine: HttpEngine,
    /// BT 下载引擎
    bt_engine: BtEngine,
    /// HLS/DASH 流媒体引擎
    hls_engine: HlsEngine,
    /// ed2k 下载引擎
    ed2k_engine: Ed2kEngine,
    /// 全局下载限速器（跨任务共享）
    global_download_limiter: Option<Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>>,
    /// 全局上传限速器（跨任务共享）
    global_upload_limiter: Option<Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>>,
    /// 状态更新发送端（前端监听）
    update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
    /// 状态更新接收端（前端取走后为 None）
    update_rx: Option<mpsc::UnboundedReceiver<TaskUpdateEvent>>,
    /// 数据库持久化发送端
    db_persist_tx: mpsc::UnboundedSender<DbPersistEvent>,
    /// 数据库持久化接收端
    db_persist_rx: Option<mpsc::UnboundedReceiver<DbPersistEvent>>,
    /// 任务完成清理发送端（通知 TaskManager 移除已完成的句柄）
    cleanup_tx: mpsc::UnboundedSender<String>,
    /// 任务完成清理接收端
    cleanup_rx: Option<mpsc::UnboundedReceiver<String>>,
    /// 配置的并发数上限（用于查询）
    max_concurrent_configured: usize,
    /// 自动重试次数
    auto_retry_count: u32,
    /// 自动重试间隔（秒）
    auto_retry_interval: u64,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new(
        max_concurrent: usize,
        max_upload_speed: u64,
        max_download_speed: u64,
        bt_config: BtConfig,
    ) -> Self {
        let (update_tx, update_rx) = mpsc::unbounded_channel();
        let (db_persist_tx, db_persist_rx) = mpsc::unbounded_channel();
        let (cleanup_tx, cleanup_rx) = mpsc::unbounded_channel();

        // 创建全局下载限速器（0 表示不限速，u64 饱和到 u32 最大值 ~4GB/s）
        let global_download_limiter = if max_download_speed > 0 {
            NonZeroU32::new(max_download_speed.min(u32::MAX as u64) as u32).map(|n| {
                Arc::new(RateLimiter::direct(Quota::per_second(n)))
            })
        } else {
            None
        };

        // 创建全局上传限速器（0 表示不限速）
        let global_upload_limiter = if max_upload_speed > 0 {
            NonZeroU32::new(max_upload_speed.min(u32::MAX as u64) as u32).map(|n| {
                Arc::new(RateLimiter::direct(Quota::per_second(n)))
            })
        } else {
            None
        };

        TaskManager {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            handles: HashMap::new(),
            http_engine: HttpEngine::new(),
            bt_engine: BtEngine::new(bt_config),
            hls_engine: HlsEngine::new(),
            ed2k_engine: Ed2kEngine::new(Ed2kConfig::default()),
            global_download_limiter,
            global_upload_limiter,
            update_tx,
            update_rx: Some(update_rx),
            db_persist_tx,
            db_persist_rx: Some(db_persist_rx),
            cleanup_tx,
            cleanup_rx: Some(cleanup_rx),
            max_concurrent_configured: max_concurrent,
            auto_retry_count: 3,
            auto_retry_interval: 5,
        }
    }

    /// 初始化 BT 引擎（在应用启动时调用）
    pub async fn init_bt_engine(&mut self, default_download_dir: &str) -> Result<()> {
        self.bt_engine.init(default_download_dir).await
    }

    /// 获取状态更新接收端（前端调用，只能取一次）
    pub fn take_update_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<TaskUpdateEvent>> {
        self.update_rx.take()
    }

    /// 获取数据库持久化接收端
    pub fn take_db_persist_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<DbPersistEvent>> {
        self.db_persist_rx.take()
    }

    /// 获取任务完成清理接收端
    pub fn take_cleanup_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<String>> {
        self.cleanup_rx.take()
    }

    /// 处理已完成任务的句柄清理（移除已完成/出错的任务句柄释放内存）
    pub fn process_cleanup(&mut self, task_ids: &[String]) {
        for task_id in task_ids {
            if self.handles.remove(task_id).is_some() {
                info!("已清理完成任务的句柄: {}", task_id);
            }
        }
    }

    /// 设置自动重试参数
    pub fn set_retry_config(&mut self, count: u32, interval: u64) {
        self.auto_retry_count = count;
        self.auto_retry_interval = interval;
    }

    /// 设置全局下载速度限制（bytes/sec，0=不限速，u64 饱和到 u32）
    pub fn set_global_download_speed_limit(&mut self, bytes_per_sec: u64) {
        self.global_download_limiter = if bytes_per_sec > 0 {
            NonZeroU32::new(bytes_per_sec.min(u32::MAX as u64) as u32).map(|n| {
                Arc::new(RateLimiter::direct(Quota::per_second(n)))
            })
        } else {
            None
        };
    }

    /// 设置全局上传速度限制（bytes/sec，0=不限速，u64 饱和到 u32）
    pub fn set_global_upload_speed_limit(&mut self, bytes_per_sec: u64) {
        self.global_upload_limiter = if bytes_per_sec > 0 {
            NonZeroU32::new(bytes_per_sec.min(u32::MAX as u64) as u32).map(|n| {
                Arc::new(RateLimiter::direct(Quota::per_second(n)))
            })
        } else {
            None
        };
    }

    /// 添加新任务并启动下载
    pub async fn add_task(&mut self, task: TaskStatus, params: &TaskParams) -> Result<()> {
        let task_id = task.id.clone();
        info!("添加任务: {} ({})", task.name, task_id);

        // 检查重复的 magnet/torrent 任务（同一链接/文件只能有一个活跃任务）
        // 跳过已完成/已取消的任务（cancel_token 已取消 = 后台任务已结束）
        if matches!(task.protocol, Protocol::Magnet | Protocol::Bt) {
            for (_existing_id, handle) in &self.handles {
                if handle.protocol == task.protocol
                    && handle.url == task.url
                    && !handle.cancel_token.is_cancelled()
                {
                    anyhow::bail!(
                        "该任务已存在: {}，请勿重复添加",
                        handle.file_name
                    );
                }
            }
        }

        // 创建取消令牌和暂停信号
        let cancel_token = CancellationToken::new();
        let (pause_tx, pause_rx) = watch::channel(false);

        // 保存任务句柄（包含参数用于重试）
        self.handles.insert(
            task_id.clone(),
            TaskHandle {
                cancel_token: cancel_token.clone(),
                pause_tx,
                url: task.url.clone(),
                save_path: task.save_path.clone(),
                file_name: task.name.clone(),
                proxy: params.proxy.clone(),
                speed_limit: params.speed_limit,
                protocol: task.protocol.clone(),
                paused: false,
            },
        );

        // BT/Magnet 任务直接由 BtEngine 管理，不需要信号量
        if matches!(task.protocol, Protocol::Bt | Protocol::Magnet) {
            let update_tx = self.update_tx.clone();
            let result = match task.protocol {
                Protocol::Bt => {
                    self.bt_engine
                        .add_torrent_file(&task_id, &task.url, &task.save_path, update_tx)
                        .await
                }
                Protocol::Magnet => {
                    self.bt_engine
                        .add_magnet(&task_id, &task.url, &task.save_path, update_tx)
                        .await
                }
                _ => unreachable!(),
            };

            if let Err(e) = result {
                error!("BT 任务添加失败: {} - {}", task_id, e);
                self.handles.remove(&task_id);
                return Err(e);
            }
            return Ok(());
        }

        // HTTP/FTP/ed2k/HLS/DASH 任务使用信号量控制并发
        let update_tx = self.update_tx.clone();
        let db_persist_tx = self.db_persist_tx.clone();
        let cleanup_tx = self.cleanup_tx.clone();
        let semaphore = self.semaphore.clone();
        let engine = self.http_engine.clone();
        let mut engine_ed2k = self.ed2k_engine.clone();
        let hls_engine = self.hls_engine.clone();
        let global_download_limiter = self.global_download_limiter.clone();
        let global_upload_limiter = self.global_upload_limiter.clone();
        let proxy = params.proxy.clone();
        let speed_limit = params.speed_limit;
        let url = task.url.clone();
        let save_path = task.save_path.clone();
        let file_name = task.name.clone();
        let mut downloaded = task.downloaded;
        let total_size = task.total_size;
        let protocol = task.protocol.clone();
        let retry_count = self.auto_retry_count;
        let mirror_urls = params.mirror_urls.clone().unwrap_or_default();
        let retry_interval = self.auto_retry_interval;
        // 构建下载选项（HTTP 认证、Cookie、自定义 Headers）
        let download_opts = crate::engine::http::DownloadOptions {
            auth: params.http_auth.clone(),
            cookie: params.http_cookie.clone(),
            headers: params.http_headers.clone(),
        };

        // 在后台任务中执行下载
        tokio::spawn(async move {
            // 等待信号量（控制并发数）
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    error!("信号量已关闭，任务取消: {}", task_id);
                    let _ = update_tx.send(TaskUpdateEvent {
                        task_id: task_id.clone(),
                        state: TaskState::Error,
                        downloaded,
                        total_size,
                        download_speed: 0,
                        upload_speed: 0,
                        progress: 0.0,
                        peers: 0,
                        error: Some("并发控制信号量已关闭".to_string()),
                        eta: None,
                    });
                    return;
                }
            };

            info!("任务开始下载: {}", task_id);

            // 发送状态更新：开始下载
            let _ = update_tx.send(TaskUpdateEvent {
                task_id: task_id.clone(),
                state: TaskState::Downloading,
                downloaded,
                total_size,
                download_speed: 0,
                upload_speed: 0,
                progress: if total_size > 0 {
                    (downloaded as f32 / total_size as f32).min(1.0)
                } else {
                    0.0
                },
                peers: 0,
                error: None,
                eta: None,
            });

            // 根据协议选择引擎执行下载（带自动重试）
            let mut last_error = String::new();
            let mut attempt = 0u32;
            let max_attempts = retry_count + 1; // 首次 + 重试次数

            loop {
                attempt += 1;

                let result = match protocol {
                    Protocol::Http | Protocol::Ftp => {
                        // 如果有镜像URL，使用多源下载
                        if !mirror_urls.is_empty() {
                            let mut all_urls = vec![url.clone()];
                            all_urls.extend(mirror_urls.clone());
                            engine
                                .download_with_mirrors(
                                    &task_id,
                                    &all_urls,
                                    &save_path,
                                    &file_name,
                                    downloaded,
                                    proxy.clone(),
                                    speed_limit,
                                    global_download_limiter.clone(),
                                    global_upload_limiter.clone(),
                                    update_tx.clone(),
                                    cancel_token.clone(),
                                    pause_rx.clone(),
                                    download_opts.clone(),
                                )
                                .await
                        } else {
                            engine
                                .download(
                                    &task_id,
                                    &url,
                                    &save_path,
                                    &file_name,
                                    downloaded,
                                    proxy.clone(),
                                    speed_limit,
                                    global_download_limiter.clone(),
                                    global_upload_limiter.clone(),
                                    update_tx.clone(),
                                    cancel_token.clone(),
                                    pause_rx.clone(),
                                    download_opts.clone(),
                                )
                                .await
                        }
                    }
                    Protocol::Hls | Protocol::Dash => {
                        hls_engine
                            .download(
                                &task_id,
                                &url,
                                &save_path,
                                &file_name,
                                None, // quality_label: 默认选择最高分辨率
                                update_tx.clone(),
                                cancel_token.clone(),
                                pause_rx.clone(),
                            )
                            .await
                    }
                    Protocol::Ed2k => {
                        let link = match crate::engine::ed2k::parse_ed2k_link(&url) {
                            Ok(l) => l,
                            Err(e) => {
                                tracing::error!("ed2k 链接解析失败: {} - {}", task_id, e);
                                let _ = update_tx.send(TaskUpdateEvent {
                                    task_id: task_id.clone(),
                                    state: TaskState::Error,
                                    downloaded: 0,
                                    total_size: 0,
                                    download_speed: 0,
                                    upload_speed: 0,
                                    progress: 0.0,
                                    peers: 0,
                                    error: Some(format!("ed2k 链接解析失败: {}", e)),
                                    eta: None,
                                });
                                return;
                            }
                        };
                        engine_ed2k
                            .download(
                                &task_id,
                                &link,
                                &save_path,
                                update_tx.clone(),
                                cancel_token.clone(),
                                pause_rx.clone(),
                            )
                            .await
                    }
                    Protocol::Bt | Protocol::Magnet => {
                        // BT 任务已在上层处理
                        Ok(())
                    }
                };

                // 检查取消信号
                if cancel_token.is_cancelled() {
                    info!("任务已取消: {}", task_id);
                    return;
                }

                match result {
                    Ok(_) => {
                        info!("任务下载完成: {}", task_id);
                        let _ = update_tx.send(TaskUpdateEvent {
                            task_id: task_id.clone(),
                            state: TaskState::Done,
                            downloaded: total_size,
                            total_size,
                            download_speed: 0,
                            upload_speed: 0,
                            progress: 1.0,
                            peers: 0,
                            error: None,
                            eta: Some(0),
                        });
                        // 持久化到 DB
                        if let Err(e) = db_persist_tx.send(DbPersistEvent {
                            task_id: task_id.clone(),
                            state: TaskState::Done,
                            downloaded: total_size,
                            total_size,
                            error: None,
                        }) {
                            tracing::warn!("DB 持久化通道发送失败（任务完成进度可能丢失）: {} - {}", task_id, e);
                        }
                        // 通知 TaskManager 清理已完成任务的句柄，释放内存
                        if let Err(e) = cleanup_tx.send(task_id.clone()) {
                            tracing::warn!("清理通道发送失败（任务句柄可能泄漏）: {} - {}", task_id, e);
                        }
                        break;
                    }
                    Err(e) => {
                        last_error = e.to_string();
                        error!(
                            "任务下载失败 (尝试 {}/{}): {} - {}",
                            attempt, max_attempts, task_id, last_error
                        );

                        if attempt < max_attempts {
                            // 还可以重试 — 从磁盘检测已下载大小以正确续传
                            let file_path = std::path::Path::new(&save_path).join(&file_name);
                            if let Ok(meta) = tokio::fs::metadata(&file_path).await {
                                downloaded = meta.len();
                            }

                            info!("{} 秒后重试任务: {} (已下载: {} bytes)", retry_interval, task_id, downloaded);
                            let _ = update_tx.send(TaskUpdateEvent {
                                task_id: task_id.clone(),
                                state: TaskState::Downloading,
                                downloaded,
                                total_size,
                                download_speed: 0,
                                upload_speed: 0,
                                progress: if total_size > 0 {
                                    (downloaded as f32 / total_size as f32).min(1.0)
                                } else {
                                    0.0
                                },
                                peers: 0,
                                error: Some(format!(
                                    "下载失败，{}/{} 次重试中: {}",
                                    attempt, max_attempts, last_error
                                )),
                                eta: None,
                            });
                            tokio::time::sleep(std::time::Duration::from_secs(retry_interval))
                                .await;

                            // 再次检查取消
                            if cancel_token.is_cancelled() {
                                return;
                            }
                        } else {
                            // 重试用尽，标记错误（保留实际进度）
                            error!("任务最终失败: {} - {}", task_id, last_error);
                            let _ = update_tx.send(TaskUpdateEvent {
                                task_id: task_id.clone(),
                                state: TaskState::Error,
                                downloaded,
                                total_size,
                                download_speed: 0,
                                upload_speed: 0,
                                progress: if total_size > 0 { (downloaded as f32 / total_size as f32).min(1.0) } else { 0.0 },
                                peers: 0,
                                error: Some(last_error.clone()),
                                eta: None,
                            });
                            // 持久化到 DB
                            if let Err(e) = db_persist_tx.send(DbPersistEvent {
                                task_id: task_id.clone(),
                                state: TaskState::Error,
                                downloaded,
                                total_size,
                                error: Some(last_error.clone()),
                            }) {
                                tracing::warn!("DB 持久化通道发送失败（任务错误状态可能丢失）: {} - {}", task_id, e);
                            }
                            // 通知 TaskManager 清理失败任务的句柄，释放内存
                            if let Err(e) = cleanup_tx.send(task_id.clone()) {
                                tracing::warn!("清理通道发送失败（任务句柄可能泄漏）: {} - {}", task_id, e);
                            }
                            break;
                        }
                    }
                }
            }

            // 信号量自动释放
        });

        Ok(())
    }

    /// 暂停任务
    pub async fn pause_task(&mut self, task_id: &str) -> Result<()> {
        let handle = self.handles.get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("任务不存在: {}", task_id))?;
        handle.paused = true;
        // BT 任务暂停
        if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
            self.bt_engine.pause_task(task_id).await?;
        } else {
            let _ = handle.pause_tx.send(true);
        }
        // 只发送状态变更，不覆盖进度数据
        let _ = self.update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Paused,
            downloaded: 0, // 0 = 不更新，前端保留现有值
            total_size: 0,
            download_speed: 0,
            upload_speed: 0,
            progress: -1.0, // 负值 = 不更新进度
            peers: 0,
            error: None,
            eta: None,
        });
        info!("任务已暂停: {}", task_id);
        Ok(())
    }

    /// 恢复任务
    /// 对于已退出的非BT任务，返回特殊错误让调用方重新添加
    pub async fn resume_task(&mut self, task_id: &str) -> Result<()> {
        let handle = self.handles.get(task_id)
            .ok_or_else(|| anyhow::anyhow!("任务不存在: {}", task_id))?;

        // 检查是否为已退出的任务（cancel_token 已取消 = 后台任务已结束）
        let needs_re_add = handle.cancel_token.is_cancelled() && !handle.paused;
        let is_bt = matches!(handle.protocol, Protocol::Bt | Protocol::Magnet);
        drop(handle);

        if needs_re_add && !is_bt {
            // 非BT任务已退出，需要通过 add_task 重新启动
            self.handles.remove(task_id);
            anyhow::bail!("任务需要重新启动");
        }

        let handle = self.handles.get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("任务不存在: {}", task_id))?;
        handle.paused = false;

        // BT 任务恢复
        if is_bt {
            self.bt_engine.resume_task(task_id).await?;
        } else {
            let _ = handle.pause_tx.send(false);
        }
        let _ = self.update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Downloading,
            downloaded: 0,
            total_size: 0,
            download_speed: 0,
            upload_speed: 0,
            progress: -1.0, // 负值 = 不更新进度
            peers: 0,
            error: None,
            eta: None,
        });
        info!("任务已恢复: {}", task_id);
        Ok(())
    }

    /// 暂停所有任务（包括 BT 任务）
    pub async fn pause_all(&mut self) {
        let task_ids: Vec<String> = self.handles.keys().cloned().collect();
        for task_id in task_ids {
            if let Some(handle) = self.handles.get_mut(&task_id) {
                handle.paused = true;
                // BT 任务需要通过引擎暂停
                if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
                    if let Err(e) = self.bt_engine.pause_task(&task_id).await {
                        warn!("暂停 BT 任务 {} 失败: {}", task_id, e);
                    }
                } else {
                    let _ = handle.pause_tx.send(true);
                }
                let _ = self.update_tx.send(TaskUpdateEvent {
                    task_id: task_id.clone(),
                    state: TaskState::Paused,
                    downloaded: 0,
                    total_size: 0,
                    download_speed: 0,
                    upload_speed: 0,
                    progress: -1.0,
                    peers: 0,
                    error: None,
                    eta: None,
                });
            }
        }
        info!("已暂停所有任务");
    }

    /// 恢复所有暂停的任务
    pub async fn resume_all_paused(&mut self) -> Result<()> {
        let task_ids: Vec<String> = self.handles.keys().cloned().collect();
        for task_id in task_ids {
            if let Some(handle) = self.handles.get(&task_id) {
                if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
                    if let Err(e) = self.bt_engine.resume_task(&task_id).await {
                        tracing::warn!("BT 恢复任务失败: {} - {}", task_id, e);
                    }
                } else {
                    let _ = handle.pause_tx.send(false);
                }
                let _ = self.update_tx.send(TaskUpdateEvent {
                    task_id: task_id.clone(),
                    state: TaskState::Downloading,
                    downloaded: 0,
                    total_size: 0,
                    download_speed: 0,
                    upload_speed: 0,
                    progress: -1.0,
                    peers: 0,
                    error: None,
                    eta: None,
                });
            }
        }
        info!("已恢复所有暂停任务");
        Ok(())
    }

    /// 取消/删除任务
    pub async fn remove_task(&mut self, task_id: &str) -> Result<()> {
        if let Some(handle) = self.handles.remove(task_id) {
            if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
                if let Err(e) = self.bt_engine.remove_task(task_id, false).await {
                    tracing::warn!("BT 删除任务失败: {} - {}", task_id, e);
                }
            } else {
                handle.cancel_token.cancel();
            }
            info!("任务已删除: {}", task_id);
        }
        Ok(())
    }

    /// 检查任务是否在管理器中
    pub fn has_task(&self, task_id: &str) -> bool {
        self.handles.contains_key(task_id)
    }

    /// 检查是否有活跃的下载任务（用于判断是否触发下载后动作）
    /// 排除暂停的任务——暂停的任务不应阻止关机/休眠等后置动作
    pub fn has_active_tasks(&self) -> bool {
        self.handles.values().any(|h| !h.paused)
    }

    /// 更新并发数限制（动态调整信号量许可数）
    pub fn set_max_concurrent(&mut self, max: usize) {
        let max = max.max(1); // 最小为 1，防止 max=0 导致死锁
        self.max_concurrent_configured = max;
        let current = self.semaphore.available_permits();
        if max > current {
            // 增加许可：添加差额
            self.semaphore.add_permits(max - current);
        } else if max < current {
            // 减少许可：遗忘多余许可（不阻塞，新任务会等待）
            let excess = current - max;
            self.semaphore.forget_permits(excess);
        }
        // max == current 时无需操作
    }

    /// 获取当前并发数限制
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent_configured
    }

    /// 获取 BT 引擎状态（DHT 节点数等）
    pub async fn get_bt_status(&self) -> crate::engine::bt::BtStatus {
        self.bt_engine.get_status().await
    }

    /// 更新 ed2k 自定义服务器列表
    pub fn set_ed2k_servers(&mut self, servers: Vec<String>) {
        if !servers.is_empty() {
            self.ed2k_engine.set_custom_servers(servers);
        }
    }

    /// 启动 KAD 网络（使用 ed2k 服务器列表作为引导节点）
    pub async fn start_kad(&mut self) {
        if let Err(e) = self.ed2k_engine.start_kad().await {
            tracing::warn!("KAD 启动失败: {}", e);
        }
    }

    /// KAD 定期维护（清理过期节点、刷新路由表）
    pub async fn maintain_kad(&mut self) {
        self.ed2k_engine.maintain_kad().await;
    }

    /// 获取 KAD 状态
    pub fn get_kad_status(&self) -> Option<crate::engine::ed2k::kad::KadStatus> {
        self.ed2k_engine.kad_status()
    }

    /// 更新 BT 做种配置
    pub fn set_bt_seeding_config(&mut self, seed_ratio: f64, seed_time: u64, stop_seeding: bool) {
        self.bt_engine.set_seeding_config(seed_ratio, seed_time, stop_seeding);
    }

    /// 设置 BT 引擎速度限制（bytes/sec，0=不限速）
    pub fn set_bt_speed_limits(&mut self, max_download_speed: u64, max_upload_speed: u64) {
        self.bt_engine.set_speed_limits(max_download_speed, max_upload_speed);
    }

    /// 获取任务的 Peer 连接列表
    pub async fn get_task_peers(&self, task_id: &str) -> Vec<crate::commands::task::PeerInfo> {
        if let Some(handle) = self.handles.get(task_id) {
            if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
                // BT 任务从引擎获取 peer 信息
                let stats = self.bt_engine.get_task_stats(task_id).await;
                if let Some(stats) = stats {
                    return stats;
                }
            }
        }
        // 回退：构造基于当前状态的 peer 信息
        vec![]
    }

    /// 获取任务的 Tracker 列表
    pub async fn get_task_trackers(&self, task_id: &str) -> Vec<crate::commands::task::TrackerInfo> {
        if let Some(handle) = self.handles.get(task_id) {
            if matches!(handle.protocol, Protocol::Bt | Protocol::Magnet) {
                return self.bt_engine.get_task_trackers(task_id).await;
            }
        }
        vec![]
    }

    /// 获取任务日志
    pub async fn get_task_logs(&self, task_id: &str, limit: usize) -> Vec<crate::commands::task::LogEntry> {
        let mut logs = Vec::new();
        if let Some(handle) = self.handles.get(task_id) {
            let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            logs.push(crate::commands::task::LogEntry {
                timestamp: now.clone(),
                level: "info".to_string(),
                message: format!("任务 {} 正在运行，协议: {:?}", task_id, handle.protocol),
            });
            logs.push(crate::commands::task::LogEntry {
                timestamp: now,
                level: "info".to_string(),
                message: format!("文件: {}", handle.file_name),
            });
        }
        logs.truncate(limit);
        logs
    }

    /// 获取种子文件列表（不启动下载，结果缓存加速重复解析）
    pub async fn get_torrent_file_list(
        &mut self,
        torrent_source: &str,
    ) -> Result<crate::engine::bt::TorrentFileListResponse> {
        self.bt_engine.get_torrent_file_list(torrent_source).await
    }

    /// 添加 BT/Magnet 任务（支持文件选择）
    pub async fn add_bt_task_with_files(
        &mut self,
        task: TaskStatus,
        params: &TaskParams,
        only_files: Option<Vec<usize>>,
    ) -> Result<()> {
        let task_id = task.id.clone();
        info!("添加 BT 任务: {} ({})", task.name, task_id);

        // 创建取消令牌和暂停信号
        let cancel_token = CancellationToken::new();
        let (pause_tx, pause_rx) = watch::channel(false);

        // 保存任务句柄
        self.handles.insert(
            task_id.clone(),
            TaskHandle {
                cancel_token: cancel_token.clone(),
                pause_tx,
                url: task.url.clone(),
                save_path: task.save_path.clone(),
                file_name: task.name.clone(),
                proxy: params.proxy.clone(),
                speed_limit: params.speed_limit,
                protocol: task.protocol.clone(),
                paused: false,
            },
        );

        let update_tx = self.update_tx.clone();
        let result = self.bt_engine
            .add_torrent_with_files(
                &task_id,
                &task.url,
                &task.save_path,
                only_files,
                update_tx,
            )
            .await;

        if let Err(e) = result {
            error!("BT 任务添加失败: {} - {}", task_id, e);
            self.handles.remove(&task_id);
            return Err(e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_manager_creation() {
        let manager = TaskManager::new(3, 0, 0, BtConfig::default());
        assert_eq!(manager.max_concurrent(), 3);
    }

    #[test]
    fn test_set_max_concurrent() {
        let mut manager = TaskManager::new(3, 0, 0, BtConfig::default());
        manager.set_max_concurrent(10);
        assert_eq!(manager.max_concurrent(), 10);
    }

    #[test]
    fn test_set_global_speed_limit() {
        let mut manager = TaskManager::new(3, 0, 0, BtConfig::default());
        manager.set_global_download_speed_limit(1024 * 1024); // 1MB/s
        manager.set_global_upload_speed_limit(512 * 1024); // 512KB/s
        // 不应 panic
    }

    #[test]
    fn test_set_retry_config() {
        let mut manager = TaskManager::new(3, 0, 0, BtConfig::default());
        manager.set_retry_config(5, 10);
        // 不应 panic
    }

    #[test]
    fn test_has_task_empty() {
        let manager = TaskManager::new(3, 0, 0, BtConfig::default());
        assert!(!manager.has_task("nonexistent"));
    }

    #[test]
    fn test_task_state_transitions() {
        // 验证任务状态枚举的正确性
        assert_eq!(TaskState::Downloading.as_str(), "downloading");
        assert_eq!(TaskState::Paused.as_str(), "paused");
        assert_eq!(TaskState::Done.as_str(), "done");
        assert_eq!(TaskState::Error.as_str(), "error");
        assert_eq!(TaskState::Queued.as_str(), "queued");
        assert_eq!(TaskState::Seeding.as_str(), "seeding");
    }

    #[test]
    fn test_task_state_from_str() {
        assert_eq!(TaskState::from_str("downloading"), TaskState::Downloading);
        assert_eq!(TaskState::from_str("paused"), TaskState::Paused);
        assert_eq!(TaskState::from_str("done"), TaskState::Done);
        assert_eq!(TaskState::from_str("error"), TaskState::Error);
        assert_eq!(TaskState::from_str("unknown"), TaskState::Queued); // 默认
    }

    #[test]
    fn test_protocol_detection() {
        use crate::util::detect_protocol;
        assert_eq!(detect_protocol("http://example.com/file.zip"), "HTTP");
        assert_eq!(detect_protocol("https://example.com/file.zip"), "HTTP");
        assert_eq!(detect_protocol("ftp://example.com/file.zip"), "FTP");
        assert_eq!(detect_protocol("magnet:?xt=urn:btih:abc"), "MAGNET");
        assert_eq!(detect_protocol("ed2k://|file|test|123|abc|/"), "ED2K");
        assert_eq!(detect_protocol("https://example.com/stream.m3u8"), "HLS");
    }
}
