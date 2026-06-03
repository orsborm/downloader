// RSS 过滤规则引擎
// 支持正则表达式匹配、通配符、关键词
// 提供预编译版本避免每次匹配重新编译正则

use regex::Regex;

/// 预编译的匹配模式（避免每次匹配重新编译正则）
#[derive(Debug, Clone)]
pub enum CompiledPattern {
    /// 包含匹配（不区分大小写）
    Contains(String),
    /// 取反匹配
    Not(Box<CompiledPattern>),
    /// 正则表达式（已编译）
    Regex(Regex),
    /// 正则表达式（编译失败，回退到包含匹配）
    RegexFallback(String),
    /// 通配符匹配
    Wildcard(String),
}

impl CompiledPattern {
    /// 从模式字符串预编译
    pub fn compile(pattern: &str) -> Self {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            return CompiledPattern::Contains(String::new());
        }
        // 取反匹配
        if let Some(inner) = pattern.strip_prefix('!') {
            return CompiledPattern::Not(Box::new(CompiledPattern::compile_inner(inner)));
        }
        CompiledPattern::compile_inner(pattern)
    }

    /// 内部编译逻辑
    fn compile_inner(pattern: &str) -> Self {
        let pattern_lower = pattern.to_lowercase();
        // 正则表达式
        if let Some(re_pattern) = pattern_lower.strip_prefix("re:") {
            return match Regex::new(re_pattern) {
                Ok(re) => CompiledPattern::Regex(re),
                Err(_) => CompiledPattern::RegexFallback(re_pattern.to_string()),
            };
        }
        // 通配符
        if let Some(wc_pattern) = pattern_lower.strip_prefix("wc:") {
            return CompiledPattern::Wildcard(wc_pattern.to_string());
        }
        // 默认包含匹配
        CompiledPattern::Contains(pattern_lower)
    }

    /// 使用预编译模式匹配文本
    pub fn is_match(&self, text: &str) -> bool {
        let text_lower = text.to_lowercase();
        match self {
            CompiledPattern::Contains(pattern) => {
                if pattern.is_empty() { true } else { text_lower.contains(pattern.as_str()) }
            }
            CompiledPattern::Not(inner) => !inner.is_match(text),
            CompiledPattern::Regex(re) => re.is_match(&text_lower),
            CompiledPattern::RegexFallback(pattern) => text_lower.contains(pattern.as_str()),
            CompiledPattern::Wildcard(pattern) => {
                wildcard_match(text_lower.as_bytes(), pattern.as_bytes())
            }
        }
    }
}

/// 检查文本是否匹配模式（每次重新编译，用于简单场景）
///
/// # 模式语法
/// - 纯文本：包含匹配（不区分大小写）
/// - 以 `re:` 开头：正则表达式匹配
/// - 以 `wc:` 开头：通配符匹配（* 匹配任意字符）
/// - 以 `!` 开头：取反匹配
pub fn matches_pattern(text: &str, pattern: &str) -> bool {
    let text_lower = text.to_lowercase();
    let pattern = pattern.trim();

    if pattern.is_empty() {
        return true;
    }

    // 取反匹配
    if let Some(inner) = pattern.strip_prefix('!') {
        return !matches_pattern_inner(&text_lower, inner);
    }

    matches_pattern_inner(&text_lower, pattern)
}

/// 内部匹配逻辑
fn matches_pattern_inner(text_lower: &str, pattern: &str) -> bool {
    let pattern_lower = pattern.to_lowercase();

    // 正则表达式匹配
    if let Some(re_pattern) = pattern_lower.strip_prefix("re:") {
        return matches_regex(text_lower, re_pattern);
    }

    // 通配符匹配
    if let Some(wc_pattern) = pattern_lower.strip_prefix("wc:") {
        return matches_wildcard(text_lower, wc_pattern);
    }

    // 默认：包含匹配
    text_lower.contains(&pattern_lower)
}

/// 正则表达式匹配（使用 regex crate）
fn matches_regex(text: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(re) => re.is_match(text),
        Err(_) => {
            // 正则无效时回退到包含匹配
            text.contains(pattern)
        }
    }
}

/// 通配符匹配（* 匹配任意字符序列，? 匹配单个字符）
fn matches_wildcard(text: &str, pattern: &str) -> bool {
    let text_bytes = text.as_bytes();
    let pattern_bytes = pattern.as_bytes();
    wildcard_match(text_bytes, pattern_bytes)
}

/// 通配符匹配核心算法
fn wildcard_match(text: &[u8], pattern: &[u8]) -> bool {
    let mut t = 0;
    let mut p = 0;
    let mut star_t = 0;
    let mut star_p = usize::MAX;

    while t < text.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == text[t]) {
            // 匹配单个字符
            t += 1;
            p += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            // 通配符：记录位置，尝试匹配零个或多个字符
            star_p = p;
            star_t = t;
            p += 1;
        } else if star_p != usize::MAX {
            // 回溯：尝试匹配更多字符
            p = star_p + 1;
            star_t += 1;
            t = star_t;
        } else {
            return false;
        }
    }

    // 检查 pattern 剩余部分是否都是 *
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_match() {
        assert!(matches_pattern("Hello World", "hello"));
        assert!(matches_pattern("Hello World", "WORLD"));
        assert!(!matches_pattern("Hello World", "xyz"));
    }

    #[test]
    fn test_negate_match() {
        assert!(matches_pattern("Hello World", "!xyz"));
        assert!(!matches_pattern("Hello World", "!hello"));
    }

    #[test]
    fn test_wildcard_match() {
        assert!(matches_pattern("test.zip", "wc:*.zip"));
        assert!(matches_pattern("test.tar.gz", "wc:*.tar.gz"));
        assert!(matches_pattern("file01.txt", "wc:file??.txt"));
        assert!(!matches_pattern("test.txt", "wc:*.zip"));
    }

    #[test]
    fn test_regex_match() {
        assert!(matches_pattern("my_test_file.zip", "re:.*test.*\\.zip"));
        assert!(matches_pattern("test.zip", "re:.*test.*\\.zip"));
        assert!(!matches_pattern("test.txt", "re:.*test.*\\.zip"));
        assert!(matches_pattern("abc123def", "re:\\d+"));
        assert!(!matches_pattern("abcdef", "re:\\d+"));
    }

    #[test]
    fn test_regex_complex_patterns() {
        // 字符类
        assert!(matches_pattern("hello123", "re:[a-z]+\\d+"));
        // 锚点
        assert!(matches_pattern("start of line", "re:^start"));
        assert!(!matches_pattern("not start", "re:^start"));
        // 量词
        assert!(matches_pattern("aaa", "re:a{3}"));
        assert!(!matches_pattern("aa", "re:a{3}"));
    }

    #[test]
    fn test_invalid_regex_fallback() {
        // 无效正则应回退到包含匹配
        assert!(matches_pattern("test[invalid", "re:[invalid"));
    }

    #[test]
    fn test_combined_patterns() {
        // 取反 + 正则
        assert!(matches_pattern("hello", "!re:\\d+"));
        assert!(!matches_pattern("hello123", "!re:\\d+"));
    }
}
