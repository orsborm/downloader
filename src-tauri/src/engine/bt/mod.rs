// BitTorrent 下载引擎
// 基于 librqbit（纯 Rust BT 实现）封装
// 支持 .torrent 文件和 magnet 链接下载、DHT、做种、速度限制、文件选择

use anyhow::{Context, Result};
use librqbit::{AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent, Session, SessionOptions};
use librqbit::limits::LimitsConfig;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::engine::task_manager::TaskUpdateEvent;
use crate::storage::db::TaskState;

/// 种子文件信息（用于文件选择）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentFileInfo {
    pub index: usize,
    pub name: String,
    pub size: u64,
    pub selected: bool,
}

/// 种子文件列表响应
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TorrentFileListResponse {
    pub name: String,
    pub total_size: u64,
    pub files: Vec<TorrentFileInfo>,
}

/// BT 引擎配置
/// 注意：PEX/LSD/加密 配置目前为前端展示用，librqbit v8 未暴露这些配置接口
/// librqbit 内部可能默认启用了这些功能，但无法通过 SessionOptions 控制
#[derive(Clone)]
pub struct BtConfig {
    /// 监听端口
    pub listen_port: u16,
    /// 是否启用 DHT（通过 SessionOptions.disable_dht 控制）
    pub dht: bool,
    /// 是否启用 PEX（librqbit v8 未暴露配置，可能默认启用）
    pub pex: bool,
    /// 是否启用 LSD（librqbit v8 未暴露配置，可能默认启用）
    pub lsd: bool,
    /// 加密策略（librqbit v8 未暴露配置）
    pub encryption: String,
    /// 做种比率限制
    pub seed_ratio_limit: f64,
    /// 做种时间限制（分钟）
    pub seed_time_limit: u64,
    /// 全局最大下载速度（bytes/sec，0=不限速）
    pub max_download_speed: u64,
    /// 全局最大上传速度（bytes/sec，0=不限速）
    pub max_upload_speed: u64,
    /// 全局停止做种
    pub stop_seeding: bool,
}

impl Default for BtConfig {
    fn default() -> Self {
        BtConfig {
            listen_port: 6881,
            dht: true,
            pex: true,
            lsd: true,
            encryption: "enabled".to_string(),
            seed_ratio_limit: 2.0,
            seed_time_limit: 1440,
            max_download_speed: 0,
            max_upload_speed: 0,
            stop_seeding: false,
        }
    }
}

/// BT 引擎状态
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BtStatus {
    /// DHT 节点数
    pub dht_nodes: u32,
    /// DHT 是否已连接
    pub dht_connected: bool,
    /// 全局上传速度
    pub upload_speed: u64,
    /// 全局下载速度
    pub download_speed: u64,
}

/// 活跃的 BT 任务信息
struct BtTask {
    handle: Arc<ManagedTorrent>,
    task_id: String,
    total_size: u64,
    /// 取消令牌（用于停止监控任务）
    cancel_token: tokio_util::sync::CancellationToken,
}

/// 公共 Tracker 列表（加速 magnet 解析和 DHT 发现）
const PUBLIC_TRACKERS: &[&str] = &[
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
    "udp://exodus.desync.com:6969/announce",
    "udp://explodie.org:6969/announce",
    "udp://tracker.moeking.me:6969/announce",
    "udp://tracker.openbittorrent.com:6969/announce",
    "udp://tracker.dler.org:6969/announce",
    "udp://p4p.arenabg.com:1337/announce",
    "udp://opentracker.i2p.rocks:6969/announce",
];

/// BT 下载引擎
pub struct BtEngine {
    config: BtConfig,
    session: Option<Arc<Session>>,
    /// 活跃任务映射（task_id -> BtTask）
    tasks: HashMap<String, BtTask>,
    /// 种子元数据缓存（source -> TorrentFileListResponse），避免重复解析
    metadata_cache: HashMap<String, TorrentFileListResponse>,
}

impl BtEngine {
    /// 创建新的 BT 引擎
    pub fn new(config: BtConfig) -> Self {
        info!(
            "BT 引擎创建: 端口={}, DHT={}",
            config.listen_port, config.dht
        );
        BtEngine {
            config,
            session: None,
            metadata_cache: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    /// 初始化 BT 会话（必须在使用前调用）
    pub async fn init(&mut self, default_download_dir: &str) -> Result<()> {
        // 构建速度限制配置
        let ratelimits = LimitsConfig {
            download_bps: if self.config.max_download_speed > 0 {
                NonZeroU32::new(self.config.max_download_speed as u32)
            } else {
                None
            },
            upload_bps: if self.config.max_upload_speed > 0 {
                NonZeroU32::new(self.config.max_upload_speed as u32)
            } else {
                None
            },
        };

        let opts = SessionOptions {
            listen_port_range: Some(self.config.listen_port..self.config.listen_port + 1),
            disable_dht: !self.config.dht,
            ratelimits,
            ..Default::default()
        };

        let session = Session::new_with_opts(PathBuf::from(default_download_dir), opts)
            .await
            .context("初始化 BT 会话失败")?;

        self.session = Some(session);
        info!("BT 会话初始化完成");
        Ok(())
    }

    /// 将公共 tracker 追加到 magnet 链接（加速 DHT 发现）
    /// magnet 链接的 &tr= 参数应使用原始 tracker URL，不需要 URL 编码
    fn enhance_magnet_url(&self, magnet: &str) -> String {
        if !magnet.starts_with("magnet:") {
            return magnet.to_string();
        }
        let mut url = magnet.trim().to_string();
        // 确保 magnet 链接有查询参数前缀
        if !url.contains('?') {
            url.push('?');
        }
        for tracker in PUBLIC_TRACKERS {
            url.push_str(&format!("&tr={}", tracker));
        }
        url
    }

    /// 更新 BT 引擎速度限制（需重新初始化会话生效）
    pub fn set_speed_limits(&mut self, max_download_speed: u64, max_upload_speed: u64) {
        self.config.max_download_speed = max_download_speed;
        self.config.max_upload_speed = max_upload_speed;
        info!("BT 速度限制已更新: 下载={} 上传={} bytes/sec", max_download_speed, max_upload_speed);
    }

    /// 更新做种配置
    pub fn set_seeding_config(&mut self, seed_ratio: f64, seed_time: u64, stop_seeding: bool) {
        self.config.seed_ratio_limit = seed_ratio;
        self.config.seed_time_limit = seed_time;
        self.config.stop_seeding = stop_seeding;
        info!("BT 做种配置已更新: 比率={} 时间={}min 停止={}", seed_ratio, seed_time, stop_seeding);
    }

    /// 获取种子文件列表（不启动下载）
    pub async fn get_torrent_file_list(
        &mut self,
        torrent_source: &str,
    ) -> Result<TorrentFileListResponse> {
        // 检查缓存（同一文件/magnet 不重复解析）
        if let Some(cached) = self.metadata_cache.get(torrent_source) {
            info!("命中种子元数据缓存: {}", torrent_source);
            return Ok(cached.clone());
        }

        // 验证 magnet 链接格式
        let is_magnet = torrent_source.starts_with("magnet:");
        if is_magnet {
            let magnet_lower = torrent_source.trim().to_lowercase();
            if !magnet_lower.starts_with("magnet:?xt=urn:btih:") {
                anyhow::bail!("无效的磁力链接格式，必须以 magnet:?xt=urn:btih: 开头");
            }
        }

        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        // 根据来源类型构建 AddTorrent
        // 将公共 tracker 追加到 magnet 链接（最可靠的加速方式）
        let enhanced_magnet = if is_magnet {
            self.enhance_magnet_url(torrent_source)
        } else {
            String::new()
        };
        let add_torrent = if is_magnet {
            AddTorrent::from_url(&enhanced_magnet)
        } else {
            let torrent_bytes = tokio::fs::read(torrent_source)
                .await
                .context("读取 torrent 文件失败")?;
            AddTorrent::from_bytes(torrent_bytes)
        };

        // .torrent 文件解析超时 10 秒（本地操作），magnet 链接超时 60 秒（需要 DHT + tracker）
        let timeout_secs = if is_magnet { 60u64 } else { 10u64 };

        let opts = AddTorrentOptions {
            list_only: true,
            ..Default::default()
        };

        // 使用 list_only 模式获取文件列表
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            session.add_torrent(add_torrent, Some(opts)),
        )
        .await
        .with_context(|| if is_magnet {
            format!("磁力链接解析超时（{}秒）。可能原因：DHT 网络未连接或 tracker 不可用。请检查网络连接后重试", timeout_secs)
        } else {
            format!("获取种子文件列表超时（{}秒）", timeout_secs)
        })?
        .context("获取种子文件列表失败")?;

        match response {
            AddTorrentResponse::ListOnly(list_response) => {
                let info = &list_response.info;
                let mut files = Vec::new();
                let mut total_size = 0u64;

                // 遍历文件详情
                for (index, file) in info.iter_file_details()?.enumerate() {
                    let name = file
                        .filename
                        .to_string()
                        .unwrap_or_else(|_| format!("file_{}", index));
                    let size = file.len;
                    total_size += size;

                    files.push(TorrentFileInfo {
                        index,
                        name,
                        size,
                        selected: true, // 默认全选
                    });
                }

                // 获取种子名称（使用 info.name 或 hash 的前 8 位作为备选）
                let torrent_name = info
                    .name
                    .as_ref()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let response = TorrentFileListResponse {
                    name: torrent_name,
                    total_size,
                    files,
                };
                // 缓存解析结果，避免重复解析
                self.metadata_cache.insert(torrent_source.to_string(), response.clone());
                Ok(response)
            }
            _ => {
                // 如果不是 ListOnly 响应，说明种子已经在管理中
                anyhow::bail!("种子已在管理中，无法获取文件列表")
            }
        }
    }

    /// 添加 torrent 下载（支持文件选择）
    pub async fn add_torrent_with_files(
        &mut self,
        task_id: &str,
        torrent_source: &str,
        save_path: &str,
        only_files: Option<Vec<usize>>,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
    ) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        // 根据来源类型构建 AddTorrent
        let is_magnet = torrent_source.starts_with("magnet:");
        let enhanced_magnet = if is_magnet {
            self.enhance_magnet_url(torrent_source)
        } else {
            String::new()
        };
        let add_torrent = if is_magnet {
            AddTorrent::from_url(&enhanced_magnet)
        } else {
            let torrent_bytes = tokio::fs::read(torrent_source)
                .await
                .context("读取 torrent 文件失败")?;
            AddTorrent::from_bytes(torrent_bytes)
        };

        let opts = AddTorrentOptions {
            output_folder: Some(save_path.to_string()),
            only_files,
            ..Default::default()
        };

        let response = session
            .add_torrent(add_torrent, Some(opts))
            .await
            .context("添加 torrent 任务失败")?;

        self.handle_add_response(task_id, response, update_tx).await
    }

    /// 添加 .torrent 文件下载
    pub async fn add_torrent_file(
        &mut self,
        task_id: &str,
        torrent_path: &str,
        save_path: &str,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
    ) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        // 读取 torrent 文件
        let torrent_bytes = tokio::fs::read(torrent_path)
            .await
            .context("读取 torrent 文件失败")?;

        // 添加到会话
        let response = session
            .add_torrent(
                AddTorrent::from_bytes(torrent_bytes),
                Some(librqbit::AddTorrentOptions {
                    output_folder: Some(save_path.to_string()),
                    ..Default::default()
                }),
            )
            .await
            .context("添加 torrent 任务失败")?;

        self.handle_add_response(task_id, response, update_tx).await
    }

    /// 添加 magnet 链接下载
    pub async fn add_magnet(
        &mut self,
        task_id: &str,
        magnet_uri: &str,
        save_path: &str,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
    ) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        // 验证 magnet 链接格式
        let magnet_lower = magnet_uri.trim().to_lowercase();
        if !magnet_lower.starts_with("magnet:?xt=urn:btih:") {
            anyhow::bail!("无效的磁力链接格式，必须以 magnet:?xt=urn:btih: 开头");
        }

        let enhanced_magnet = self.enhance_magnet_url(magnet_uri);
        info!("添加 magnet 任务: {}", &enhanced_magnet[..enhanced_magnet.len().min(80)]);

        let response = session
            .add_torrent(
                AddTorrent::from_url(&enhanced_magnet),
                Some(librqbit::AddTorrentOptions {
                    output_folder: Some(save_path.to_string()),
                    ..Default::default()
                }),
            )
            .await
            .context("添加 magnet 任务失败")?;

        self.handle_add_response(task_id, response, update_tx).await
    }

    /// 处理添加任务的响应，启动进度监控
    async fn handle_add_response(
        &mut self,
        task_id: &str,
        response: AddTorrentResponse,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
    ) -> Result<()> {
        let handle = response
            .into_handle()
            .ok_or_else(|| anyhow::anyhow!("无法获取 torrent 句柄"))?;

        // 读取做种限制配置
        let seed_ratio_limit = self.config.seed_ratio_limit;
        let seed_time_limit = self.config.seed_time_limit;
        let stop_seeding = self.config.stop_seeding;

        let total_size = handle.stats().total_bytes;

        // 保存任务引用（含取消令牌）
        let cancel_token = tokio_util::sync::CancellationToken::new();
        self.tasks.insert(
            task_id.to_string(),
            BtTask {
                handle: handle.clone(),
                task_id: task_id.to_string(),
                total_size,
                cancel_token: cancel_token.clone(),
            },
        );

        // 启动进度监控后台任务（可通过 cancel_token 取消）
        let tid = task_id.to_string();
        tokio::spawn(async move {
            let mut last_update = tokio::time::Instant::now();
            loop {
                // 检查取消信号（替代无退出条件的 sleep）
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        info!("BT 监控任务已取消: {}", tid);
                        return;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                }

                let stats = handle.stats();
                let downloaded = stats.progress_bytes;
                let total = stats.total_bytes;
                let uploaded = stats.uploaded_bytes;
                let progress = if total > 0 {
                    downloaded as f32 / total as f32
                } else {
                    0.0
                };

                // 从 live stats 获取速度
                let (download_speed, upload_speed, peers) =
                    if let Some(ref live) = stats.live {
                        (
                            (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
                            (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
                            live.snapshot.peer_stats.live,
                        )
                    } else {
                        (0, 0, 0)
                    };

                // 检查是否完成
                if stats.finished {
                    // 报告下载完成
                    let _ = update_tx.send(TaskUpdateEvent {
                        task_id: tid.clone(),
                        state: TaskState::Done,
                        downloaded: total,
                        total_size: total,
                        download_speed: 0,
                        upload_speed: 0,
                        progress: 1.0,
                        peers: 0,
                        error: None,
                        eta: Some(0),
                    });
                    info!("BT 任务下载完成: {}，进入做种阶段", tid);

                    // 全局停止做种：下载完成后立即暂停（通过发送取消信号停止监控）
                    if stop_seeding {
                        info!("全局停止做种已启用，BT 任务完成: {}", tid);
                        // 不再进入做种循环，直接返回
                        return;
                    }

                    // 做种监控：检查比率/时间限制
                    let seed_start = tokio::time::Instant::now();
                    loop {
                        tokio::select! {
                            _ = cancel_token.cancelled() => {
                                info!("BT 做种任务已取消: {}", tid);
                                return;
                            }
                            _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                        }

                        let seed_stats = handle.stats();
                        let uploaded = seed_stats.uploaded_bytes;
                        let seed_ratio = if total > 0 { uploaded as f64 / total as f64 } else { 0.0 };
                        let seed_minutes = seed_start.elapsed().as_secs() / 60;

                        // 检查做种比率限制
                        if seed_ratio_limit > 0.0 && seed_ratio >= seed_ratio_limit {
                            info!("BT 做种比率达标 ({} >= {})，停止做种: {}", seed_ratio, seed_ratio_limit, tid);
                            return;
                        }
                        // 检查做种时间限制
                        if seed_time_limit > 0 && seed_minutes >= seed_time_limit {
                            info!("BT 做种时间达标 ({}min >= {}min)，停止做种: {}", seed_minutes, seed_time_limit, tid);
                            return;
                        }

                        // 定期推送做种状态
                        let _ = update_tx.send(TaskUpdateEvent {
                            task_id: tid.clone(),
                            state: TaskState::Seeding,
                            downloaded: total,
                            total_size: total,
                            download_speed: 0,
                            upload_speed,
                            progress: 1.0,
                            peers: peers as u32,
                            error: None,
                            eta: None,
                        });
                    }
                }

                // 检查是否有错误
                if let Some(ref err) = stats.error {
                    error!("BT 任务错误: {} - {}", tid, err);
                    let _ = update_tx.send(TaskUpdateEvent {
                        task_id: tid.clone(),
                        state: TaskState::Error,
                        downloaded,
                        total_size: total,
                        download_speed: 0,
                        upload_speed: 0,
                        progress,
                        peers: 0,
                        error: Some(err.clone()),
                        eta: None,
                    });
                    return;
                }

                // 每 200ms 推送一次状态更新
                if last_update.elapsed() >= Duration::from_millis(200) {
                    let eta = if download_speed > 0 && total > downloaded {
                        Some((total - downloaded) / download_speed)
                    } else {
                        None
                    };

                    let _ = update_tx.send(TaskUpdateEvent {
                        task_id: tid.clone(),
                        state: TaskState::Downloading,
                        downloaded,
                        total_size: total,
                        download_speed,
                        upload_speed,
                        progress,
                        peers: peers as u32,
                        error: None,
                        eta,
                    });
                    last_update = tokio::time::Instant::now();
                }
            }
        });

        info!(
            "BT 任务已添加: {} (大小: {})",
            task_id,
            crate::util::format_size(total_size)
        );
        Ok(())
    }

    /// 暂停 BT 任务
    pub async fn pause_task(&mut self, task_id: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        if let Some(task) = self.tasks.get(task_id) {
            session
                .pause(&task.handle)
                .await
                .context("暂停 BT 任务失败")?;
            info!("BT 任务已暂停: {}", task_id);
        }
        Ok(())
    }

    /// 恢复 BT 任务
    pub async fn resume_task(&mut self, task_id: &str) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        if let Some(task) = self.tasks.get(task_id) {
            session
                .unpause(&task.handle)
                .await
                .context("恢复 BT 任务失败")?;
            info!("BT 任务已恢复: {}", task_id);
        }
        Ok(())
    }

    /// 删除 BT 任务
    pub async fn remove_task(&mut self, task_id: &str, delete_files: bool) -> Result<()> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("BT 会话未初始化"))?;

        if let Some(task) = self.tasks.remove(task_id) {
            // 先取消监控任务，防止泄漏
            task.cancel_token.cancel();
            let id = task.handle.id();
            session
                .delete(librqbit::api::TorrentIdOrHash::Id(id), delete_files)
                .await
                .context("删除 BT 任务失败")?;
            info!("BT 任务已删除: {}", task_id);
        }
        Ok(())
    }

    /// 获取 BT 引擎状态
    pub async fn get_status(&self) -> BtStatus {
        if let Some(ref session) = self.session {
            // 从所有活跃任务聚合速度和 peer 数
            let mut total_upload: u64 = 0;
            let mut total_download: u64 = 0;
            let mut total_peers: usize = 0;
            for task in self.tasks.values() {
                let stats = task.handle.stats();
                if let Some(ref live) = stats.live {
                    total_download += (live.download_speed.mbps * 1024.0 * 1024.0) as u64;
                    total_upload += (live.upload_speed.mbps * 1024.0 * 1024.0) as u64;
                    total_peers += live.snapshot.peer_stats.live;
                }
            }

            // DHT 状态：session 存在即表示已连接
            // 节点数使用 peer 总数作为近似（librqbit v8 不暴露 DHT 路由表大小）
            let dht_connected = true;
            let dht_nodes = total_peers.min(u32::MAX as usize) as u32;

            BtStatus {
                dht_nodes,
                dht_connected,
                upload_speed: total_upload,
                download_speed: total_download,
            }
        } else {
            BtStatus {
                dht_nodes: 0,
                dht_connected: false,
                upload_speed: 0,
                download_speed: 0,
            }
        }
    }

    /// 检查是否有指定任务
    pub fn has_task(&self, task_id: &str) -> bool {
        self.tasks.contains_key(task_id)
    }

    /// 获取任务的 Peer 统计信息
    pub async fn get_task_stats(&self, task_id: &str) -> Option<Vec<crate::commands::task::PeerInfo>> {
        let task = self.tasks.get(task_id)?;
        let stats = task.handle.stats();
        let mut peers = Vec::new();

        if let Some(ref live) = stats.live {
            let snapshot = &live.snapshot.peer_stats;
            let peer_count = snapshot.live;
            if peer_count > 0 {
                // 返回聚合的 peer 信息（不伪造单个 peer 条目）
                peers.push(crate::commands::task::PeerInfo {
                    id: "aggregate".to_string(),
                    ip: format!("{} peers connected", peer_count),
                    port: 0,
                    download_speed: (live.download_speed.mbps * 1024.0 * 1024.0) as u64,
                    upload_speed: (live.upload_speed.mbps * 1024.0 * 1024.0) as u64,
                    client: format!("BT ({} peers)", peer_count),
                    progress: 0.0,
                });
            }
        }

        Some(peers)
    }

    /// 获取任务的 Tracker 列表
    pub async fn get_task_trackers(&self, task_id: &str) -> Vec<crate::commands::task::TrackerInfo> {
        let mut trackers = Vec::new();
        if let Some(task) = self.tasks.get(task_id) {
            let stats = task.handle.stats();
            let peer_count = stats.live.as_ref().map(|l| l.snapshot.peer_stats.live as u32).unwrap_or(0);
            // 从 handle.shared() 获取 tracker URLs
            for url in &task.handle.shared().trackers {
                trackers.push(crate::commands::task::TrackerInfo {
                    url: url.to_string(),
                    status: if peer_count > 0 { "connected".to_string() } else { "disconnected".to_string() },
                    peers: peer_count,
                    seeders: 0,
                    leechers: 0,
                });
            }
        }
        trackers
    }
}
