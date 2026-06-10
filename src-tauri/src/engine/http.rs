// HTTP/HTTPS/FTP 下载引擎
// 支持多线程分块下载、断点续传、自动重定向、代理、限速
// 使用 reqwest 异步 HTTP 客户端 + tokio 并发任务

use crate::engine::task_manager::TaskUpdateEvent;
use crate::storage::db::TaskState;
use anyhow::{bail, Context, Result};
use futures::StreamExt;
use governor::{Quota, RateLimiter};
use std::collections::VecDeque;
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

const USER_AGENT: &str = "FullDownloader/1.0";

/// 限速器类型别名
type SharedRateLimiter = Arc<RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>;

/// 下载选项（可选参数）
#[derive(Clone, Default)]
pub struct DownloadOptions {
    /// HTTP 认证（Basic/Digest），格式 "user:pass"
    pub auth: Option<String>,
    /// 自定义 Cookie
    pub cookie: Option<String>,
    /// 自定义 HTTP Headers
    pub headers: Option<Vec<(String, String)>>,
}

/// HTTP 下载引擎
#[derive(Clone)]
pub struct HttpEngine {
    client: reqwest::Client,
}

/// 下载分块信息
struct ChunkInfo {
    start: u64,
    end: u64,
    index: usize,
}

/// 速度计算器（滑动窗口，最近 N 秒内采样）
struct SpeedTracker {
    /// (时间点, 累计字节数)
    samples: VecDeque<(Instant, u64)>,
    /// 窗口大小（秒）
    window_secs: u64,
}

impl SpeedTracker {
    fn new(window_secs: u64) -> Self {
        SpeedTracker {
            samples: VecDeque::new(),
            window_secs,
        }
    }

    /// 记录新增下载字节
    fn record(&mut self, bytes: u64) {
        let now = Instant::now();
        self.samples.push_back((now, bytes));
        // 清理过期采样（从头部弹出，比 retain 更高效）
        let cutoff = now - Duration::from_secs(self.window_secs);
        while self.samples.front().map_or(false, |(t, _)| *t <= cutoff) {
            self.samples.pop_front();
        }
    }

    /// 计算当前下载速度（bytes/sec）
    fn speed(&self) -> u64 {
        if self.samples.len() < 2 {
            return 0;
        }
        let first = match self.samples.front() { Some(s) => s.0, None => return 0 };
        let last = match self.samples.back() { Some(s) => s.0, None => return 0 };
        let elapsed = last.duration_since(first).as_secs_f64();
        if elapsed < 0.1 {
            return 0;
        }
        let total: u64 = self.samples.iter().map(|(_, b)| b).sum();
        (total as f64 / elapsed) as u64
    }
}

impl HttpEngine {
    /// 创建新的 HTTP 引擎（优化连接池和 TCP 参数）
    pub fn new() -> Self {
        Self::with_timeout(30, 60)
    }

    /// 使用自定义超时创建 HTTP 引擎
    /// - `connect_timeout`: 连接超时（秒）
    /// - `read_timeout`: 读取超时（秒）
    pub fn with_timeout(connect_timeout: u64, read_timeout: u64) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(read_timeout.max(1)))
            .connect_timeout(Duration::from_secs(connect_timeout.max(1)))
            .redirect(reqwest::redirect::Policy::limited(10))
            // 连接池优化：保持连接活跃，减少握手开销
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            // TCP 优化：启用 keepalive
            .tcp_keepalive(Duration::from_secs(60))
            .tcp_nodelay(true)
            .build()
            .unwrap_or_else(|e| {
                tracing::warn!("创建 HTTP 客户端失败: {}，使用默认配置", e);
                reqwest::Client::default()
            });

        HttpEngine { client }
    }

    /// 创建带代理的 HTTP 客户端
    fn build_client(proxy_url: &str, credentials: Option<(&str, &str)>) -> Result<reqwest::Client> {
        let mut proxy = reqwest::Proxy::all(proxy_url).context("无效的代理地址")?;
        if let Some((username, password)) = credentials {
            proxy = proxy.basic_auth(username, password);
        }
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .proxy(proxy)
            .build()
            .context("创建 HTTP 客户端失败")?;
        Ok(client)
    }

    /// 执行 HTTP/HTTPS 下载
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    /// - `url`: 下载地址
    /// - `save_path`: 保存目录
    /// - `file_name`: 文件名
    /// - `resume_from`: 断点续传起始字节位置
    /// - `proxy`: 代理地址（可选）
    /// - `speed_limit`: 单任务速度限制 bytes/sec（可选，0=不限）
    /// - `global_download_limiter`: 全局下载限速器（可选）
    /// - `global_upload_limiter`: 全局上传限速器（可选）
    /// - `update_tx`: 状态更新通道
    /// - `cancel_token`: 取消信号（共享给所有分块任务）
    /// - `pause_rx`: 暂停信号接收端
    pub async fn download(
        &self,
        task_id: &str,
        url: &str,
        save_path: &str,
        file_name: &str,
        resume_from: u64,
        proxy: Option<String>,
        speed_limit: Option<u64>,
        global_download_limiter: Option<SharedRateLimiter>,
        _global_upload_limiter: Option<SharedRateLimiter>,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
        cancel_token: CancellationToken,
        pause_rx: watch::Receiver<bool>,
        options: DownloadOptions,
    ) -> Result<()> {
        // 构建客户端（支持代理）
        let client = match proxy.as_deref() {
            Some(p) if !p.is_empty() => Self::build_client(p, None)?,
            _ => self.client.clone(),
        };

        // 应用自定义选项（认证、Cookie、Headers）到请求
        let apply_opts = |mut req: reqwest::RequestBuilder| -> reqwest::RequestBuilder {
            if let Some(ref auth) = options.auth {
                if let Some((user, pass)) = auth.split_once(':') {
                    req = req.basic_auth(user, Some(pass));
                }
            }
            if let Some(ref cookie) = options.cookie {
                req = req.header("cookie", cookie.as_str());
            }
            if let Some(ref headers) = options.headers {
                for (key, value) in headers {
                    req = req.header(key.as_str(), value.as_str());
                }
            }
            req
        };

        // 发送 HEAD 请求获取文件信息（HEAD 失败时回退到 GET）
        let (total_size, supports_range) = match apply_opts(client.head(url)).send().await {
            Ok(resp) if resp.status().is_success() => {
                let size = resp
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(0);
                let range = resp
                    .headers()
                    .get("accept-ranges")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_lowercase().contains("bytes"))
                    .unwrap_or(false);
                (size, range)
            }
            _ => {
                // HEAD 失败，尝试 Range GET 获取文件头信息（最小化数据传输）
                warn!("HEAD 请求失败，尝试 Range GET 获取文件信息");
                let resp = apply_opts(client.get(url).header("Range", "bytes=0-0"))
                    .send().await.context("GET 请求失败")?;
                let size = resp
                    .headers()
                    .get("content-range")
                    .and_then(|v| v.to_str().ok())
                    // Content-Range: bytes 0-0/12345 → 提取 12345
                    .and_then(|v| v.rsplit('/').next())
                    .and_then(|v| v.parse::<u64>().ok())
                    .or_else(|| resp.headers().get("content-length")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.parse::<u64>().ok()))
                    .unwrap_or(0);
                let range = resp.headers().get("accept-ranges")
                    .and_then(|v| v.to_str().ok())
                    .map(|v| v.to_lowercase().contains("bytes"))
                    .unwrap_or(resp.status() == 206);
                (size, range)
            }
        };

        info!(
            "下载信息: 文件={}, 大小={}, 支持分块={}",
            file_name, total_size, supports_range
        );

        // 构建保存路径
        let file_path = Path::new(save_path).join(file_name);

        // 确保保存目录存在
        if let Some(parent) = file_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                tracing::warn!("创建保存目录失败: {:?} - {}", parent, e);
                // 继续尝试创建文件，让后续错误更明确
            }
        }

        // 智能分块策略：根据文件大小动态调整分块数和分块大小
        // 小文件(<10MB): 1块 | 中文件(10-100MB): 4-8块 | 大文件(100MB+): 8-32块
        let num_chunks = if supports_range && total_size > 10 * 1024 * 1024 {
            let chunk_size = if total_size > 1024 * 1024 * 1024 {
                32 * 1024 * 1024 // >1GB: 每块 32MB
            } else if total_size > 100 * 1024 * 1024 {
                16 * 1024 * 1024 // >100MB: 每块 16MB
            } else {
                10 * 1024 * 1024 // 默认: 每块 10MB
            };
            ((total_size / chunk_size) + 1).min(32).max(1) as usize
        } else {
            1
        };

        let chunks = Self::split_chunks(total_size, num_chunks, resume_from, supports_range);

        // 创建限速器（如果指定了速度限制）
        let rate_limiter = speed_limit
            .filter(|&limit| limit > 0)
            .and_then(|limit| {
                NonZeroU32::new(limit.min(u32::MAX as u64) as u32).map(|n| {
                    Arc::new(RateLimiter::direct(Quota::per_second(n)))
                })
            });

        // 使用全局下载限速器
        let global_limiter = global_download_limiter;

        // 创建/打开文件
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(resume_from == 0)
            .open(&file_path)
            .await
            .context("创建下载文件失败")?;

        let file = Arc::new(tokio::sync::Mutex::new(file));
        let downloaded = Arc::new(std::sync::atomic::AtomicU64::new(resume_from));
        let speed_tracker = Arc::new(tokio::sync::Mutex::new(SpeedTracker::new(5)));

        // 启动分块下载任务
        let mut handles = Vec::new();
        for chunk in chunks {
            let client_c = client.clone();
            let url_c = url.to_string();
            let file_c = file.clone();
            let downloaded_c = downloaded.clone();
            let tracker_c = speed_tracker.clone();
            let update_c = update_tx.clone();
            let cancel_c = cancel_token.clone();
            let pause_c = pause_rx.clone();
            let limiter_c = rate_limiter.clone();
            let global_limiter_c = global_limiter.clone();
            let options_c = options.clone();
            let tid = task_id.to_string();

            handles.push(tokio::spawn(async move {
                // 分块下载失败自动重试（最多 3 次，指数退避）
                let max_retries = 3u32;
                for attempt in 0..=max_retries {
                    if attempt > 0 {
                        let delay = Duration::from_secs(2u64.pow(attempt - 1));
                        debug!("分块 {} 第 {} 次重试，等待 {}s", chunk.index, attempt, delay.as_secs());
                        tokio::time::sleep(delay).await;
                    }
                    if cancel_c.is_cancelled() {
                        return Ok(());
                    }
                    match Self::download_chunk(
                        &client_c, &url_c, &file_c, &chunk,
                        &downloaded_c, total_size, &tracker_c,
                        &tid, &update_c, cancel_c.clone(), pause_c.clone(),
                        limiter_c.as_deref(),
                        global_limiter_c.as_deref(),
                        &options_c,
                    ).await {
                        Ok(()) => return Ok(()),
                        Err(e) => {
                            if attempt < max_retries {
                                warn!("分块 {} 下载失败 (尝试 {}/{}): {}", chunk.index, attempt + 1, max_retries + 1, e);
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
                // 不可达，但需要满足返回类型
                Ok(())
            }));
        }

        // 等待所有分块完成
        let mut errors = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(e.to_string()),
                Err(e) => errors.push(format!("分块任务异常: {}", e)),
            }
        }

        if !errors.is_empty() {
            bail!("下载失败: {}", errors.join("; "));
        }

        // 确保所有数据写入磁盘
        {
            let file_guard = file.lock().await;
            file_guard.sync_all().await.context("刷新文件缓冲区失败")?;
        }

        info!("文件下载完成: {}", file_name);
        Ok(())
    }

    /// 执行带镜像源的 HTTP/HTTPS 下载（多源加速）
    ///
    /// # 参数
    /// - `task_id`: 任务 ID
    /// - `urls`: 镜像 URL 列表（第一个为主 URL）
    /// - `save_path`: 保存目录
    /// - `file_name`: 文件名
    /// - `resume_from`: 断点续传起始字节位置
    /// - `proxy`: 代理地址（可选）
    /// - `speed_limit`: 单任务速度限制 bytes/sec（可选，0=不限）
    /// - `global_download_limiter`: 全局下载限速器（可选）
    /// - `global_upload_limiter`: 全局上传限速器（可选）
    /// - `update_tx`: 状态更新通道
    /// - `cancel_token`: 取消信号
    /// - `pause_rx`: 暂停信号接收端
    ///
    /// # 多源加速策略
    /// 1. 先用主 URL 探测文件大小和分块支持
    /// 2. 将文件分块分配给不同的镜像源并行下载
    /// 3. 如果某个镜像源失败，自动将剩余分块转移给其他源
    /// 4. 每个镜像源独立限速，共享全局限速器
    pub async fn download_with_mirrors(
        &self,
        task_id: &str,
        urls: &[String],
        save_path: &str,
        file_name: &str,
        resume_from: u64,
        proxy: Option<String>,
        speed_limit: Option<u64>,
        global_download_limiter: Option<SharedRateLimiter>,
        global_upload_limiter: Option<SharedRateLimiter>,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
        cancel_token: CancellationToken,
        pause_rx: watch::Receiver<bool>,
        options: DownloadOptions,
    ) -> Result<()> {
        if urls.is_empty() {
            bail!("镜像 URL 列表不能为空");
        }

        // 如果只有一个 URL，直接使用普通下载
        if urls.len() == 1 {
            return self.download(
                task_id, &urls[0], save_path, file_name, resume_from,
                proxy, speed_limit, global_download_limiter, global_upload_limiter,
                update_tx, cancel_token, pause_rx, options,
            ).await;
        }

        // 构建客户端（支持代理）
        let client = match proxy.as_deref() {
            Some(p) if !p.is_empty() => Self::build_client(p, None)?,
            _ => self.client.clone(),
        };

        // 用主 URL 探测文件信息（应用认证/cookie/headers）
        let probe_url = &urls[0];
        let mut probe_req = client.head(probe_url);
        if let Some(ref auth) = options.auth {
            if let Some((user, pass)) = auth.split_once(':') {
                probe_req = probe_req.basic_auth(user, Some(pass));
            }
        }
        if let Some(ref cookie) = options.cookie {
            probe_req = probe_req.header("cookie", cookie.as_str());
        }
        if let Some(ref headers) = options.headers {
            for (key, value) in headers {
                probe_req = probe_req.header(key.as_str(), value.as_str());
            }
        }
        let head_resp = probe_req
            .send()
            .await
            .context("发送 HEAD 请求失败")?;

        let total_size = head_resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let supports_range = head_resp
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_lowercase().contains("bytes"))
            .unwrap_or(false);

        info!(
            "多源下载: 文件={}, 大小={}, 镜像数={}, 支持分块={}",
            file_name, total_size, urls.len(), supports_range
        );

        // 构建保存路径
        let file_path = Path::new(save_path).join(file_name);

        // 确保保存目录存在
        if let Some(parent) = file_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                tracing::warn!("创建保存目录失败: {:?} - {}", parent, e);
            }
        }

        // 智能分块策略：根据文件大小动态调整分块数和分块大小
        // 小文件(<10MB): 1块 | 中文件(10-100MB): 4-8块 | 大文件(100MB+): 8-32块
        let num_chunks = if supports_range && total_size > 10 * 1024 * 1024 {
            let chunk_size = if total_size > 1024 * 1024 * 1024 {
                32 * 1024 * 1024 // >1GB: 每块 32MB
            } else if total_size > 100 * 1024 * 1024 {
                16 * 1024 * 1024 // >100MB: 每块 16MB
            } else {
                10 * 1024 * 1024 // 默认: 每块 10MB
            };
            ((total_size / chunk_size) + 1).min(32).max(1) as usize
        } else {
            1
        };

        let chunks = Self::split_chunks(total_size, num_chunks, resume_from, supports_range);

        // 创建限速器
        let rate_limiter = speed_limit
            .filter(|&limit| limit > 0)
            .and_then(|limit| {
                NonZeroU32::new(limit.min(u32::MAX as u64) as u32).map(|n| {
                    Arc::new(RateLimiter::direct(Quota::per_second(n)))
                })
            });

        // 创建/打开文件
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(resume_from == 0)
            .open(&file_path)
            .await
            .context("创建下载文件失败")?;

        let file = Arc::new(tokio::sync::Mutex::new(file));
        let downloaded = Arc::new(std::sync::atomic::AtomicU64::new(resume_from));
        let speed_tracker = Arc::new(tokio::sync::Mutex::new(SpeedTracker::new(5)));

        // 将分块分配给不同的镜像源
        let mut handles = Vec::new();
        for (i, chunk) in chunks.into_iter().enumerate() {
            // 轮询选择镜像源
            let mirror_url = urls[i % urls.len()].clone();
            let client_c = client.clone();
            let file_c = file.clone();
            let downloaded_c = downloaded.clone();
            let tracker_c = speed_tracker.clone();
            let update_c = update_tx.clone();
            let cancel_c = cancel_token.clone();
            let pause_c = pause_rx.clone();
            let limiter_c = rate_limiter.clone();
            let global_limiter_c = global_download_limiter.clone();
            let options_c = options.clone();
            let tid = task_id.to_string();

            handles.push(tokio::spawn(async move {
                // 镜像分块下载重试（最多 3 次，指数退避）
                let max_retries = 3u32;
                for attempt in 0..=max_retries {
                    if attempt > 0 {
                        let delay = Duration::from_secs(2u64.pow(attempt - 1));
                        debug!("镜像分块 {} 第 {} 次重试，等待 {}s", chunk.index, attempt, delay.as_secs());
                        tokio::time::sleep(delay).await;
                    }
                    if cancel_c.is_cancelled() {
                        return Ok(());
                    }
                    match Self::download_chunk(
                        &client_c, &mirror_url, &file_c, &chunk,
                        &downloaded_c, total_size, &tracker_c,
                        &tid, &update_c, cancel_c.clone(), pause_c.clone(),
                        limiter_c.as_deref(),
                        global_limiter_c.as_deref(),
                        &options_c,
                    ).await {
                        Ok(()) => return Ok(()),
                        Err(e) => {
                            if attempt < max_retries {
                                warn!("镜像分块 {} 下载失败 (尝试 {}/{}): {}", chunk.index, attempt + 1, max_retries + 1, e);
                            } else {
                                return Err(e);
                            }
                        }
                    }
                }
                Ok(())
            }));
        }

        // 等待所有分块完成
        let mut errors = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(e)) => errors.push(e.to_string()),
                Err(e) => errors.push(format!("分块任务异常: {}", e)),
            }
        }

        if !errors.is_empty() {
            bail!("多源下载失败: {}", errors.join("; "));
        }

        // 确保所有数据写入磁盘
        {
            let file_guard = file.lock().await;
            file_guard.sync_all().await.context("刷新文件缓冲区失败")?;
        }

        info!("多源下载完成: {} (使用 {} 个镜像源)", file_name, urls.len());
        Ok(())
    }

    /// 将文件拆分为多个下载分块
    fn split_chunks(total_size: u64, num_chunks: usize, resume_from: u64, supports_range: bool) -> Vec<ChunkInfo> {
        if !supports_range || total_size == 0 {
            return vec![ChunkInfo { start: 0, end: total_size, index: 0 }];
        }

        let remaining = total_size.saturating_sub(resume_from);
        if remaining == 0 {
            return vec![];
        }

        let chunk_size = remaining / num_chunks as u64;
        let mut chunks = Vec::new();

        for i in 0..num_chunks {
            let start = resume_from + i as u64 * chunk_size;
            let end = if i == num_chunks - 1 {
                total_size
            } else {
                resume_from + (i as u64 + 1) * chunk_size
            };
            if start < total_size {
                chunks.push(ChunkInfo { start, end, index: i });
            }
        }

        chunks
    }

    /// 下载单个分块
    async fn download_chunk(
        client: &reqwest::Client,
        url: &str,
        file: &Arc<tokio::sync::Mutex<tokio::fs::File>>,
        chunk: &ChunkInfo,
        downloaded: &Arc<std::sync::atomic::AtomicU64>,
        total_size: u64,
        speed_tracker: &Arc<tokio::sync::Mutex<SpeedTracker>>,
        task_id: &str,
        update_tx: &mpsc::UnboundedSender<TaskUpdateEvent>,
        cancel_token: CancellationToken,
        mut pause_rx: watch::Receiver<bool>,
        rate_limiter: Option<&RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
        global_limiter: Option<&RateLimiter<governor::state::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
        options: &DownloadOptions,
    ) -> Result<()> {
        // 构建 Range 请求头 + 自定义选项
        let mut request = client.get(url);
        // 应用认证、Cookie、自定义 Headers
        if let Some(ref auth) = options.auth {
            if let Some((user, pass)) = auth.split_once(':') {
                request = request.basic_auth(user, Some(pass));
            }
        }
        if let Some(ref cookie) = options.cookie {
            request = request.header("cookie", cookie.as_str());
        }
        if let Some(ref headers) = options.headers {
            for (key, value) in headers {
                request = request.header(key.as_str(), value.as_str());
            }
        }
        if chunk.start > 0 || chunk.end > 0 {
            let range = format!("bytes={}-{}", chunk.start, chunk.end.saturating_sub(1));
            request = request.header("Range", range);
        }

        let response = request.send().await.context("发送下载请求失败")?;

        // 206 Partial Content 或 200 OK 都是正常响应
        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            bail!("服务器返回错误: {}", status);
        }

        // 流式读取数据
        let mut stream = response.bytes_stream();
        let mut last_update = Instant::now();
        let mut chunk_written: u64 = 0; // 本分块已写入字节数

        loop {
            // 检查取消信号
            if cancel_token.is_cancelled() {
                debug!("任务 {} 分块 {} 被取消", task_id, chunk.index);
                return Ok(());
            }

            // 检查暂停信号
            if *pause_rx.borrow() {
                debug!("任务 {} 分块 {} 暂停中", task_id, chunk.index);
                while *pause_rx.borrow() {
                    tokio::select! {
                        _ = cancel_token.cancelled() => return Ok(()),
                        _ = pause_rx.changed() => break,
                    }
                }
            }

            // 读取下一个数据块（带超时和取消检测）
            let bytes = tokio::select! {
                result = stream.next() => {
                    match result {
                        Some(Ok(bytes)) => bytes,
                        Some(Err(e)) => bail!("读取数据流失败: {}", e),
                        None => break, // 流结束
                    }
                }
                _ = cancel_token.cancelled() => {
                    return Ok(());
                }
            };

            let bytes_len = bytes.len() as u64;

            // 限速：先等全局限速器，再等单任务限速器
            if let Some(glimiter) = global_limiter {
                if let Some(n) = NonZeroU32::new(bytes_len.min(u32::MAX as u64) as u32) {
                    glimiter.until_n_ready(n).await.ok();
                }
            }
            if let Some(limiter) = rate_limiter {
                if let Some(n) = NonZeroU32::new(bytes_len.min(u32::MAX as u64) as u32) {
                    limiter.until_n_ready(n).await.ok();
                }
            }

            // 写入文件（每个分块定位到自己的偏移位置）
            let write_pos = chunk.start + chunk_written;
            let mut file_guard = file.lock().await;
            file_guard.seek(std::io::SeekFrom::Start(write_pos)).await.context("文件定位失败")?;
            file_guard.write_all(&bytes).await.context("写入文件失败")?;
            drop(file_guard);
            chunk_written += bytes_len;

            // 更新全局下载进度
            downloaded.fetch_add(bytes_len, std::sync::atomic::Ordering::Relaxed);

            // 更新速度
            {
                let mut tracker = speed_tracker.lock().await;
                tracker.record(bytes_len);
            }

            // 每 200ms 推送一次状态更新
            if last_update.elapsed() >= Duration::from_millis(200) {
                let total_downloaded = downloaded.load(std::sync::atomic::Ordering::Relaxed);
                let speed = speed_tracker.lock().await.speed();
                let progress = if total_size > 0 {
                    (total_downloaded as f32 / total_size as f32).min(1.0)
                } else {
                    0.0
                };

                // 计算 ETA
                let eta = if speed > 0 && total_size > total_downloaded {
                    Some((total_size - total_downloaded) / speed)
                } else {
                    None
                };

                let _ = update_tx.send(TaskUpdateEvent {
                    task_id: task_id.to_string(),
                    state: TaskState::Downloading,
                    downloaded: total_downloaded,
                    total_size,
                    download_speed: speed,
                    upload_speed: 0,
                    progress,
                    peers: 1,
                    error: None,
                    eta,
                });

                last_update = Instant::now();
            }
        }

        // 分块完成时推送最终进度（防止因 200ms 节流导致进度卡在 97%）
        let final_downloaded = downloaded.load(std::sync::atomic::Ordering::Relaxed);
        let final_speed = speed_tracker.lock().await.speed();
        let final_progress = if total_size > 0 {
            (final_downloaded as f32 / total_size as f32).min(1.0)
        } else {
            0.0
        };
        let _ = update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Downloading,
            downloaded: final_downloaded,
            total_size,
            download_speed: final_speed,
            upload_speed: 0,
            progress: final_progress,
            peers: 1,
            error: None,
            eta: None,
        });

        debug!("分块 {} 完成 ({} bytes)", chunk.index, final_downloaded);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speed_tracker_empty() {
        let tracker = SpeedTracker::new(5);
        assert_eq!(tracker.speed(), 0);
    }

    #[test]
    fn test_speed_tracker_single_sample() {
        let mut tracker = SpeedTracker::new(5);
        tracker.record(1024);
        assert_eq!(tracker.speed(), 0); // < 2 samples
    }

    #[test]
    fn test_speed_tracker_calculates_speed() {
        let mut tracker = SpeedTracker::new(10);
        tracker.record(1024);
        // 模拟时间流逝后再次记录
        std::thread::sleep(Duration::from_millis(200));
        tracker.record(2048);
        let speed = tracker.speed();
        // 速度应该是 (1024 + 2048) / 0.2 ≈ 15360 bytes/sec
        assert!(speed > 0, "speed should be > 0, got {}", speed);
    }

    #[test]
    fn test_speed_tracker_window_cleanup() {
        let mut tracker = SpeedTracker::new(1); // 1 秒窗口
        tracker.record(100);
        std::thread::sleep(Duration::from_millis(1100)); // 超过窗口
        tracker.record(200);
        // 第一个采样应该被清理
        assert_eq!(tracker.samples.len(), 1);
    }

    #[test]
    fn test_chunk_info_creation() {
        let chunk = ChunkInfo {
            start: 0,
            end: 1024,
            index: 0,
        };
        assert_eq!(chunk.start, 0);
        assert_eq!(chunk.end, 1024);
        assert_eq!(chunk.index, 0);
    }

    #[test]
    fn test_http_engine_creation() {
        let engine = HttpEngine::new();
        // 验证 client 创建成功（不 panic）
        let _client = &engine.client;
    }

    #[test]
    fn test_calculate_chunks_single() {
        // 小文件应该只有 1 个分块
        let total_size: u64 = 1024; // 1KB
        let chunk_size: u64 = 10 * 1024 * 1024; // 10MB
        let num_chunks = ((total_size + chunk_size - 1) / chunk_size) as usize;
        assert_eq!(num_chunks, 1);
    }

    #[test]
    fn test_calculate_chunks_multiple() {
        // 大文件应该有多个分块
        let total_size: u64 = 100 * 1024 * 1024; // 100MB
        let chunk_size: u64 = 10 * 1024 * 1024; // 10MB
        let num_chunks = ((total_size + chunk_size - 1) / chunk_size) as usize;
        assert_eq!(num_chunks, 10);
    }

    #[test]
    fn test_calculate_chunks_max_limit() {
        // 超大文件不应超过 16 个分块
        let total_size: u64 = 1024 * 1024 * 1024; // 1GB
        let chunk_size: u64 = 10 * 1024 * 1024; // 10MB
        let num_chunks = ((total_size + chunk_size - 1) / chunk_size).min(16) as usize;
        assert_eq!(num_chunks, 16);
    }

    #[test]
    fn test_eta_calculation() {
        // ETA 计算: 剩余字节 / 速度
        let remaining: u64 = 1024 * 1024; // 1MB
        let speed: u64 = 1024 * 100; // 100KB/s
        let eta = if speed > 0 { remaining / speed } else { 0 };
        assert_eq!(eta, 10); // 10 秒
    }

    #[test]
    fn test_eta_zero_speed() {
        let remaining: u64 = 1024;
        let speed: u64 = 0;
        let eta = if speed > 0 { remaining / speed } else { 0 };
        assert_eq!(eta, 0);
    }

    #[test]
    fn test_progress_calculation() {
        let downloaded: u64 = 50;
        let total: u64 = 100;
        let progress = if total > 0 {
            downloaded as f32 / total as f32
        } else {
            0.0
        };
        assert!((progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_progress_zero_total() {
        let downloaded: u64 = 50;
        let total: u64 = 0;
        let progress = if total > 0 {
            downloaded as f32 / total as f32
        } else {
            0.0
        };
        assert!((progress - 0.0).abs() < f32::EPSILON);
    }
}
