// HLS/DASH 流媒体下载引擎
// 解析 m3u8/mpd 播放列表，下载 ts 分片，合并为完整文件
// 支持分辨率选择、直播录制、DRM 检测、AES-128 解密、并行下载

pub mod m3u8;
pub mod merge;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, watch, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};

use crate::engine::task_manager::TaskUpdateEvent;
use crate::storage::db::TaskState;

/// HLS 加密密钥缓存（避免重复下载同一密钥）
#[derive(Clone)]
struct KeyCache {
    keys: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl KeyCache {
    fn new() -> Self {
        KeyCache {
            keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取密钥（如果缓存中没有则下载）
    async fn get_key(&self, client: &reqwest::Client, key_url: &str) -> Result<Vec<u8>> {
        let mut keys = self.keys.lock().await;
        if let Some(key) = keys.get(key_url) {
            return Ok(key.clone());
        }

        // 下载密钥
        let response = client.get(key_url).send().await?;
        let key_bytes = response.bytes().await?.to_vec();
        keys.insert(key_url.to_string(), key_bytes.clone());
        Ok(key_bytes)
    }
}

/// 流媒体类型
#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    /// HLS (HTTP Live Streaming)
    Hls,
    /// DASH (Dynamic Adaptive Streaming)
    Dash,
}

/// 流媒体质量/分辨率
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamQuality {
    /// 分辨率标签（如 "720p", "1080p"）
    pub label: String,
    /// 带宽（bps）
    pub bandwidth: u64,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 播放列表 URL
    pub url: String,
}

/// 分片信息
#[derive(Debug, Clone)]
pub struct Segment {
    /// 分片 URL
    pub url: String,
    /// 分片序号
    pub index: u32,
    /// 分片时长（秒）
    pub duration: f64,
    /// 是否加密
    pub encrypted: bool,
    /// 加密密钥 URL（如果有）
    pub key_url: Option<String>,
    /// 加密 IV（如果有）
    pub key_iv: Option<Vec<u8>>,
}

/// 播放列表信息
#[derive(Debug, Clone)]
pub struct PlaylistInfo {
    /// 流媒体类型
    pub stream_type: StreamType,
    /// 是否为直播流
    pub is_live: bool,
    /// 总时长（秒，点播）
    pub total_duration: f64,
    /// 可用分辨率列表
    pub qualities: Vec<StreamQuality>,
    /// 分片列表（当为单码率时）
    pub segments: Vec<Segment>,
}

/// 流媒体下载引擎
#[derive(Clone)]
pub struct HlsEngine {
    client: reqwest::Client,
    key_cache: KeyCache,
}

impl HlsEngine {
    /// 创建新的 HLS 引擎
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        HlsEngine {
            client,
            key_cache: KeyCache::new(),
        }
    }

    /// 解析播放列表（支持 HLS m3u8 和 DASH MPD）
    pub async fn parse_playlist(&self, url: &str) -> Result<PlaylistInfo> {
        let content = self.fetch_text(url).await?;

        // 检测 DASH MPD 格式
        if url.ends_with(".mpd") || content.contains("<MPD") {
            let (qualities, segments) = m3u8::parse_mpd(&content, url)?;
            let total_duration: f64 = segments.iter().map(|s| s.duration).sum();
            return Ok(PlaylistInfo {
                stream_type: StreamType::Dash,
                is_live: false,
                total_duration,
                qualities,
                segments,
            });
        }

        if content.contains("#EXT-X-STREAM-INF") {
            // Master playlist：包含多个码率
            let qualities = m3u8::parse_master_playlist(&content, url)?;
            Ok(PlaylistInfo {
                stream_type: StreamType::Hls,
                is_live: false,
                total_duration: 0.0,
                qualities,
                segments: Vec::new(),
            })
        } else {
            // Media playlist：包含分片列表
            let segments = m3u8::parse_media_playlist(&content, url)?;
            let is_live = content.contains("#EXT-X-ENDLIST") == false;
            let total_duration: f64 = segments.iter().map(|s| s.duration).sum();

            Ok(PlaylistInfo {
                stream_type: StreamType::Hls,
                is_live,
                total_duration,
                qualities: Vec::new(),
                segments,
            })
        }
    }

    /// 下载 HLS 流
    pub async fn download(
        &self,
        task_id: &str,
        playlist_url: &str,
        save_path: &str,
        file_name: &str,
        quality_label: Option<&str>,
        update_tx: mpsc::UnboundedSender<TaskUpdateEvent>,
        cancel_token: CancellationToken,
        pause_rx: watch::Receiver<bool>,
    ) -> Result<()> {
        info!("HLS 下载开始: {}", playlist_url);

        // 1. 解析播放列表
        let mut playlist = self.parse_playlist(playlist_url).await?;

        // 2. 如果是 master playlist，选择指定分辨率
        if !playlist.qualities.is_empty() {
            let quality = self.select_quality(&playlist.qualities, quality_label);
            info!("选择分辨率: {} ({}bps)", quality.label, quality.bandwidth);

            // 解析子播放列表
            let sub_playlist = self.parse_playlist(&quality.url).await?;
            playlist.segments = sub_playlist.segments;
            playlist.is_live = sub_playlist.is_live;
        }

        if playlist.segments.is_empty() {
            bail!("播放列表中没有分片");
        }

        info!("共 {} 个分片, 直播: {}", playlist.segments.len(), playlist.is_live);

        // 3. 创建临时目录
        let temp_dir = Path::new(save_path).join(format!(".hls_temp_{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&temp_dir).await?;

        // 4. 并行下载所有分片（最多 8 个并发）
        let total_segments = playlist.segments.len() as u64;
        let downloaded_segments = Arc::new(std::sync::atomic::AtomicU64::new(0));
        let segment_files = Arc::new(Mutex::new(Vec::new()));
        let max_concurrent = 8usize;

        // 分批并行下载
        for chunk in playlist.segments.chunks(max_concurrent) {
            // 检查取消
            if cancel_token.is_cancelled() {
                info!("HLS 下载被取消");
                let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                return Ok(());
            }

            // 检查暂停
            let mut pause_rx_c = pause_rx.clone();
            while *pause_rx_c.borrow() {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                        return Ok(());
                    },
                    _ = pause_rx_c.changed() => break,
                }
            }

            // 并行下载当前批次
            let mut handles = Vec::new();
            for (i, segment) in chunk.iter().enumerate() {
                let client = self.client.clone();
                let key_cache = self.key_cache.clone();
                let segment = segment.clone();
                let temp_dir = temp_dir.clone();
                let cancel_c = cancel_token.clone();
                let downloaded_c = downloaded_segments.clone();
                let segment_files_c = segment_files.clone();
                let update_c = update_tx.clone();
                let tid = task_id.to_string();

                handles.push(tokio::spawn(async move {
                    // 检查取消
                    if cancel_c.is_cancelled() {
                        return Ok(());
                    }

                    let segment_file = temp_dir.join(format!("seg_{:06}.ts", segment.index));

                    // 下载分片
                    let data = client.get(&segment.url).send().await?.bytes().await?;

                    // 如果分片加密，进行解密
                    let final_data = if segment.encrypted {
                        if let Some(ref key_url) = segment.key_url {
                            let key = key_cache.get_key(&client, key_url).await?;
                            let iv = segment.key_iv.clone().unwrap_or_else(|| {
                                // 默认 IV 为分片序号的 big-endian 16 字节表示
                                let mut iv = vec![0u8; 16];
                                let seq_bytes = segment.index.to_be_bytes();
                                iv[12..16].copy_from_slice(&seq_bytes);
                                iv
                            });
                            decrypt_aes_128_cbc(&data, &key, &iv)?
                        } else {
                            data.to_vec()
                        }
                    } else {
                        data.to_vec()
                    };

                    tokio::fs::write(&segment_file, &final_data).await?;

                    let count = downloaded_c.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    segment_files_c.lock().await.push(segment_file);

                    // 推送进度
                    let progress = count as f32 / total_segments as f32;
                    let _ = update_c.send(TaskUpdateEvent {
                        task_id: tid,
                        state: TaskState::Downloading,
                        downloaded: count,
                        total_size: total_segments,
                        download_speed: 0,
                        upload_speed: 0,
                        progress,
                        peers: 0,
                        error: None,
                        eta: None,
                    });

                    debug!("分片 {}/{} 下载完成", count, total_segments);
                    Ok::<(), anyhow::Error>(())
                }));
            }

            // 等待当前批次完成
            for handle in handles {
                match handle.await {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        warn!("分片下载失败: {}", e);
                        if !playlist.is_live {
                            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                            bail!("分片下载失败: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("分片任务异常: {}", e);
                        if !playlist.is_live {
                            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                            bail!("分片任务异常: {}", e);
                        }
                    }
                }
            }
        }

        // 5. 排序并合并分片为完整文件
        let output_path = Path::new(save_path).join(file_name);
        let mut sorted_files = segment_files.lock().await.clone();
        sorted_files.sort(); // 按文件名排序确保顺序正确
        info!("合并 {} 个分片到: {}", sorted_files.len(), output_path.display());

        merge::merge_ts_files(&sorted_files, &output_path).await?;

        // 6. 清理临时文件
        if let Err(e) = tokio::fs::remove_dir_all(&temp_dir).await {
            warn!("清理临时目录失败: {}", e);
        }

        // 推送完成状态
        let _ = update_tx.send(TaskUpdateEvent {
            task_id: task_id.to_string(),
            state: TaskState::Done,
            downloaded: total_segments,
            total_size: total_segments,
            download_speed: 0,
            upload_speed: 0,
            progress: 1.0,
            peers: 0,
            error: None,
            eta: None,
        });

        info!("HLS 下载完成: {}", file_name);
        Ok(())
    }

    /// 选择视频质量
    fn select_quality<'a>(
        &self,
        qualities: &'a [StreamQuality],
        preferred: Option<&str>,
    ) -> &'a StreamQuality {
        // 如果指定了分辨率，尝试匹配
        if let Some(label) = preferred {
            if let Some(q) = qualities.iter().find(|q| q.label == label) {
                return q;
            }
        }

        // 默认选择最高分辨率
        qualities
            .iter()
            .max_by_key(|q| q.bandwidth)
            .unwrap_or(&qualities[0])
    }

    /// 下载单个分片
    async fn download_segment(&self, url: &str, save_path: &Path) -> Result<()> {
        let response = self.client.get(url).send().await?;
        let bytes = response.bytes().await?;
        tokio::fs::write(save_path, bytes).await?;
        Ok(())
    }

    /// 获取 URL 内容
    async fn fetch_text(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        let text = response.text().await?;
        Ok(text)
    }
}

/// AES-128-CBC 解密
/// HLS 使用 AES-128-CBC 加密分片，密钥从 EXT-X-KEY 标签获取
fn decrypt_aes_128_cbc(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>> {
    use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};

    type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

    if key.len() != 16 {
        bail!("AES-128 密钥长度必须为 16 字节");
    }
    if iv.len() != 16 {
        bail!("AES-128 IV 长度必须为 16 字节");
    }

    // 确保数据长度是 16 的倍数
    if data.len() % 16 != 0 {
        bail!("加密数据长度不是 16 的倍数");
    }

    // 使用 AES-128-CBC 解密
    let mut buf = data.to_vec();
    let decryptor = Aes128CbcDec::new_from_slices(key, iv)
        .map_err(|e| anyhow::anyhow!("创建 AES 解密器失败: {}", e))?;

    let decrypted = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| anyhow::anyhow!("AES 解密失败: {}", e))?;

    Ok(decrypted.to_vec())
}
