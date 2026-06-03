// ed2k Tag 系统模块
// Tag 用于编码元数据（文件名、大小、类型等）
// 支持 Hash/String/Integer/Float/Bool 类型

use bytes::{Buf, BufMut, BytesMut};

/// Tag 类型标识
pub const TAG_HASH: u8 = 0x01;    // 16 字节 hash
pub const TAG_STRING: u8 = 0x02;  // 字符串
pub const TAG_UINT32: u8 = 0x03;  // 32 位整数
pub const TAG_FLOAT32: u8 = 0x04; // 32 位浮点
pub const TAG_BOOL: u8 = 0x05;    // 布尔值
pub const TAG_BLOB: u8 = 0x06;    // 二进制数据
pub const TAG_UINT16: u8 = 0x08;  // 16 位整数
pub const TAG_UINT8: u8 = 0x09;   // 8 位整数
pub const TAG_UINT64: u8 = 0x0A;  // 64 位整数

/// 常见文件 Tag 类型
pub const FT_FILENAME: u8 = 0x01;     // 文件名
pub const FT_FILESIZE: u8 = 0x02;     // 文件大小
pub const FT_FILETYPE: u8 = 0x03;     // 文件类型
pub const FT_FILEFORMAT: u8 = 0x04;   // 文件格式
pub const FT_SOURCES: u8 = 0x15;      // 源数量

/// Tag 值枚举
#[derive(Debug, Clone)]
pub enum TagValue {
    Hash([u8; 16]),
    String(String),
    UInt32(u32),
    UInt16(u16),
    UInt8(u8),
    UInt64(u64),
    Float32(f32),
    Bool(bool),
    Blob(Vec<u8>),
}

/// Tag 结构
#[derive(Debug, Clone)]
pub struct Tag {
    /// Tag 类型
    pub tag_type: u8,
    /// Tag 名称（单字节 ID 或字符串名称）
    pub name: TagName,
    /// Tag 值
    pub value: TagValue,
}

/// Tag 名称
#[derive(Debug, Clone)]
pub enum TagName {
    /// 单字节 ID（常见 tag 使用）
    Id(u8),
    /// 字符串名称（自定义 tag 使用）
    Name(String),
}

impl Tag {
    /// 创建单字节 ID 的 tag
    pub fn new_id(tag_type: u8, name_id: u8, value: TagValue) -> Self {
        Tag {
            tag_type,
            name: TagName::Id(name_id),
            value,
        }
    }

    /// 创建字符串名称的 tag
    pub fn new_name(tag_type: u8, name: &str, value: TagValue) -> Self {
        Tag {
            tag_type,
            name: TagName::Name(name.to_string()),
            value,
        }
    }

    /// 创建文件名 tag
    pub fn filename(name: &str) -> Self {
        Tag::new_id(TAG_STRING, FT_FILENAME, TagValue::String(name.to_string()))
    }

    /// 创建文件大小 tag
    pub fn filesize(size: u64) -> Self {
        if size <= u32::MAX as u64 {
            Tag::new_id(TAG_UINT32, FT_FILESIZE, TagValue::UInt32(size as u32))
        } else {
            Tag::new_id(TAG_UINT64, FT_FILESIZE, TagValue::UInt64(size))
        }
    }

    /// 从字节流解析单个 Tag
    pub fn from_bytes(buf: &mut BytesMut) -> Option<Self> {
        if buf.remaining() < 1 {
            return None;
        }

        let tag_type = buf.get_u8();

        // 读取名称
        let name = if tag_type & 0x80 != 0 {
            // 单字节 ID
            let name_id = buf.get_u8();
            TagName::Id(name_id)
        } else {
            // 字符串名称
            if buf.remaining() < 2 {
                return None;
            }
            let name_len = buf.get_u16_le() as usize;
            if buf.remaining() < name_len {
                return None;
            }
            let name_bytes = buf.copy_to_bytes(name_len);
            let name_str = String::from_utf8_lossy(&name_bytes).to_string();
            TagName::Name(name_str)
        };

        // 清除高位获取实际类型
        let real_type = tag_type & 0x7F;

        // 读取值
        let value = match real_type {
            TAG_HASH => {
                if buf.remaining() < 16 {
                    return None;
                }
                let mut hash = [0u8; 16];
                buf.copy_to_slice(&mut hash);
                TagValue::Hash(hash)
            }
            TAG_STRING => {
                if buf.remaining() < 2 {
                    return None;
                }
                let len = buf.get_u16_le() as usize;
                if buf.remaining() < len {
                    return None;
                }
                let bytes = buf.copy_to_bytes(len);
                TagValue::String(String::from_utf8_lossy(&bytes).to_string())
            }
            TAG_UINT32 => {
                if buf.remaining() < 4 {
                    return None;
                }
                TagValue::UInt32(buf.get_u32_le())
            }
            TAG_UINT16 => {
                if buf.remaining() < 2 {
                    return None;
                }
                TagValue::UInt16(buf.get_u16_le())
            }
            TAG_UINT8 => {
                if buf.remaining() < 1 {
                    return None;
                }
                TagValue::UInt8(buf.get_u8())
            }
            TAG_UINT64 => {
                if buf.remaining() < 8 {
                    return None;
                }
                TagValue::UInt64(buf.get_u64_le())
            }
            TAG_FLOAT32 => {
                if buf.remaining() < 4 {
                    return None;
                }
                TagValue::Float32(f32::from_bits(buf.get_u32_le()))
            }
            TAG_BOOL => {
                if buf.remaining() < 1 {
                    return None;
                }
                TagValue::Bool(buf.get_u8() != 0)
            }
            _ => return None,
        };

        Some(Tag {
            tag_type: real_type,
            name,
            value,
        })
    }

    /// 序列化为字节流
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();

        // Tag 类型
        let type_byte = match &self.name {
            TagName::Id(_) => self.tag_type | 0x80, // 设置高位表示单字节 ID
            TagName::Name(_) => self.tag_type,
        };
        buf.put_u8(type_byte);

        // 名称
        match &self.name {
            TagName::Id(id) => {
                buf.put_u8(*id);
            }
            TagName::Name(name) => {
                let name_bytes = name.as_bytes();
                buf.put_u16_le(name_bytes.len() as u16);
                buf.extend_from_slice(name_bytes);
            }
        }

        // 值
        match &self.value {
            TagValue::Hash(hash) => {
                buf.extend_from_slice(hash);
            }
            TagValue::String(s) => {
                let bytes = s.as_bytes();
                buf.put_u16_le(bytes.len() as u16);
                buf.extend_from_slice(bytes);
            }
            TagValue::UInt32(v) => {
                buf.put_u32_le(*v);
            }
            TagValue::UInt16(v) => {
                buf.put_u16_le(*v);
            }
            TagValue::UInt8(v) => {
                buf.put_u8(*v);
            }
            TagValue::UInt64(v) => {
                buf.put_u64_le(*v);
            }
            TagValue::Float32(v) => {
                buf.put_u32_le(v.to_bits());
            }
            TagValue::Bool(v) => {
                buf.put_u8(if *v { 1 } else { 0 });
            }
            TagValue::Blob(data) => {
                buf.put_u32_le(data.len() as u32);
                buf.extend_from_slice(data);
            }
        }

        buf
    }
}

/// 解析 Tag 列表
pub fn parse_tag_list(buf: &mut BytesMut) -> Vec<Tag> {
    if buf.remaining() < 1 {
        return Vec::new();
    }

    let count = buf.get_u8() as usize;
    let mut tags = Vec::with_capacity(count);

    for _ in 0..count {
        match Tag::from_bytes(buf) {
            Some(tag) => tags.push(tag),
            None => break,
        }
    }

    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_tag() {
        let tag = Tag::filename("test.txt");
        let bytes = tag.to_bytes();
        let mut buf = BytesMut::from(&bytes[..]);
        let parsed = Tag::from_bytes(&mut buf).unwrap();

        if let TagValue::String(name) = parsed.value {
            assert_eq!(name, "test.txt");
        } else {
            panic!("Expected string tag value");
        }
    }

    #[test]
    fn test_filesize_tag() {
        let tag = Tag::filesize(1024 * 1024);
        let bytes = tag.to_bytes();
        let mut buf = BytesMut::from(&bytes[..]);
        let parsed = Tag::from_bytes(&mut buf).unwrap();

        if let TagValue::UInt32(size) = parsed.value {
            assert_eq!(size, 1024 * 1024);
        } else {
            panic!("Expected u32 tag value");
        }
    }
}
