// 工具模块
// 包含格式化、剪贴板监听、便携模式检测等通用功能

/// 格式化文件大小（自动选择 B/KB/MB/GB/TB）
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.1} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// 格式化下载速度
pub fn format_speed(bytes_per_sec: u64) -> String {
    format!("{}/s", format_size(bytes_per_sec))
}

/// 格式化剩余时间
pub fn format_eta(seconds: u64) -> String {
    if seconds == 0 {
        return "∞".to_string();
    }
    if seconds < 60 {
        return format!("{}s", seconds);
    }
    if seconds < 3600 {
        let mins = seconds / 60;
        let secs = seconds % 60;
        return format!("{}m{}s", mins, secs);
    }
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    format!("{}h{}m", hours, mins)
}

/// 从 URL 中提取文件名
pub fn extract_filename(url: &str) -> String {
    let lower = url.to_lowercase();

    // magnet 链接：提取 dn (display name) 参数，或用 hash 前 8 位
    if lower.starts_with("magnet:") {
        // 尝试从查询参数中提取 dn
        if let Ok(parsed) = url::Url::parse(url) {
            for (key, value) in parsed.query_pairs() {
                if key.eq_ignore_ascii_case("dn") && !value.is_empty() {
                    return sanitize_filename(&value);
                }
            }
            // 回退：用 btih hash 前 8 位
            for (key, value) in parsed.query_pairs() {
                if key.eq_ignore_ascii_case("xt") && value.starts_with("urn:btih:") {
                    let hash = &value[9..];
                    let short = &hash[..hash.len().min(8)];
                    return format!("magnet_{}", short);
                }
            }
        }
        return "magnet_download".to_string();
    }

    // ed2k 链接：格式 ed2k://|file|<name>|<size>|<hash>|/
    if lower.starts_with("ed2k://") {
        let body = &url[7..];
        let parts: Vec<&str> = body.split('|').collect();
        // 找到 "file" 标记后的文件名
        for (i, part) in parts.iter().enumerate() {
            if *part == "file" && i + 1 < parts.len() {
                return sanitize_filename(parts[i + 1]);
            }
        }
        return "ed2k_download".to_string();
    }

    // HTTP/FTP：从 URL 路径提取
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(mut segments) = parsed.path_segments() {
            if let Some(last) = segments.next_back() {
                if !last.is_empty() {
                    return urlencoding::decode(last)
                        .unwrap_or_else(|_| last.into())
                        .to_string();
                }
            }
        }
    }

    // 回退：使用域名
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return host.to_string();
        }
    }

    "download".to_string()
}

/// 清理文件名中的非法字符（保留中文、字母、数字、常见符号）
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 检测协议类型
pub fn detect_protocol(url: &str) -> &'static str {
    let lower = url.to_lowercase();
    if lower.starts_with("magnet:") {
        "MAGNET"
    } else if lower.starts_with("ed2k://") {
        "ED2K"
    } else if lower.starts_with("ftp://") || lower.starts_with("ftps://") {
        "FTP"
    } else if lower.split('?').next().map_or(false, |p| p.ends_with(".m3u8")) {
        "HLS"
    } else if lower.split('?').next().map_or(false, |p| p.ends_with(".mpd")) {
        "DASH"
    } else if lower.starts_with("http://") || lower.starts_with("https://") {
        "HTTP"
    } else if lower.ends_with(".torrent") {
        "BT"
    } else {
        "HTTP" // 默认
    }
}

/// 验证文件名是否安全（防止路径遍历攻击）
/// 返回 Ok(安全文件名) 或 Err(错误信息)
pub fn validate_filename(filename: &str) -> Result<String, String> {
    // 检查空文件名
    if filename.is_empty() {
        return Err("文件名不能为空".to_string());
    }

    // 检查路径遍历字符
    if filename.contains("..") {
        return Err("文件名不能包含 '..'".to_string());
    }

    // 检查路径分隔符
    if filename.contains('/') || filename.contains('\\') {
        return Err("文件名不能包含路径分隔符".to_string());
    }

    // 检查 Windows 保留字符
    if filename.contains(':') || filename.contains('*') || filename.contains('?')
        || filename.contains('"') || filename.contains('<') || filename.contains('>')
        || filename.contains('|') {
        return Err("文件名包含非法字符".to_string());
    }

    // 检查 Windows 保留名称
    let name_without_ext = filename.split('.').next().unwrap_or(filename).to_uppercase();
    let reserved_names = ["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4",
                          "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2",
                          "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"];
    if reserved_names.contains(&name_without_ext.as_str()) {
        return Err("文件名是系统保留名称".to_string());
    }

    // 检查文件名长度
    if filename.len() > 255 {
        return Err("文件名过长（最大255字符）".to_string());
    }

    Ok(filename.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(0), "∞");
        assert_eq!(format_eta(30), "30s");
        assert_eq!(format_eta(90), "1m30s");
        assert_eq!(format_eta(3661), "1h1m");
    }

    #[test]
    fn test_detect_protocol() {
        assert_eq!(detect_protocol("https://example.com/file.zip"), "HTTP");
        assert_eq!(detect_protocol("ftp://example.com/file.zip"), "FTP");
        assert_eq!(detect_protocol("magnet:?xt=urn:btih:abc123"), "MAGNET");
        assert_eq!(detect_protocol("ed2k://|file|test|123|abc|/"), "ED2K");
        assert_eq!(detect_protocol("https://cdn.example.com/stream.m3u8"), "HLS");
        assert_eq!(detect_protocol("https://cdn.example.com/manifest.mpd"), "DASH");
    }
}
