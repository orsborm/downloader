// 解压密码管理模块
// 管理常用解压密码，支持按域名/路径关联

use serde::{Deserialize, Serialize};

/// 密码管理器
pub struct PasswordManager {
    passwords: Vec<PasswordEntry>,
}

/// 密码条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordEntry {
    /// 唯一 ID
    pub id: String,
    /// 名称/描述
    pub name: String,
    /// 密码值
    pub password: String,
    /// 关联域名模式（如 "example.com"）
    pub domain_pattern: String,
    /// 关联路径模式（如 "/downloads/"）
    pub path_pattern: String,
    /// 是否启用
    pub enabled: bool,
    /// 使用次数
    pub use_count: u32,
}

impl PasswordManager {
    /// 创建新的密码管理器
    pub fn new() -> Self {
        PasswordManager {
            passwords: Vec::new(),
        }
    }

    /// 添加密码
    pub fn add(&mut self, entry: PasswordEntry) {
        self.passwords.push(entry);
    }

    /// 删除密码
    pub fn remove(&mut self, id: &str) {
        self.passwords.retain(|p| p.id != id);
    }

    /// 获取所有密码
    pub fn all(&self) -> &[PasswordEntry] {
        &self.passwords
    }

    /// 根据文件路径查找匹配的密码
    pub fn find_for_file(&self, file_path: &str) -> Vec<&PasswordEntry> {
        self.passwords
            .iter()
            .filter(|p| p.enabled)
            .filter(|p| {
                let domain_match = p.domain_pattern.is_empty()
                    || file_path.contains(&p.domain_pattern);
                let path_match = p.path_pattern.is_empty()
                    || file_path.contains(&p.path_pattern);
                domain_match && path_match
            })
            .collect()
    }

    /// 记录密码使用
    pub fn record_use(&mut self, id: &str) {
        if let Some(entry) = self.passwords.iter_mut().find(|p| p.id == id) {
            entry.use_count += 1;
        }
    }

    /// 按使用次数排序获取密码列表
    pub fn sorted_by_usage(&self) -> Vec<&PasswordEntry> {
        let mut entries: Vec<&PasswordEntry> = self.passwords.iter().collect();
        entries.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        entries
    }
}
