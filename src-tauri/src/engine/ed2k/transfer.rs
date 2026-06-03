// ed2k 文件传输模块
// 实现客户端握手、文件请求、分片下载、队列管理

use anyhow::{bail, Result};
use std::collections::HashMap;
use tokio::net::TcpStream;
use tracing::{debug, info};

use super::hash::{ed2k_hash, ED2K_CHUNK_SIZE};
use super::proto::*;

/// 传输状态
#[derive(Debug, Clone, PartialEq)]
pub enum TransferState {
    /// 等待连接
    Connecting,
    /// 握手中
    Handshaking,
    /// 请求文件名
    RequestingFilename,
    /// 请求 hashset
    RequestingHashset,
    /// 等待上传许可
    WaitingUpload,
    /// 下载中
    Downloading,
    /// 完成
    Done,
    /// 错误
    Error(String),
}

/// ed2k 文件传输
pub struct Ed2kTransfer {
    /// 目标文件 hash
    file_hash: [u8; 16],
    /// 文件大小
    file_size: u64,
    /// 已下载大小
    downloaded: u64,
    /// 分片 hash 列表（hashset）
    hashset: Vec<[u8; 16]>,
    /// 已下载的分片位图
    piece_bitmap: Vec<bool>,
    /// 传输状态
    state: TransferState,
    /// TCP 连接
    stream: Option<TcpStream>,
    /// 对端客户端信息
    remote_name: String,
}

impl Ed2kTransfer {
    /// 创建新的文件传输
    pub fn new(file_hash: [u8; 16], file_size: u64) -> Self {
        let num_pieces = (file_size as usize + ED2K_CHUNK_SIZE - 1) / ED2K_CHUNK_SIZE;

        Ed2kTransfer {
            file_hash,
            file_size,
            downloaded: 0,
            hashset: Vec::new(),
            piece_bitmap: vec![false; num_pieces],
            state: TransferState::Connecting,
            stream: None,
            remote_name: String::new(),
        }
    }

    /// 连接到对端客户端
    pub async fn connect(&mut self, addr: std::net::SocketAddr) -> Result<()> {
        info!("连接到 ed2k 客户端: {}", addr);
        let stream = TcpStream::connect(addr).await?;
        self.stream = Some(stream);
        self.state = TransferState::Handshaking;

        // 发送客户端问候
        self.send_hello().await?;

        Ok(())
    }

    /// 发送客户端问候
    async fn send_hello(&mut self) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;

        let packet = build_hello(0, 0, "Downloader", &self.file_hash);
        let data = packet.to_bytes();
        tokio::io::AsyncWriteExt::write_all(stream, &data).await?;

        debug!("发送客户端问候");
        Ok(())
    }

    /// 请求文件名
    pub async fn request_filename(&mut self) -> Result<()> {
        self.state = TransferState::RequestingFilename;
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;

        let packet = build_request_filename(&self.file_hash);
        let data = packet.to_bytes();
        tokio::io::AsyncWriteExt::write_all(stream, &data).await?;

        debug!("请求文件名");
        Ok(())
    }

    /// 请求 hashset
    pub async fn request_hashset(&mut self) -> Result<()> {
        self.state = TransferState::RequestingHashset;
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;

        let packet = build_hashset_request(&self.file_hash);
        let data = packet.to_bytes();
        tokio::io::AsyncWriteExt::write_all(stream, &data).await?;

        debug!("请求 hashset");
        Ok(())
    }

    /// 请求下载（发送上传请求）
    pub async fn request_upload(&mut self) -> Result<()> {
        self.state = TransferState::WaitingUpload;
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;

        let packet = Packet::new(PROTO_EDONKEY, OP_STARTUPLOADREQ, bytes::BytesMut::new());
        let data = packet.to_bytes();
        tokio::io::AsyncWriteExt::write_all(stream, &data).await?;

        debug!("发送上传请求");
        Ok(())
    }

    /// 请求数据分片
    pub async fn request_parts(&mut self, piece_index: usize) -> Result<()> {
        if piece_index >= self.piece_bitmap.len() {
            bail!("分片索引越界");
        }

        if self.piece_bitmap[piece_index] {
            debug!("分片 {} 已下载，跳过", piece_index);
            return Ok(());
        }

        let start = (piece_index as u64) * (ED2K_CHUNK_SIZE as u64);
        let end = std::cmp::min(start + ED2K_CHUNK_SIZE as u64, self.file_size);

        // ed2k 请求 3 个分片（如果可用）
        let offsets = [(start, end); 3];

        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;
        let packet = build_request_parts(&self.file_hash, &offsets);
        let data = packet.to_bytes();
        tokio::io::AsyncWriteExt::write_all(stream, &data).await?;

        debug!("请求分片 {}: {} - {}", piece_index, start, end);
        Ok(())
    }

    /// 处理接收到的数据分片
    pub fn handle_received_part(&mut self, offset: u64, data: &[u8]) -> Result<()> {
        let piece_index = (offset / ED2K_CHUNK_SIZE as u64) as usize;

        if piece_index < self.piece_bitmap.len() {
            // 校验分片 hash
            if piece_index < self.hashset.len() {
                let computed_hash = ed2k_hash(data);
                if computed_hash != self.hashset[piece_index] {
                    bail!("分片 {} hash 校验失败", piece_index);
                }
            }

            self.piece_bitmap[piece_index] = true;
            self.downloaded += data.len() as u64;

            debug!(
                "分片 {} 接收完成: {} bytes (总进度: {}/{})",
                piece_index,
                data.len(),
                self.downloaded,
                self.file_size
            );
        }

        Ok(())
    }

    /// 处理 hashset 响应
    pub fn handle_hashset(&mut self, hashset_data: &[u8]) -> Result<()> {
        if hashset_data.len() < 2 {
            bail!("hashset 数据太短");
        }

        let count = u16::from_le_bytes([hashset_data[0], hashset_data[1]]) as usize;
        let mut offset = 2;

        self.hashset.clear();
        for _ in 0..count {
            if offset + 16 > hashset_data.len() {
                break;
            }
            let mut hash = [0u8; 16];
            hash.copy_from_slice(&hashset_data[offset..offset + 16]);
            self.hashset.push(hash);
            offset += 16;
        }

        info!("收到 hashset: {} 个分片", self.hashset.len());
        Ok(())
    }

    /// 处理队列排名
    /// 队列排名越高，等待时间越长
    /// 排名 1 = 最快，排名 >200 = 非常慢
    pub fn handle_queue_rank(&mut self, rank: u32) {
        info!("队列排名: {}", rank);

        if rank == 0 {
            // 排名为 0 表示立即可下载
            self.state = TransferState::Downloading;
            return;
        }

        // 计算等待时间：基础 10 秒 + 每个排名加 5 秒，最大 300 秒
        let wait_secs = (10u64).saturating_add((rank as u64).saturating_mul(5)).min(300);
        info!("队列排名 {}，等待 {} 秒后重试", rank, wait_secs);

        // 保持等待状态，实际重试由上层调度器控制
        self.state = TransferState::WaitingUpload;
    }

    /// 检查是否下载完成
    pub fn is_complete(&self) -> bool {
        self.piece_bitmap.iter().all(|&b| b)
    }

    /// 获取下载进度 (0.0 ~ 1.0)
    pub fn progress(&self) -> f32 {
        if self.file_size == 0 {
            return 1.0;
        }
        self.downloaded as f32 / self.file_size as f32
    }

    /// 获取已下载大小
    pub fn downloaded(&self) -> u64 {
        self.downloaded
    }

    /// 获取传输状态
    pub fn state(&self) -> &TransferState {
        &self.state
    }

    /// 获取 TCP 流的可变引用（用于引擎直接读取数据）
    pub fn stream_mut(&mut self) -> Result<&mut TcpStream> {
        self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            tokio::io::AsyncWriteExt::shutdown(&mut stream).await.ok();
        }
        self.state = TransferState::Done;
    }
}
