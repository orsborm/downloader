// eDonkey2000 (ed2k) 协议引擎模块
// 实现 ed2k 链接下载、服务器连接、KAD 网络、源交换
// 参考 libed2k (C++) 和 eMule 的协议实现

pub mod hash;
pub mod kad;
pub mod proto;
pub mod server;
pub mod tag;
pub mod transfer;

use crate::engine::task_manager::TaskUpdateEvent;
use crate::storage::db::TaskState;
use anyhow::Result;
use hash::ED2K_CHUNK_SIZE;
use server::ServerConnection;
use std::net::SocketAddr;
use tokio::sync::{mpsc, watch};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use transfer::Ed2kTransfer;

/// 默认 ed2k 服务器列表（公共服务器，作为最后的回退）
const DEFAULT_SERVERS: &[(&str, u16)] = &[
    ("91.200.42.46", 4661),   // eMule server
    ("176.103.48.41", 4661),  // TV Underground
];

/// ed2k 引擎配置
#[derive(Debug, Clone)]
pub struct Ed2kConfig {
    /// 客户端用户名
    pub username: String,
    /// 监听端口
    pub port: u16,
    /// 是否启用 KAD
    pub kad_enabled: bool,
    /// 是否启用信用系统
    pub credit_enabled: bool,
    /// 连接超时（秒）
    pub connect_timeout: u64,
    /// 最大源数
    pub max_sources: usize,
    /// 自定义服务器列表（格式 "ip:port"，为空则使用默认列表）
    pub custom_servers: Vec<String>,
}

impl Default for Ed2kConfig {
    fn default() -> Self {
        Ed2kConfig {
            username: "Downloader".to_string(),
            port: 4661,
            kad_enabled: true,
            credit_enabled: true,
            connect_timeout: 15,
            max_sources: 50,
            custom_servers: Vec::new(),
        }
    }
}

/// ed2k 链接解析结果
#[derive(Debug, Clone)]
pub struct Ed2kLink {
    /// 文件名
    pub filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// ed2k hash（32 字节十六进制）
    pub hash: String,
    /// 服务器地址（可选）
    pub server: Option<String>,
}

/// 解析 ed2k 链接
/// 格式: ed2k://|file|<filename>|<size>|<hash>|/
pub fn parse_ed2k_link(url: &str) -> Result<Ed2kLink> {
    let url = url.trim();

    if !url.to_lowercase().starts_with("ed2k://") {
        anyhow::bail!("不是有效的 ed2k 链接");
    }

    // 去掉 ed2k:// 前缀，按 | 分割
    let body = &url[7..];
    let parts: Vec<&str> = body.split('|').collect();

    // 格式: |file|<name>|<size>|<hash>|/
    if parts.len() < 5 {
        anyhow::bail!("ed2k 链接格式不正确");
    }

    if parts[0] != "file" && parts[1] != "file" {
        anyhow::bail!("ed2k 链接类型不是 file");
    }

    let file_idx = if parts[0] == "file" { 1 } else { 2 };
    if parts.len() < file_idx + 4 {
        anyhow::bail!("ed2k 链接参数不完整");
    }

    let filename = parts[file_idx].to_string();
    let size: u64 = parts[file_idx + 1]
        .parse()
        .map_err(|_| anyhow::anyhow!("ed2k 链接文件大小无效"))?;
    let hash = parts[file_idx + 2].to_string();

    if hash.len() != 32 {
        anyhow::bail!("ed2k hash 长度无效（应为 32 字符十六进制）");
    }

    // 解析可选的服务器地址: ed2k://|file|name|size|hash|/|server,ip:port|/
    let mut server = None;
    for part in &parts[file_idx + 3..] {
        if let Some(addr) = part.strip_prefix("server,") {
            if !addr.is_empty() {
                server = Some(addr.to_string());
            }
            break;
        }
    }

    Ok(Ed2kLink {
        filename,
        size,
        hash,
        server,
    })
}

/// 将 32 字符十六进制 hash 转换为 16 字节数组
fn hex_to_hash(hex: &str) -> Result<[u8; 16]> {
    if hex.len() != 32 {
        anyhow::bail!("hash 长度必须为 32 字符");
    }
    let mut arr = [0u8; 16];
    for i in 0..16 {
        arr[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| anyhow::anyhow!("hash 不是有效的十六进制"))?;
    }
    Ok(arr)
}

/// ed2k 引擎主体
#[derive(Clone)]
pub struct Ed2kEngine {
    config: Ed2kConfig,
    kad_engine: Option<kad::KadEngine>,
}

impl Ed2kEngine {
    /// 创建新的 ed2k 引擎
    pub fn new(config: Ed2kConfig) -> Self {
        info!("ed2k 引擎初始化: 端口={}, KAD={}", config.port, config.kad_enabled);
        let kad_engine = if config.kad_enabled {
            // 生成随机 KAD 节点 ID
            let mut id_bytes = [0u8; 16];
            for b in &mut id_bytes {
                *b = rand_byte();
            }
            Some(kad::KadEngine::new(kad::KadId(id_bytes)))
        } else {
            None
        };
        Ed2kEngine { config, kad_engine }
    }

    /// 更新自定义服务器列表
    pub fn set_custom_servers(&mut self, servers: Vec<String>) {
        self.config.custom_servers = servers;
    }

    /// 执行 ed2k 链接下载
    ///
    /// 完整流程：
    /// 1. 解析 ed2k hash
    /// 2. 连接 ed2k 服务器
    /// 3. 发送登录请求获取客户端 ID
    /// 4. 查找文件源（通过服务器）
    /// 5. 连接源并请求下载
    /// 6. 分块下载并校验
    pub async fn download(
        &mut self,
        task_id: &str,
        link: &Ed2kLink,
        save_path: &str,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
        cancel_token: CancellationToken,
        mut pause_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        info!("ed2k 下载开始: {} ({} bytes)", link.filename, link.size);

        let file_hash = hex_to_hash(&link.hash)?;

        // 空文件直接完成
        if link.size == 0 {
            let file_path = std::path::Path::new(save_path).join(&link.filename);
            tokio::fs::write(&file_path, b"").await?;
            let _ = update_tx.send(TaskUpdateEvent {
                task_id: task_id.to_string(),
                state: TaskState::Done,
                downloaded: 0,
                total_size: 0,
                download_speed: 0,
                upload_speed: 0,
                progress: 1.0,
                peers: 0,
                error: None,
                eta: Some(0),
            });
            return Ok(());
        }

        // 发送"正在连接服务器"状态
        let _ = update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Downloading,
            downloaded: 0,
            total_size: link.size,
            download_speed: 0,
            upload_speed: 0,
            progress: 0.0,
            peers: 0,
            error: None,
            eta: None,
        });

        // 尝试连接服务器列表（优先使用链接指定的服务器，其次配置的自定义服务器，最后默认服务器）
        let servers = if let Some(ref srv) = link.server {
            vec![srv.clone()]
        } else if !self.config.custom_servers.is_empty() {
            self.config.custom_servers.clone()
        } else {
            DEFAULT_SERVERS
                .iter()
                .map(|(ip, port)| format!("{}:{}", ip, port))
                .collect()
        };

        let mut server_conn = None;
        for srv_addr in &servers {
            if cancel_token.is_cancelled() {
                return Ok(());
            }
            let addr: SocketAddr = match srv_addr.parse() {
                Ok(a) => a,
                Err(_) => continue,
            };
            let mut conn = ServerConnection::new(addr, &self.config.username, self.config.port);
            match tokio::time::timeout(
                std::time::Duration::from_secs(self.config.connect_timeout),
                conn.connect(),
            )
            .await
            {
                Ok(Ok(())) => {
                    info!("已连接 ed2k 服务器: {}", addr);
                    server_conn = Some(conn);
                    break;
                }
                Ok(Err(e)) => {
                    warn!("连接 ed2k 服务器 {} 失败: {}", addr, e);
                }
                Err(_) => {
                    warn!("连接 ed2k 服务器 {} 超时", addr);
                }
            }
        }

        let mut server_conn = match server_conn {
            Some(conn) => conn,
            None => {
                anyhow::bail!("无法连接到任何 ed2k 服务器");
            }
        };

        // 通过服务器查找文件源
        info!("查找文件源: {}", link.hash);
        server_conn.get_sources(&file_hash, link.size).await?;

        // 等待源响应（带超时）
        let sources = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            self.wait_for_sources(&mut server_conn, &file_hash),
        )
        .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                warn!("获取源失败: {}", e);
                Vec::new()
            }
            Err(_) => {
                warn!("等待源响应超时");
                Vec::new()
            }
        };

        if sources.is_empty() {
            anyhow::bail!("未找到文件源，请稍后重试");
        }

        info!("找到 {} 个源", sources.len());

        // 通过 KAD 网络补充源（如果启用）
        let mut all_sources = sources;
        if let Some(ref mut kad) = self.kad_engine {
            info!("尝试通过 KAD 网络查找源...");
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                kad.search_source(&file_hash),
            )
            .await
            {
                Ok(kad_sources) => {
                    if !kad_sources.is_empty() {
                        info!("KAD 找到 {} 个额外源", kad_sources.len());
                        for src in kad_sources {
                            if !all_sources.contains(&src) {
                                all_sources.push(src);
                            }
                        }
                    }
                }
                Err(_) => {
                    debug!("KAD 搜索源超时");
                }
            }
        }

        // 限制源数量
        let sources: Vec<SocketAddr> = all_sources.into_iter().take(self.config.max_sources).collect();

        let _ = update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Downloading,
            downloaded: 0,
            total_size: link.size,
            download_speed: 0,
            upload_speed: 0,
            progress: 0.0,
            peers: sources.len() as u32,
            error: None,
            eta: None,
        });

        // 使用第一个可用源进行下载
        let file_path = std::path::Path::new(save_path).join(&link.filename);
        let mut downloaded: u64 = 0;
        let mut last_speed_update = std::time::Instant::now();
        let mut speed_samples: Vec<u64> = Vec::new();
        let mut connected_peers = 0u32;
        // 已完成分片追踪（防止跨源重复计数）
        let mut completed_pieces: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (src_idx, src_addr) in sources.iter().enumerate() {
            if cancel_token.is_cancelled() {
                return Ok(());
            }

            // 检查暂停状态
            if *pause_rx.borrow() {
                while *pause_rx.borrow() {
                    if cancel_token.is_cancelled() {
                        return Ok(());
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }

            let mut transfer = Ed2kTransfer::new(file_hash, link.size);

            // 连接到源
            let connect_result = tokio::time::timeout(
                std::time::Duration::from_secs(self.config.connect_timeout),
                transfer.connect(*src_addr),
            )
            .await;

            match connect_result {
                Ok(Ok(())) => {
                    connected_peers += 1;
                    debug!("已连接到源 {}: {}", src_idx, src_addr);
                }
                Ok(Err(e)) => {
                    warn!("连接源 {} 失败: {}", src_addr, e);
                    continue;
                }
                Err(_) => {
                    warn!("连接源 {} 超时", src_addr);
                    continue;
                }
            }

            // 请求文件名和 hashset
            if let Err(e) = transfer.request_filename().await {
                warn!("请求文件名失败: {}", e);
                continue;
            }

            if let Err(e) = transfer.request_hashset().await {
                warn!("请求 hashset 失败: {}", e);
            }

            // 请求上传许可
            if let Err(e) = transfer.request_upload().await {
                warn!("请求上传许可失败: {}", e);
                continue;
            }

            // 逐块下载
            let num_pieces = (link.size as usize + ED2K_CHUNK_SIZE - 1) / ED2K_CHUNK_SIZE;
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(false)
                .open(&file_path)
                .await?;

            for piece_idx in 0..num_pieces {
                // 跳过已完成的分片（跨源恢复）
                if completed_pieces.contains(&piece_idx) {
                    continue;
                }

                if cancel_token.is_cancelled() {
                    transfer.disconnect().await;
                    return Ok(());
                }

                // 暂停检查
                if *pause_rx.borrow() {
                    while *pause_rx.borrow() {
                        if cancel_token.is_cancelled() {
                            return Ok(());
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }

                let piece_start = (piece_idx as u64) * (ED2K_CHUNK_SIZE as u64);

                if let Err(e) = transfer.request_parts(piece_idx).await {
                    warn!("请求分片 {} 失败: {}", piece_idx, e);
                    break;
                }

                // 等待数据接收（带超时）
                let (recv_offset, piece_data) = match tokio::time::timeout(
                    std::time::Duration::from_secs(30),
                    self.receive_piece(&mut transfer, piece_idx),
                )
                .await
                {
                    Ok(Ok((offset, data))) => (offset, data),
                    Ok(Err(e)) => {
                        warn!("接收分片 {} 失败: {}", piece_idx, e);
                        break;
                    }
                    Err(_) => {
                        warn!("接收分片 {} 超时", piece_idx);
                        break;
                    }
                };

                // 使用服务器返回的偏移量写入（而非假设的 piece_start）
                let write_offset = if recv_offset > 0 { recv_offset } else { piece_start };

                // 校验并写入文件
                if let Err(e) = transfer.handle_received_part(write_offset, &piece_data) {
                    warn!("分片 {} 校验失败: {}", piece_idx, e);
                    continue;
                }

                tokio::io::AsyncSeekExt::seek(&mut file, tokio::io::SeekFrom::Start(write_offset)).await?;
                tokio::io::AsyncWriteExt::write_all(&mut file, &piece_data).await?;
                // 标记分片完成并累加字节（防止跨源重复计数）
                completed_pieces.insert(piece_idx);
                downloaded += piece_data.len() as u64;

                // 计算速度
                speed_samples.push(piece_data.len() as u64);
                let elapsed = last_speed_update.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    let total: u64 = speed_samples.iter().sum();
                    (total as f64 / elapsed) as u64
                } else {
                    0
                };

                // 每秒更新一次速度采样
                if elapsed >= 1.0 {
                    speed_samples.clear();
                    last_speed_update = std::time::Instant::now();
                }

                let progress = if link.size > 0 {
                    downloaded as f32 / link.size as f32
                } else {
                    1.0
                };

                let eta = if speed > 0 {
                    Some((link.size - downloaded) / speed)
                } else {
                    None
                };

                let _ = update_tx.send(TaskUpdateEvent {
                    task_id: task_id.to_string(),
                    state: TaskState::Downloading,
                    downloaded,
                    total_size: link.size,
                    download_speed: speed,
                    upload_speed: 0,
                    progress,
                    peers: connected_peers,
                    error: None,
                    eta,
                });

                debug!(
                    "分片 {}/{} 下载完成 (进度: {:.1}%)",
                    piece_idx + 1,
                    num_pieces,
                    progress * 100.0
                );
            }

            transfer.disconnect().await;

            // 检查是否下载完成
            if downloaded >= link.size {
                info!("ed2k 下载完成: {}", link.filename);
                let _ = update_tx.send(TaskUpdateEvent {
                    task_id: task_id.to_string(),
                    state: TaskState::Done,
                    downloaded: link.size,
                    total_size: link.size,
                    download_speed: 0,
                    upload_speed: 0,
                    progress: 1.0,
                    peers: connected_peers,
                    error: None,
                    eta: Some(0),
                });
                server_conn.disconnect().await;
                return Ok(());
            }
        }

        server_conn.disconnect().await;

        if downloaded < link.size {
            anyhow::bail!(
                "ed2k 下载不完整: {}/{} bytes ({:.1}%)",
                downloaded,
                link.size,
                (downloaded as f64 / link.size as f64) * 100.0
            );
        }

        Ok(())
    }

    /// 等待服务器返回文件源
    async fn wait_for_sources(
        &self,
        conn: &mut ServerConnection,
        _file_hash: &[u8; 16],
    ) -> Result<Vec<SocketAddr>> {
        // 实际实现需要读取服务器响应包并解析 OP_FOUNDSOURCES
        // 当前版本使用轮询接收
        let mut sources = Vec::new();

        // 尝试读取几个响应包
        for _ in 0..5 {
            match tokio::time::timeout(
                std::time::Duration::from_secs(2),
                conn.recv_packet(),
            )
            .await
            {
                Ok(Ok(packet)) => {
                    match packet.header.opcode {
                        proto::OP_FOUNDSOURCES => {
                            // 解析源地址列表
                            // 格式: 文件hash(16) + 源数量(1) + 每个源: IP(4) + Port(2)
                            if packet.payload.len() >= 17 {
                                let hash_end = 16;
                                let count = packet.payload[hash_end] as usize;
                                let mut offset = hash_end + 1;
                                for _ in 0..count {
                                    if offset + 6 > packet.payload.len() {
                                        break;
                                    }
                                    let ip = std::net::Ipv4Addr::new(
                                        packet.payload[offset],
                                        packet.payload[offset + 1],
                                        packet.payload[offset + 2],
                                        packet.payload[offset + 3],
                                    );
                                    let port = u16::from_le_bytes([
                                        packet.payload[offset + 4],
                                        packet.payload[offset + 5],
                                    ]);
                                    sources.push(SocketAddr::new(ip.into(), port));
                                    offset += 6;
                                }
                            }
                        }
                        _ => {
                            debug!("收到其他服务器响应: 0x{:02X}", packet.header.opcode);
                        }
                    }
                }
                _ => break,
            }
        }

        Ok(sources)
    }

    /// 接收一个分片的数据
    /// 使用循环替代递归处理 OP_QUEUERANK，防止栈溢出
    async fn receive_piece(
        &self,
        transfer: &mut Ed2kTransfer,
        piece_index: usize,
    ) -> Result<(u64, Vec<u8>)> {
        const MAX_QUEUE_RETRIES: u32 = 60; // 最多等待 5 分钟（60 * 5 秒）
        let mut queue_retries = 0u32;

        loop {
            let stream = transfer.stream_mut()?;

            // 读取响应包
            let mut header_buf = [0u8; proto::HEADER_SIZE];
            tokio::io::AsyncReadExt::read_exact(stream, &mut header_buf).await?;

            let header = proto::PacketHeader::from_bytes(&header_buf)
                .ok_or_else(|| anyhow::anyhow!("无效的包头"))?;

            let payload_size = header.payload_size();
            let mut payload = vec![0u8; payload_size];
            tokio::io::AsyncReadExt::read_exact(stream, &mut payload).await?;

            match header.opcode {
                proto::OP_SENDINGPART => {
                    // 格式: 文件hash(16) + 开始偏移(8) + 结束偏移(8) + 数据
                    if payload.len() >= 32 {
                        // 解析偏移量，确保数据写入正确位置
                        let start_offset = u64::from_le_bytes([
                            payload[16], payload[17], payload[18], payload[19],
                            payload[20], payload[21], payload[22], payload[23],
                        ]);
                        let data = payload[32..].to_vec();
                        return Ok((start_offset, data));
                    } else {
                        anyhow::bail!("SENDINGPART 包太短")
                    }
                }
                proto::OP_QUEUERANK => {
                    queue_retries += 1;
                    if queue_retries > MAX_QUEUE_RETRIES {
                        anyhow::bail!(
                            "队列等待超时：已重试 {} 次（分片 {}）",
                            queue_retries,
                            piece_index
                        );
                    }
                    if payload.len() >= 4 {
                        let rank = u32::from_le_bytes([
                            payload[0],
                            payload[1],
                            payload[2],
                            payload[3],
                        ]);
                        transfer.handle_queue_rank(rank);
                        debug!(
                            "队列等待: rank={}, 重试 {}/{}",
                            rank, queue_retries, MAX_QUEUE_RETRIES
                        );
                        // 队列等待后继续循环重试
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    } else {
                        anyhow::bail!("QUEUERANK 包太短")
                    }
                }
                _ => {
                    anyhow::bail!("未预期的操作码: 0x{:02X}", header.opcode)
                }
            }
        }
    }
}

/// 生成随机字节（使用系统时间 + 线程局部状态，用于 KAD 节点 ID）
fn rand_byte() -> u8 {
    use std::cell::RefCell;
    use std::time::{SystemTime, UNIX_EPOCH};

    thread_local! {
        static RNG_STATE: RefCell<u64> = RefCell::new(0);
    }

    RNG_STATE.with(|state| {
        let mut s = state.borrow_mut();
        if *s == 0 {
            // 初始化种子：系统时间纳秒
            *s = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
        }
        // xorshift64 算法
        *s ^= *s << 13;
        *s ^= *s >> 7;
        *s ^= *s << 17;
        (*s & 0xFF) as u8
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ed2k_link() {
        let link = "ed2k://|file|example.txt|12345|abcdef1234567890abcdef1234567890|/";
        let result = parse_ed2k_link(link).unwrap();
        assert_eq!(result.filename, "example.txt");
        assert_eq!(result.size, 12345);
        assert_eq!(result.hash, "abcdef1234567890abcdef1234567890");
        assert!(result.server.is_none());
    }

    #[test]
    fn test_parse_ed2k_link_with_server() {
        let link = "ed2k://|file|test.iso|999999|abcdef1234567890abcdef1234567890|/|server,192.168.1.1:4661|/";
        let result = parse_ed2k_link(link).unwrap();
        assert_eq!(result.filename, "test.iso");
        assert_eq!(result.size, 999999);
        assert_eq!(result.server, Some("192.168.1.1:4661".to_string()));
    }

    #[test]
    fn test_parse_invalid_link() {
        assert!(parse_ed2k_link("http://example.com").is_err());
        assert!(parse_ed2k_link("ed2k://|file|test").is_err());
    }

    #[test]
    fn test_parse_ed2k_link_hash_too_short() {
        assert!(parse_ed2k_link("ed2k://|file|a.txt|100|abc|/").is_err());
    }

    #[test]
    fn test_hex_to_hash() {
        let hash = hex_to_hash("abcdef1234567890abcdef1234567890").unwrap();
        assert_eq!(hash.len(), 16);
        assert_eq!(hash[0], 0xab);
        assert_eq!(hash[15], 0x90);
    }

    #[test]
    fn test_hex_to_hash_invalid() {
        assert!(hex_to_hash("xyz").is_err());
        assert!(hex_to_hash("abcdef1234567890abcdef123456789").is_err());
    }

    #[test]
    fn test_ed2k_config_default() {
        let config = Ed2kConfig::default();
        assert_eq!(config.username, "Downloader");
        assert_eq!(config.port, 4661);
        assert!(config.kad_enabled);
        assert_eq!(config.connect_timeout, 15);
        assert_eq!(config.max_sources, 50);
    }
}
