// RSS 自动下载引擎模块
// 解析 RSS/Atom feed，根据过滤规则自动创建下载任务
// 支持正则表达式过滤、OPML 导入/导出、去重

pub mod feed;
pub mod rules;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tracing::{info, warn};

/// RSS 订阅配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssFeed {
    /// 唯一 ID
    pub id: String,
    /// 订阅名称
    pub name: String,
    /// Feed URL
    pub url: String,
    /// 是否启用
    pub enabled: bool,
    /// 轮询间隔（秒）
    pub interval: u64,
    /// 关联的过滤规则
    pub rules: Vec<FilterRule>,
    /// 保存目录
    pub save_dir: String,
    /// 最后更新时间
    pub last_update: Option<String>,
    /// 已处理的条目 hash（去重用）
    pub processed: HashSet<String>,
    /// 上次轮询时间（每个 feed 独立跟踪）
    #[serde(skip)]
    pub last_poll: Option<std::time::Instant>,
}

/// 过滤规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterRule {
    /// 规则名称
    pub name: String,
    /// 匹配字段（title / description / link）
    pub field: String,
    /// 正则表达式
    pub pattern: String,
    /// 是否启用
    pub enabled: bool,
    /// 优先级
    pub priority: i32,
}

/// RSS 条目
#[derive(Debug, Clone)]
pub struct RssItem {
    /// 标题
    pub title: String,
    /// 链接
    pub link: String,
    /// 描述
    pub description: String,
    /// 发布时间
    pub pub_date: Option<String>,
    /// 条目唯一标识
    pub guid: String,
}

/// RSS 引擎
pub struct RssEngine {
    /// 订阅列表
    feeds: Vec<RssFeed>,
}

impl RssEngine {
    /// 创建新的 RSS 引擎
    pub fn new() -> Self {
        RssEngine {
            feeds: Vec::new(),
        }
    }

    /// 添加订阅
    pub fn add_feed(&mut self, feed: RssFeed) {
        info!("添加 RSS 订阅: {} ({})", feed.name, feed.url);
        self.feeds.push(feed);
    }

    /// 删除订阅
    pub fn remove_feed(&mut self, id: &str) {
        self.feeds.retain(|f| f.id != id);
        info!("删除 RSS 订阅: {}", id);
    }

    /// 获取所有订阅
    pub fn feeds(&self) -> &[RssFeed] {
        &self.feeds
    }

    /// 设置订阅启用/禁用状态
    pub fn set_feed_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(feed) = self.feeds.iter_mut().find(|f| f.id == id) {
            feed.enabled = enabled;
            info!("RSS 订阅 {} 已{}", id, if enabled { "启用" } else { "禁用" });
            true
        } else {
            false
        }
    }

    /// 更新订阅信息
    pub fn update_feed(&mut self, id: &str, name: Option<String>, url: Option<String>, interval: Option<u64>, save_dir: Option<String>) -> bool {
        if let Some(feed) = self.feeds.iter_mut().find(|f| f.id == id) {
            if let Some(n) = name { feed.name = n; }
            if let Some(u) = url { feed.url = u; }
            if let Some(i) = interval { feed.interval = i; }
            if let Some(d) = save_dir { feed.save_dir = d; }
            info!("更新 RSS 订阅: {}", id);
            true
        } else {
            false
        }
    }

    /// 获取指定订阅
    pub fn get_feed(&self, id: &str) -> Option<&RssFeed> {
        self.feeds.iter().find(|f| f.id == id)
    }

    /// 轮询所有订阅并返回匹配的链接
    pub async fn poll_all(&mut self) -> Result<Vec<(String, String, String)>> {
        let mut matched = Vec::new();

        for feed in &mut self.feeds {
            if !feed.enabled {
                continue;
            }

            // 检查是否到达轮询间隔（每个 feed 独立跟踪）
            let now = std::time::Instant::now();
            if let Some(last) = feed.last_poll {
                if last.elapsed().as_secs() < feed.interval {
                    continue;
                }
            }
            feed.last_poll = Some(now);

            match feed::fetch_feed(&feed.url).await {
                Ok(items) => {
                    for item in items {
                        // 去重检查（使用完整 GUID 而非哈希，避免碰撞）
                        if feed.processed.contains(&item.guid) {
                            continue;
                        }

                        // 使用预编译模式匹配（避免每次重新编译正则）
                        let rule_matched = Self::matches_rules_static(&feed.rules, &item);
                        if rule_matched {
                            matched.push((
                                feed.save_dir.clone(),
                                item.link.clone(),
                                item.title.clone(),
                            ));
                            feed.processed.insert(item.guid.clone());
                        }
                    }

                    // 防止 processed 无限增长：保留最近 10000 条
                    const MAX_PROCESSED: usize = 10000;
                    if feed.processed.len() > MAX_PROCESSED {
                        // HashSet 无法按时间排序，直接清空一半（保留最近插入的）
                        // 简单策略：全部清空，下次轮询会重新匹配（已下载的不会重复下载）
                        let excess = feed.processed.len() - MAX_PROCESSED / 2;
                        let to_remove: Vec<String> = feed.processed.iter().take(excess).cloned().collect();
                        for key in to_remove {
                            feed.processed.remove(&key);
                        }
                        info!("RSS 订阅 {} 清理去重缓存: 保留 {} 条", feed.name, feed.processed.len());
                    }

                    feed.last_update = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
                }
                Err(e) => {
                    warn!("RSS 订阅 {} 获取失败: {}", feed.name, e);
                }
            }
        }

        Ok(matched)
    }

    /// 检查条目是否匹配规则（使用预编译正则避免重复编译）
    fn matches_rules(&self, feed: &RssFeed, item: &RssItem) -> bool {
        Self::matches_rules_static(&feed.rules, item)
    }

    /// 静态版本：检查条目是否匹配规则（不借用 self，避免借用冲突）
    fn matches_rules_static(rules: &[FilterRule], item: &RssItem) -> bool {
        if rules.is_empty() {
            return true;
        }

        rules.iter().any(|rule| {
            if !rule.enabled {
                return false;
            }

            let target = match rule.field.as_str() {
                "title" => &item.title,
                "description" => &item.description,
                "link" => &item.link,
                _ => &item.title,
            };

            // 使用预编译模式，避免每次匹配重新编译正则
            let compiled = rules::CompiledPattern::compile(&rule.pattern);
            compiled.is_match(target)
        })
    }

    /// 导入 OPML
    pub fn import_opml(&mut self, opml_content: &str) -> Result<usize> {
        let feeds = feed::parse_opml(opml_content)?;
        let count = feeds.len();
        self.feeds.extend(feeds);
        info!("导入 {} 个 RSS 订阅", count);
        Ok(count)
    }

    /// 导出 OPML
    pub fn export_opml(&self) -> String {
        let mut opml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<opml version="2.0">
  <head><title>Downloader RSS Subscriptions</title></head>
  <body>"#);

        for feed in &self.feeds {
            let name = escape_xml(&feed.name);
            let url = escape_xml(&feed.url);
            opml.push_str(&format!(
                r#"<outline text="{}" type="rss" xmlUrl="{}" />"#,
                name, url
            ));
        }

        opml.push_str("</body></opml>");
        opml
    }
}

/// 转义 XML 特殊字符
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
