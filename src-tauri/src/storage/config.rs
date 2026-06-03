// 应用配置模块
// 管理 config.toml 配置文件的读写，支持热更新

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 应用配置（对应 config.toml 结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub download: DownloadConfig,
    pub connection: ConnectionConfig,
    pub bt: BtConfig,
    pub http: HttpConfig,
    pub notification: NotificationConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub language: String,
    pub theme: String,       // light | dark | system
    pub minimize_to_tray: bool,
    pub close_to_tray: bool,
    pub auto_start: bool,
    pub font_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub default_dir: String,
    pub complete_dir: String,
    pub temp_dir: String,
    pub max_concurrent_tasks: usize,
    pub max_connections_per_task: usize,
    pub max_global_connections: usize,
    pub max_upload_speed: u64,    // bytes/sec, 0=不限
    pub max_download_speed: u64,
    pub auto_retry_count: u32,
    pub auto_retry_interval: u64, // 秒
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub bt_port: u16,
    pub ed2k_port: u16,
    pub http_port: u16,     // 0=禁用
    pub upnp: bool,
    pub nat_pmp: bool,
    pub proxy_type: String, // none | http | socks5
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_username: String,
    pub proxy_password: String,
    pub connection_timeout: u64,
    pub read_timeout: u64,
    /// JSON-RPC API 认证 token（空字符串=不认证）
    #[serde(default)]
    pub api_token: String,
    /// 自定义 ed2k 服务器列表（格式 "ip:port"）
    #[serde(default)]
    pub ed2k_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtConfig {
    pub dht: bool,
    pub pex: bool,
    pub lsd: bool,
    pub encryption: String,   // disabled | enabled | forced
    pub seed_ratio_limit: f64,
    pub seed_time_limit: u64, // 分钟, 0=不限
    pub trackers_file: String,
    /// 全局停止做种（完成后立即停止，不上传）
    #[serde(default)]
    pub stop_seeding: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub user_agent: String,
    pub cookie_policy: String, // auto | manual | disabled
    pub referer_policy: String, // strict | unsafe | none
    pub max_redirects: u32,
    pub verify_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub task_complete: bool,
    pub task_error: bool,
    pub sound: bool,
    pub position: String,
    pub duration: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub log_level: String,
    pub log_max_size: u32,     // MB
    pub log_retain_days: u32,
    pub db_backup_interval: u32, // 小时
    pub temp_cleanup_interval: u32,
    pub memory_limit: u32,     // MB, 0=不限
    /// 下载完成后动作: none | shutdown | hibernate | sleep | run_command
    #[serde(default)]
    pub post_download_action: String,
    /// 下载完成后执行的命令（post_download_action=run_command 时使用）
    #[serde(default)]
    pub post_download_command: String,
}

impl AppConfig {
    /// 默认配置
    pub fn default_config() -> Self {
        // 获取用户默认下载目录
        let default_dir = dirs_next::download_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\Downloads"))
            .to_string_lossy()
            .to_string();

        // 便携模式：临时目录在 EXE 同级 data/temp
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));
        let temp_dir = exe_dir.join("data").join("temp").to_string_lossy().to_string();

        AppConfig {
            general: GeneralConfig {
                language: "zh-CN".into(),
                theme: "system".into(),
                minimize_to_tray: true,
                close_to_tray: false,
                auto_start: false,
                font_size: 14,
            },
            download: DownloadConfig {
                default_dir,
                complete_dir: String::new(),
                temp_dir: temp_dir,
                max_concurrent_tasks: 3,
                max_connections_per_task: 64,
                max_global_connections: 200,
                max_upload_speed: 0,
                max_download_speed: 0,
                auto_retry_count: 3,
                auto_retry_interval: 5,
            },
            connection: ConnectionConfig {
                bt_port: 6881,
                ed2k_port: 4661,
                http_port: 0,
                upnp: true,
                nat_pmp: true,
                proxy_type: "none".into(),
                proxy_host: String::new(),
                proxy_port: 0,
                proxy_username: String::new(),
                proxy_password: String::new(),
                connection_timeout: 30,
                read_timeout: 60,
                api_token: String::new(),
                ed2k_servers: vec![
                    "91.200.42.46:4661".to_string(),
                    "176.103.48.41:4661".to_string(),
                ],
            },
            bt: BtConfig {
                dht: true,
                pex: true,
                lsd: true,
                encryption: "enabled".into(),
                seed_ratio_limit: 2.0,
                seed_time_limit: 1440,
                trackers_file: "./resources/trackers_best.txt".into(),
                stop_seeding: false,
            },
            http: HttpConfig {
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".into(),
                cookie_policy: "auto".into(),
                referer_policy: "strict".into(),
                max_redirects: 10,
                verify_ssl: true,
            },
            notification: NotificationConfig {
                task_complete: true,
                task_error: true,
                sound: true,
                position: "bottom-right".into(),
                duration: 5,
            },
            advanced: AdvancedConfig {
                log_level: "info".into(),
                log_max_size: 10,
                log_retain_days: 7,
                db_backup_interval: 24,
                temp_cleanup_interval: 1,
                memory_limit: 0,
                post_download_action: "none".into(),
                post_download_command: String::new(),
            },
        }
    }

    /// 从文件加载配置，文件不存在则创建默认配置
    /// 首次启动时自动生成 API token 并持久化
    pub fn load_or_default(config_path: &Path) -> Self {
        let mut config = if config_path.exists() {
            match std::fs::read_to_string(config_path) {
                Ok(content) => match toml::from_str::<AppConfig>(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        tracing::warn!("配置文件解析失败，使用默认配置: {}", e);
                        AppConfig::default_config()
                    }
                },
                Err(e) => {
                    tracing::warn!("配置文件读取失败，使用默认配置: {}", e);
                    AppConfig::default_config()
                }
            }
        } else {
            AppConfig::default_config()
        };

        // 解码敏感字段（从文件加载后还原明文）
        config.connection.api_token = decode_sensitive(&config.connection.api_token);
        config.connection.proxy_password = decode_sensitive(&config.connection.proxy_password);

        // 首次启动自动生成 API token（空 token = 未配置认证，需生成）
        if config.connection.api_token.is_empty() {
            let token = generate_api_token();
            tracing::info!("首次启动，自动生成 API token: {}***", &token[..8]);
            config.connection.api_token = token;
            // 保存配置以持久化 token
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            if let Ok(content) = toml::to_string_pretty(&config) {
                std::fs::write(config_path, content).ok();
            }
        }

        config
    }

    /// 获取当前 API token
    pub fn api_token(&self) -> &str {
        &self.connection.api_token
    }

    /// 保存配置到文件（敏感字段自动混淆编码）
    pub fn save(&self, config_path: &Path) -> anyhow::Result<()> {
        // 创建副本并编码敏感字段
        let mut save_config = self.clone();
        save_config.connection.api_token = encode_sensitive(&self.connection.api_token);
        save_config.connection.proxy_password = encode_sensitive(&self.connection.proxy_password);
        let content = toml::to_string_pretty(&save_config)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    /// 获取代理 URL（不含凭据，避免密码泄露到日志/Referer）
    pub fn proxy_url(&self) -> Option<String> {
        match self.connection.proxy_type.as_str() {
            "none" | "" => None,
            "http" => Some(format!(
                "http://{}:{}",
                self.connection.proxy_host,
                self.connection.proxy_port
            )),
            "socks5" => Some(format!(
                "socks5://{}:{}",
                self.connection.proxy_host,
                self.connection.proxy_port
            )),
            _ => None,
        }
    }

    /// 获取代理凭据（用户名、密码），无凭据时返回 None
    pub fn proxy_credentials(&self) -> Option<(String, String)> {
        if self.connection.proxy_username.is_empty() {
            None
        } else {
            Some((
                self.connection.proxy_username.clone(),
                self.connection.proxy_password.clone(),
            ))
        }
    }
}

/// 生成随机 API token（使用 UUID v4 格式，36 字符）
fn generate_api_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// XOR 编码密钥（防止配置文件中明文存储敏感信息）
const OBFUSCATE_KEY: &[u8] = b"downloader-config-key-2026";

/// 对字符串进行 XOR 混淆编码，返回 hex 字符串
fn obfuscate_encode(input: &str) -> String {
    let encoded: Vec<u8> = input
        .bytes()
        .enumerate()
        .map(|(i, b)| b ^ OBFUSCATE_KEY[i % OBFUSCATE_KEY.len()])
        .collect();
    hex_encode(&encoded)
}

/// 从 hex 字符串解码 XOR 混淆
fn obfuscate_decode(hex: &str) -> Option<String> {
    let bytes = hex_decode(hex)?;
    let decoded: Vec<u8> = bytes
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ OBFUSCATE_KEY[i % OBFUSCATE_KEY.len()])
        .collect();
    String::from_utf8(decoded).ok()
}

/// 简单 hex 编码
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 简单 hex 解码
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).ok()?;
        bytes.push(byte);
    }
    Some(bytes)
}

/// 标记值是否已混淆的前缀
const OBFUSCATED_PREFIX: &str = "obf:";

/// 对敏感字段值进行编码（如果尚未编码）
fn encode_sensitive(value: &str) -> String {
    if value.is_empty() || value.starts_with(OBFUSCATED_PREFIX) {
        value.to_string()
    } else {
        format!("{}{}", OBFUSCATED_PREFIX, obfuscate_encode(value))
    }
}

/// 对敏感字段值进行解码（如果已编码）
fn decode_sensitive(value: &str) -> String {
    if let Some(hex) = value.strip_prefix(OBFUSCATED_PREFIX) {
        obfuscate_decode(hex).unwrap_or_else(|| value.to_string())
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default_config();
        assert_eq!(config.general.language, "zh-CN");
        assert_eq!(config.general.theme, "system");
        assert!(config.general.minimize_to_tray);
        assert_eq!(config.download.max_concurrent_tasks, 3);
        assert_eq!(config.download.max_connections_per_task, 64);
        assert_eq!(config.connection.bt_port, 6881);
        assert_eq!(config.connection.ed2k_port, 4661);
        assert!(config.bt.dht);
        assert!(config.bt.pex);
        assert_eq!(config.advanced.log_level, "info");
    }

    #[test]
    fn test_hex_encode_decode() {
        let input = b"hello world";
        let encoded = hex_encode(input);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_hex_encode_empty() {
        let encoded = hex_encode(b"");
        assert!(encoded.is_empty());
        let decoded = hex_decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_hex_decode_odd_length() {
        assert!(hex_decode("abc").is_none()); // 奇数长度
    }

    #[test]
    fn test_obfuscate_roundtrip() {
        let original = "my_secret_token";
        let encoded = obfuscate_encode(original);
        let decoded = obfuscate_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_obfuscate_empty() {
        let encoded = obfuscate_encode("");
        assert!(encoded.is_empty());
        let decoded = obfuscate_decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_encode_sensitive_empty() {
        assert_eq!(encode_sensitive(""), "");
    }

    #[test]
    fn test_encode_sensitive_already_encoded() {
        let value = format!("{}{}", OBFUSCATED_PREFIX, "abcd");
        assert_eq!(encode_sensitive(&value), value);
    }

    #[test]
    fn test_decode_sensitive_not_encoded() {
        assert_eq!(decode_sensitive("plain_text"), "plain_text");
    }

    #[test]
    fn test_decode_sensitive_encoded() {
        let original = "secret";
        let encoded = encode_sensitive(original);
        assert_eq!(decode_sensitive(&encoded), original);
    }

    #[test]
    fn test_proxy_url_none() {
        let mut config = AppConfig::default_config();
        config.connection.proxy_type = "none".to_string();
        assert!(config.proxy_url().is_none());
    }

    #[test]
    fn test_proxy_url_http() {
        let mut config = AppConfig::default_config();
        config.connection.proxy_type = "http".to_string();
        config.connection.proxy_host = "127.0.0.1".to_string();
        config.connection.proxy_port = 8080;
        assert_eq!(config.proxy_url(), Some("http://127.0.0.1:8080".to_string()));
    }

    #[test]
    fn test_proxy_url_socks5() {
        let mut config = AppConfig::default_config();
        config.connection.proxy_type = "socks5".to_string();
        config.connection.proxy_host = "10.0.0.1".to_string();
        config.connection.proxy_port = 1080;
        assert_eq!(config.proxy_url(), Some("socks5://10.0.0.1:1080".to_string()));
    }

    #[test]
    fn test_proxy_credentials_none() {
        let config = AppConfig::default_config();
        assert!(config.proxy_credentials().is_none());
    }

    #[test]
    fn test_proxy_credentials_some() {
        let mut config = AppConfig::default_config();
        config.connection.proxy_username = "user".to_string();
        config.connection.proxy_password = "pass".to_string();
        let (user, pass) = config.proxy_credentials().unwrap();
        assert_eq!(user, "user");
        assert_eq!(pass, "pass");
    }

    #[test]
    fn test_config_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("config_test_roundtrip");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let mut config = AppConfig::default_config();
        config.general.language = "en".to_string();
        config.download.max_concurrent_tasks = 5;
        config.connection.api_token = "test_token_123".to_string();

        config.save(&path).unwrap();

        let loaded = AppConfig::load_or_default(&path);
        assert_eq!(loaded.general.language, "en");
        assert_eq!(loaded.download.max_concurrent_tasks, 5);
        assert_eq!(loaded.connection.api_token, "test_token_123");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_config_load_missing_file() {
        let path = PathBuf::from("/nonexistent/config.toml");
        let config = AppConfig::load_or_default(&path);
        // 应该返回默认配置
        assert_eq!(config.general.language, "zh-CN");
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let dir = std::env::temp_dir().join("config_test_invalid");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");
        std::fs::write(&path, "this is not valid toml {{{").unwrap();

        let config = AppConfig::load_or_default(&path);
        // 应该回退到默认配置
        assert_eq!(config.general.language, "zh-CN");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_api_token_auto_generation() {
        let dir = std::env::temp_dir().join("config_test_token_gen");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.toml");

        let config = AppConfig::load_or_default(&path);
        assert!(!config.connection.api_token.is_empty());
        assert!(config.connection.api_token.len() > 10);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
