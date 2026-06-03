// RSS/Atom Feed 解析模块
// 使用 quick-xml 解析 RSS 2.0 和 Atom 格式

use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use tracing::debug;

use super::{RssFeed, RssItem};

/// 获取并解析 RSS/Atom feed
pub async fn fetch_feed(url: &str) -> Result<Vec<RssItem>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .context("获取 RSS feed 失败")?;

    let content = response
        .text()
        .await
        .context("读取 RSS feed 内容失败")?;

    parse_feed(&content)
}

/// 解析 RSS/Atom feed 内容
pub fn parse_feed(content: &str) -> Result<Vec<RssItem>> {
    let mut items = Vec::new();

    // 检测格式并解析
    if content.contains("<rss") || content.contains("<channel") {
        items.extend(parse_rss_items(content)?);
    } else if content.contains("<feed") && content.contains("xmlns") {
        items.extend(parse_atom_items(content)?);
    }

    debug!("解析到 {} 个 feed 条目", items.len());
    Ok(items)
}

/// 解析 RSS 2.0 条目（使用 quick-xml）
fn parse_rss_items(content: &str) -> Result<Vec<RssItem>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut in_item = false;
    let mut current_title = String::new();
    let mut current_link = String::new();
    let mut current_description = String::new();
    let mut current_pub_date: Option<String> = None;
    let mut current_guid = String::new();
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "item" {
                    in_item = true;
                    current_title.clear();
                    current_link.clear();
                    current_description.clear();
                    current_pub_date = None;
                    current_guid.clear();
                } else if in_item {
                    current_tag = tag;
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_item {
                    let text = t.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "title" => current_title = text,
                        "link" => current_link = text,
                        "description" => current_description = text,
                        "pubDate" => current_pub_date = Some(text),
                        "guid" => current_guid = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "item" {
                    if !current_link.is_empty() {
                        if current_guid.is_empty() {
                            current_guid = current_link.clone();
                        }
                        items.push(RssItem {
                            title: current_title.clone(),
                            link: current_link.clone(),
                            description: current_description.clone(),
                            pub_date: current_pub_date.clone(),
                            guid: current_guid.clone(),
                        });
                    }
                    in_item = false;
                } else if in_item {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("RSS 解析错误: {}", e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(items)
}

/// 解析 Atom 条目（使用 quick-xml）
fn parse_atom_items(content: &str) -> Result<Vec<RssItem>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut items = Vec::new();
    let mut in_entry = false;
    let mut current_title = String::new();
    let mut current_link = String::new();
    let mut current_description = String::new();
    let mut current_published: Option<String> = None;
    let mut current_id = String::new();
    let mut current_tag = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "entry" {
                    in_entry = true;
                    current_title.clear();
                    current_link.clear();
                    current_description.clear();
                    current_published = None;
                    current_id.clear();
                } else if in_entry {
                    current_tag = tag.clone();
                    // Atom 链接是自闭合标签 <link href="..." />
                    if tag == "link" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                current_link = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if in_entry && tag == "link" {
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"href" {
                            current_link = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                if in_entry {
                    let text = t.unescape().unwrap_or_default().to_string();
                    match current_tag.as_str() {
                        "title" => current_title = text,
                        "summary" | "content" => current_description = text,
                        "published" | "updated" => {
                            if current_published.is_none() {
                                current_published = Some(text);
                            }
                        }
                        "id" => current_id = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "entry" {
                    if !current_link.is_empty() {
                        if current_id.is_empty() {
                            current_id = current_link.clone();
                        }
                        items.push(RssItem {
                            title: current_title.clone(),
                            link: current_link.clone(),
                            description: current_description.clone(),
                            pub_date: current_published.clone(),
                            guid: current_id.clone(),
                        });
                    }
                    in_entry = false;
                } else if in_entry {
                    current_tag.clear();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("Atom 解析错误: {}", e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(items)
}

/// 解析 OPML 内容，返回订阅列表（使用 quick-xml）
pub fn parse_opml(content: &str) -> Result<Vec<RssFeed>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut feeds = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "outline" {
                    let mut name = String::new();
                    let mut url = String::new();

                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"text" | b"title" => {
                                if name.is_empty() {
                                    name = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                            b"xmlUrl" | b"url" => {
                                if url.is_empty() {
                                    url = String::from_utf8_lossy(&attr.value).to_string();
                                }
                            }
                            _ => {}
                        }
                    }

                    if !url.is_empty() {
                        if name.is_empty() {
                            name = "未命名订阅".to_string();
                        }
                        feeds.push(RssFeed {
                            id: uuid::Uuid::new_v4().to_string(),
                            name,
                            url,
                            enabled: true,
                            interval: 1800, // 默认 30 分钟
                            rules: Vec::new(),
                            save_dir: String::new(),
                            last_update: None,
                            processed: std::collections::HashSet::new(),
                            last_poll: None,
                        });
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("OPML 解析错误: {}", e));
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(feeds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
<channel>
  <title>Test Feed</title>
  <item>
    <title>Test Item</title>
    <link>https://example.com/file1.zip</link>
    <description>A test file</description>
    <guid>item-1</guid>
  </item>
</channel>
</rss>"#;

        let items = parse_rss_items(rss).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Item");
        assert_eq!(items[0].link, "https://example.com/file1.zip");
        assert_eq!(items[0].guid, "item-1");
    }

    #[test]
    fn test_parse_rss_multiple_items() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
<channel>
  <title>Feed</title>
  <item>
    <title>Item 1</title>
    <link>https://example.com/1</link>
    <description>First</description>
  </item>
  <item>
    <title>Item 2</title>
    <link>https://example.com/2</link>
    <description>Second</description>
  </item>
</channel>
</rss>"#;

        let items = parse_rss_items(rss).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Item 1");
        assert_eq!(items[1].title, "Item 2");
    }

    #[test]
    fn test_parse_atom() {
        let atom = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Feed</title>
  <entry>
    <title>Test Entry</title>
    <link href="https://example.com/file.zip" />
    <summary>A test file</summary>
    <id>entry-1</id>
    <updated>2026-01-01T00:00:00Z</updated>
  </entry>
</feed>"#;

        let items = parse_atom_items(atom).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test Entry");
        assert_eq!(items[0].link, "https://example.com/file.zip");
        assert_eq!(items[0].guid, "entry-1");
    }

    #[test]
    fn test_parse_opml() {
        let opml = r#"<?xml version="1.0"?>
<opml version="2.0">
<body>
  <outline text="Feed 1" xmlUrl="https://example.com/feed1.xml" />
  <outline text="Feed 2" xmlUrl="https://example.com/feed2.xml" />
</body>
</opml>"#;

        let feeds = parse_opml(opml).unwrap();
        assert_eq!(feeds.len(), 2);
        assert_eq!(feeds[0].name, "Feed 1");
        assert_eq!(feeds[1].name, "Feed 2");
    }

    #[test]
    fn test_parse_feed_auto_detect() {
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
<channel>
  <item>
    <title>Test</title>
    <link>https://example.com/file.zip</link>
    <description>desc</description>
  </item>
</channel>
</rss>"#;

        let items = parse_feed(rss).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_rss_no_link() {
        // 条目没有 link 应该被跳过
        let rss = r#"<?xml version="1.0"?>
<rss version="2.0">
<channel>
  <item>
    <title>No Link Item</title>
    <description>no link</description>
  </item>
</channel>
</rss>"#;

        let items = parse_rss_items(rss).unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_parse_atom_html_entities() {
        let atom = r#"<?xml version="1.0"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <title>Test &amp; File</title>
    <link href="https://example.com/file.zip" />
  </entry>
</feed>"#;

        let items = parse_atom_items(atom).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Test & File");
    }
}
