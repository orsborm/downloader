// ed2k 服务器通信模块
// 实现 TCP 连接、登录、搜索、源查找等服务器交互

use anyhow::{bail, Result};
use bytes::{Buf, BytesMut};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

use super::proto::*;
use super::tag::{parse_tag_list, TagName, TagValue};

/// ed2k 服务器信息
#[derive(Debug, Clone)]
pub struct ServerInfo {
    /// 服务器地址
    pub addr: SocketAddr,
    /// 服务器名称
    pub name: String,
    /// 服务器描述
    pub description: String,
    /// 当前用户数
    pub users: u32,
    /// 当前文件数
    pub files: u32,
    /// 连接状态
    pub connected: bool,
    /// 延迟（毫秒）
    pub ping: u32,
}

/// 搜索结果条目
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 文件 hash (16 bytes)
    pub hash: [u8; 16],
    /// 文件 ID
    pub file_id: u32,
    /// 文件名
    pub filename: String,
    /// 文件大小（字节）
    pub size: u64,
    /// 可用源数量
    pub sources: u32,
}

/// 客户端 ID 类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ClientId {
    /// HighID：可直接连接（有公网 IP）
    High(u32),
    /// LowID：需通过服务器中转（NAT 后面）
    Low(u32),
}

/// ed2k 服务器连接
pub struct ServerConnection {
    /// TCP 连接
    stream: Option<TcpStream>,
    /// 服务器地址
    addr: SocketAddr,
    /// 客户端 ID
    client_id: ClientId,
    /// 客户端用户名
    username: String,
    /// 客户端端口
    port: u16,
    /// 服务器信息（连接后填充）
    server_info: Option<ServerInfo>,
}

impl ServerConnection {
    /// 创建新的服务器连接
    pub fn new(addr: SocketAddr, username: &str, port: u16) -> Self {
        ServerConnection {
            stream: None,
            addr,
            client_id: ClientId::Low(0),
            username: username.to_string(),
            port,
            server_info: None,
        }
    }

    /// 连接到服务器并登录
    pub async fn connect(&mut self) -> Result<()> {
        info!("连接 ed2k 服务器: {}", self.addr);

        let stream = TcpStream::connect(self.addr).await?;
        self.stream = Some(stream);

        // 发送登录请求
        self.send_login().await?;

        // 等待服务器响应
        let response = self.recv_packet().await?;

        match response.header.opcode {
            OP_SERVERIDENT => {
                info!("服务器识别成功");
                self.handle_server_ident(&response)?;
            }
            OP_IDCHANGE => {
                self.handle_id_change(&response)?;
            }
            _ => {
                warn!("收到未知响应: 0x{:02X}", response.header.opcode);
            }
        }

        Ok(())
    }

    /// 发送登录请求
    async fn send_login(&mut self) -> Result<()> {
        let client_id = match self.client_id {
            ClientId::High(id) => id,
            ClientId::Low(id) => id,
        };

        let packet = build_login_request(client_id, self.port, &self.username);
        self.send_packet(&packet).await
    }

    /// 发送搜索请求
    pub async fn search(&mut self, query: &str) -> Result<()> {
        info!("发送搜索请求: {}", query);
        let packet = build_search_request(query);
        self.send_packet(&packet).await
    }

    /// 发送源查找请求
    pub async fn get_sources(&mut self, hash: &[u8; 16], size: u64) -> Result<()> {
        debug!("查找文件源: {:02x?}", hash);
        let packet = build_get_sources(hash, size);
        self.send_packet(&packet).await
    }

    /// 处理服务器标识响应
    /// OP_SERVERIDENT (0x41): 16字节服务器hash + tag列表
    fn handle_server_ident(&mut self, packet: &Packet) -> Result<()> {
        if packet.payload.len() < 16 {
            bail!("服务器标识包太短");
        }

        // 解析服务器 hash（16 字节）
        let mut server_hash = [0u8; 16];
        server_hash.copy_from_slice(&packet.payload[..16]);

        // 解析 tag 列表（跳过前 16 字节）
        let mut tag_buf = BytesMut::from(&packet.payload[16..]);
        let tags = parse_tag_list(&mut tag_buf);

        let mut info = ServerInfo {
            addr: self.addr,
            name: String::new(),
            description: String::new(),
            users: 0,
            files: 0,
            connected: true,
            ping: 0,
        };

        for tag in tags {
            match &tag.name {
                TagName::Id(id) => {
                    match *id {
                        0x01 => { // 服务器名称
                            if let TagValue::String(ref s) = tag.value {
                                info.name = s.clone();
                            }
                        }
                        0x0B => { // 服务器描述
                            if let TagValue::String(ref s) = tag.value {
                                info.description = s.clone();
                            }
                        }
                        0x0C => { // 当前用户数
                            if let TagValue::UInt32(v) = tag.value {
                                info.users = v;
                            }
                        }
                        0x0D => { // 当前文件数
                            if let TagValue::UInt32(v) = tag.value {
                                info.files = v;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        info!("服务器标识: {} (用户: {}, 文件: {})", info.name, info.users, info.files);
        self.server_info = Some(info);
        Ok(())
    }

    /// 处理 ID 变更响应
    fn handle_id_change(&mut self, packet: &Packet) -> Result<()> {
        if packet.payload.len() < 4 {
            bail!("ID 变更包太短");
        }

        let new_id = u32::from_le_bytes([
            packet.payload[0],
            packet.payload[1],
            packet.payload[2],
            packet.payload[3],
        ]);

        // HighID 通常 > 16777216 (0x01000000)
        if new_id > 0x01000000 {
            self.client_id = ClientId::High(new_id);
            info!("获得 HighID: {}", new_id);
        } else {
            self.client_id = ClientId::Low(new_id);
            info!("获得 LowID: {}", new_id);
        }

        Ok(())
    }

    /// 发送数据包
    async fn send_packet(&mut self, packet: &Packet) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;
        let data = packet.to_bytes();
        stream.write_all(&data).await?;
        stream.flush().await?;
        Ok(())
    }

    /// 接收数据包
    pub async fn recv_packet(&mut self) -> Result<Packet> {
        let stream = self.stream.as_mut().ok_or_else(|| anyhow::anyhow!("未连接"))?;

        // 读取包头（6 字节）
        let mut header_buf = [0u8; HEADER_SIZE];
        stream.read_exact(&mut header_buf).await?;

        let header = PacketHeader::from_bytes(&header_buf)
            .ok_or_else(|| anyhow::anyhow!("无效的包头"))?;

        // 读取 payload
        let payload_size = header.payload_size();
        let mut payload = bytes::BytesMut::zeroed(payload_size);
        if payload_size > 0 {
            stream.read_exact(&mut payload).await?;
        }

        Ok(Packet { header, payload })
    }

    /// 断开连接
    pub async fn disconnect(&mut self) {
        if let Some(mut stream) = self.stream.take() {
            stream.shutdown().await.ok();
        }
        info!("已断开 ed2k 服务器连接");
    }

    /// 获取客户端 ID
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// 接收搜索结果
    /// OP_SEARCHRESULT (0x33): 文件数量 + 文件列表
    pub async fn recv_search_results(&mut self) -> Result<Vec<SearchResult>> {
        let packet = self.recv_packet().await?;
        if packet.header.opcode != 0x33 {
            bail!("期望搜索结果包，收到: 0x{:02x}", packet.header.opcode);
        }

        let mut results = Vec::new();
        let payload = &packet.payload;
        if payload.len() < 4 {
            return Ok(results);
        }

        let count = u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]) as usize;
        let mut offset = 4;

        for _ in 0..count {
            if offset + 16 > payload.len() {
                break;
            }

            // 文件 hash (16 bytes)
            let mut hash = [0u8; 16];
            hash.copy_from_slice(&payload[offset..offset + 16]);
            offset += 16;

            // 解析文件 tag 列表
            let mut file_id = 0u32;
            let mut file_size = 0u64;
            let mut filename = String::new();
            let mut sources = 0u32;

            // tag 数量
            if offset + 4 > payload.len() { break; }
            let tag_count = u32::from_le_bytes([
                payload[offset], payload[offset+1], payload[offset+2], payload[offset+3]
            ]) as usize;
            offset += 4;

            for _ in 0..tag_count {
                if offset + 3 > payload.len() { break; }
                let tag_type = payload[offset];
                offset += 1;
                let name_len = u16::from_le_bytes([payload[offset], payload[offset+1]]) as usize;
                offset += 2;

                if offset + name_len > payload.len() { break; }
                let name_bytes = &payload[offset..offset+name_len];
                offset += name_len;

                match tag_type {
                    0x02 => { // String
                        if offset + 2 > payload.len() { break; }
                        let str_len = u16::from_le_bytes([payload[offset], payload[offset+1]]) as usize;
                        offset += 2;
                        if offset + str_len > payload.len() { break; }
                        if name_len == 1 && name_bytes[0] == 0x01 {
                            filename = String::from_utf8_lossy(&payload[offset..offset+str_len]).to_string();
                        }
                        offset += str_len;
                    }
                    0x03 => { // UInt32
                        if offset + 4 > payload.len() { break; }
                        let val = u32::from_le_bytes([
                            payload[offset], payload[offset+1], payload[offset+2], payload[offset+3]
                        ]);
                        offset += 4;
                        if name_len == 1 {
                            match name_bytes[0] {
                                0x02 => { file_id = val; } // 文件 ID
                                0x04 => { sources = val; } // 源数量
                                _ => {}
                            }
                        }
                    }
                    0x04 => { // UInt64 (文件大小)
                        if offset + 8 > payload.len() { break; }
                        let val = u64::from_le_bytes([
                            payload[offset], payload[offset+1], payload[offset+2], payload[offset+3],
                            payload[offset+4], payload[offset+5], payload[offset+6], payload[offset+7],
                        ]);
                        offset += 8;
                        if name_len == 1 && name_bytes[0] == 0x02 {
                            file_size = val;
                        }
                    }
                    _ => { break; } // 未知 tag 类型，跳过
                }
            }

            results.push(SearchResult {
                hash,
                file_id,
                filename,
                size: file_size,
                sources,
            });
        }

        Ok(results)
    }
}

/// 解析服务器列表文件（server.met）
pub fn parse_server_met(data: &[u8]) -> Vec<ServerInfo> {
    let mut servers = Vec::new();

    if data.len() < 4 {
        return servers;
    }

    // server.met 格式：
    // 4 字节：服务器数量
    // 每个服务器：4字节IP + 2字节端口 + tag列表
    let count = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    let mut offset = 4;

    for _ in 0..count {
        if offset + 6 > data.len() {
            break;
        }

        let ip = std::net::Ipv4Addr::new(
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        );
        let port = u16::from_le_bytes([data[offset + 4], data[offset + 5]]);
        offset += 6;

        servers.push(ServerInfo {
            addr: SocketAddr::new(ip.into(), port),
            name: format!("{}:{}", ip, port),
            description: String::new(),
            users: 0,
            files: 0,
            connected: false,
            ping: 0,
        });
    }

    servers
}
