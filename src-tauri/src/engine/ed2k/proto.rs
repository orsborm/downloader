// ed2k 协议定义模块
// 定义协议标识、操作码、包结构、序列化/反序列化

use bytes::{Buf, BufMut, BytesMut};

/// 协议标识
pub const PROTO_EDONKEY: u8 = 0xE3;   // 标准 eDonkey 协议
pub const PROTO_EMULE: u8 = 0xC5;     // eMule 扩展协议
pub const PROTO_COMPRESSED: u8 = 0xD4; // 压缩协议 (zlib)
pub const PROTO_KAD: u8 = 0xE4;       // Kademlia DHT
pub const PROTO_KAD_COMPRESSED: u8 = 0xE5; // 压缩 Kademlia

/// TCP 包头长度（6 字节）
pub const HEADER_SIZE: usize = 6;

/// ==================== 服务器通信操作码 (C2S) ====================

/// 登录请求
pub const OP_LOGINREQUEST: u8 = 0x01;
/// 获取服务器列表
pub const OP_GETSERVERLIST: u8 = 0x14;
/// 文件搜索请求
pub const OP_SEARCHREQUEST: u8 = 0x16;
/// 查找文件源
pub const OP_GETSOURCES: u8 = 0x19;

/// ==================== 服务器响应操作码 (S2C) ====================

/// 服务器标识
pub const OP_SERVERIDENT: u8 = 0x41;
/// 客户端 ID 分配
pub const OP_IDCHANGE: u8 = 0x40;
/// 搜索结果
pub const OP_SEARCHRESULT: u8 = 0x33;
/// 找到的源
pub const OP_FOUNDSOURCES: u8 = 0x42;

/// ==================== 客户端通信操作码 (C2C) ====================

/// 客户端问候
pub const OP_HELLO: u8 = 0x01;
/// 客户端问候响应
pub const OP_HELLOANSWER: u8 = 0x4C;
/// 文件名请求
pub const OP_REQUESTFILENAME: u8 = 0x58;
/// 文件名响应
pub const OP_REQFILENAMEANSWER: u8 = 0x59;
/// 设置请求文件
pub const OP_SETREQFILEID: u8 = 0x4F;
/// 请求 hashset
pub const OP_HASHSETREQUEST: u8 = 0x51;
/// hashset 响应
pub const OP_HASHSETANSWER: u8 = 0x52;
/// 上传请求
pub const OP_STARTUPLOADREQ: u8 = 0x54;
/// 接受上传请求
pub const OP_ACCEPTUPLOADREQ: u8 = 0x55;
/// 请求分片（3 个偏移量）
pub const OP_REQUESTPARTS: u8 = 0x47;
/// 发送数据分片
pub const OP_SENDINGPART: u8 = 0x46;
/// 队列排名
pub const OP_QUEUERANK: u8 = 0x5C;

/// ==================== 包结构 ====================

/// ed2k TCP 包头
#[derive(Debug, Clone)]
pub struct PacketHeader {
    /// 协议标识
    pub protocol: u8,
    /// 包大小（不含包头前 5 字节，含操作码）
    pub size: u32,
    /// 操作码
    pub opcode: u8,
}

impl PacketHeader {
    /// 从字节流解析包头
    pub fn from_bytes(buf: &[u8]) -> Option<Self> {
        if buf.len() < HEADER_SIZE {
            return None;
        }

        Some(PacketHeader {
            protocol: buf[0],
            size: u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]),
            opcode: buf[5],
        })
    }

    /// 序列化为字节流
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut header = [0u8; HEADER_SIZE];
        header[0] = self.protocol;
        header[1..5].copy_from_slice(&self.size.to_le_bytes());
        header[5] = self.opcode;
        header
    }

    /// 获取 payload 长度（包大小 - 1 字节操作码）
    pub fn payload_size(&self) -> usize {
        (self.size as usize).saturating_sub(1)
    }
}

/// ed2k 包
#[derive(Debug, Clone)]
pub struct Packet {
    pub header: PacketHeader,
    pub payload: BytesMut,
}

impl Packet {
    /// 创建新包
    pub fn new(protocol: u8, opcode: u8, payload: BytesMut) -> Self {
        Packet {
            header: PacketHeader {
                protocol,
                size: (payload.len() as u32) + 1, // payload + 1 字节操作码
                opcode,
            },
            payload,
        }
    }

    /// 序列化为字节流（包头 + payload）
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(HEADER_SIZE + self.payload.len());
        buf.extend_from_slice(&self.header.to_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
}

/// ==================== 登录包构建 ====================

/// 构建登录请求包
pub fn build_login_request(
    client_id: u32,
    port: u16,
    username: &str,
) -> Packet {
    let mut payload = BytesMut::new();

    // 客户端 ID（4 字节，Little Endian）
    payload.put_u32_le(client_id);
    // 端口（2 字节）
    payload.put_u16_le(port);

    // Tag 列表
    // 用户名 tag
    let username_bytes = username.as_bytes();
    payload.put_u8(0x01); // tag 数量
    put_string_tag(&mut payload, 0x01, username_bytes); // CT_NAME = 0x01

    // 版本信息
    payload.put_u32_le(0x00000001); // 版本

    Packet::new(PROTO_EDONKEY, OP_LOGINREQUEST, payload)
}

/// 构建搜索请求包
pub fn build_search_request(query: &str) -> Packet {
    let mut payload = BytesMut::new();

    // 搜索类型（0x01 = 文件名搜索）
    payload.put_u8(0x01);

    // 搜索字符串 tag
    let query_bytes = query.as_bytes();
    put_string_tag(&mut payload, 0x01, query_bytes); // FT_FILENAME = 0x01

    Packet::new(PROTO_EDONKEY, OP_SEARCHREQUEST, payload)
}

/// 构建源查找请求包
pub fn build_get_sources(hash: &[u8; 16], size: u64) -> Packet {
    let mut payload = BytesMut::with_capacity(24);

    // 文件 hash（16 字节）
    payload.extend_from_slice(hash);
    // 文件大小（8 字节）
    payload.put_u64_le(size);

    Packet::new(PROTO_EDONKEY, OP_GETSOURCES, payload)
}

/// 构建客户端问候包
pub fn build_hello(
    client_id: u32,
    port: u16,
    username: &str,
    hash: &[u8; 16],
) -> Packet {
    let mut payload = BytesMut::new();

    // ed2k hash（16 字节）
    payload.extend_from_slice(hash);
    // 客户端 ID（4 字节）
    payload.put_u32_le(client_id);
    // 端口（2 字节）
    payload.put_u16_le(port);

    // Tag 列表
    payload.put_u8(0x01); // tag 数量
    put_string_tag(&mut payload, 0x01, username.as_bytes());

    Packet::new(PROTO_EDONKEY, OP_HELLO, payload)
}

/// 构建文件请求包
pub fn build_request_filename(hash: &[u8; 16]) -> Packet {
    let mut payload = BytesMut::with_capacity(16);
    payload.extend_from_slice(hash);
    Packet::new(PROTO_EDONKEY, OP_REQUESTFILENAME, payload)
}

/// 构建 hashset 请求包
pub fn build_hashset_request(hash: &[u8; 16]) -> Packet {
    let mut payload = BytesMut::with_capacity(16);
    payload.extend_from_slice(hash);
    Packet::new(PROTO_EDONKEY, OP_HASHSETREQUEST, payload)
}

/// 构建分片请求包（请求 3 个分片）
pub fn build_request_parts(
    hash: &[u8; 16],
    offsets: &[(u64, u64); 3],
) -> Packet {
    let mut payload = BytesMut::with_capacity(16 + 24);

    // 文件 hash
    payload.extend_from_slice(hash);

    // 3 对偏移量（开始位置, 结束位置）
    for (start, end) in offsets {
        payload.put_u64_le(*start);
        payload.put_u64_le(*end);
    }

    Packet::new(PROTO_EDONKEY, OP_REQUESTPARTS, payload)
}

/// ==================== 辅助函数 ====================

/// 写入字符串 tag
fn put_string_tag(buf: &mut BytesMut, tag_type: u8, value: &[u8]) {
    // Tag 头：类型(1字节) + 名称长度(1字节) + 名称 + 值长度(2字节) + 值
    buf.put_u8(0x02); // tag 类型：字符串
    buf.put_u8(1);    // 名称长度
    buf.put_u8(tag_type);
    buf.put_u16_le(value.len() as u16);
    buf.extend_from_slice(value);
}

/// 写入整数 tag
pub fn put_u32_tag(buf: &mut BytesMut, tag_type: u8, value: u32) {
    buf.put_u8(0x03); // tag 类型：整数
    buf.put_u8(1);    // 名称长度
    buf.put_u8(tag_type);
    buf.put_u32_le(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_header() {
        let header = PacketHeader {
            protocol: PROTO_EDONKEY,
            size: 10,
            opcode: OP_LOGINREQUEST,
        };
        let bytes = header.to_bytes();
        let parsed = PacketHeader::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.protocol, PROTO_EDONKEY);
        assert_eq!(parsed.size, 10);
        assert_eq!(parsed.opcode, OP_LOGINREQUEST);
    }

    #[test]
    fn test_login_request() {
        let packet = build_login_request(12345, 4661, "TestUser");
        assert_eq!(packet.header.protocol, PROTO_EDONKEY);
        assert_eq!(packet.header.opcode, OP_LOGINREQUEST);
    }
}
