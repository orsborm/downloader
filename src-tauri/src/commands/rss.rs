// RSS 订阅管理 IPC 命令
// 每次变更后自动持久化到数据库

use crate::rss::RssFeed;
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, warn};

/// RSS 订阅信息（返回给前端）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RssFeedInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub interval: u64,
    pub last_update: Option<String>,
}

impl From<&RssFeed> for RssFeedInfo {
    fn from(feed: &RssFeed) -> Self {
        RssFeedInfo {
            id: feed.id.clone(),
            name: feed.name.clone(),
            url: feed.url.clone(),
            enabled: feed.enabled,
            interval: feed.interval,
            last_update: feed.last_update.clone(),
        }
    }
}

/// 辅助函数：将 RSS 引擎中的订阅持久化到数据库
async fn persist_rss(state: &State<'_, AppState>) {
    let engine = state.rss_engine.lock().await;
    let db = state.db.lock().await;
    if let Err(e) = db.save_rss_feeds(engine.feeds()) {
        warn!("持久化 RSS 订阅失败: {}", e);
    }
}

/// 添加 RSS 订阅
#[tauri::command]
pub async fn add_rss_feed(
    state: State<'_, AppState>,
    name: String,
    url: String,
    interval: u64,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let feed = RssFeed {
        id: id.clone(),
        name,
        url,
        enabled: true,
        interval,
        rules: Vec::new(),
        save_dir: String::new(),
        last_update: None,
        processed: std::collections::HashSet::new(),
        last_poll: None,
    };

    let mut engine = state.rss_engine.lock().await;
    engine.add_feed(feed);
    drop(engine);
    info!("添加 RSS 订阅: {}", id);
    persist_rss(&state).await;
    Ok(id)
}

/// 删除 RSS 订阅
#[tauri::command]
pub async fn remove_rss_feed(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut engine = state.rss_engine.lock().await;
    engine.remove_feed(&id);
    drop(engine);
    info!("删除 RSS 订阅: {}", id);
    persist_rss(&state).await;
    Ok(())
}

/// 获取所有 RSS 订阅
#[tauri::command]
pub async fn get_rss_feeds(
    state: State<'_, AppState>,
) -> Result<Vec<RssFeedInfo>, String> {
    let engine = state.rss_engine.lock().await;
    Ok(engine.feeds().iter().map(RssFeedInfo::from).collect())
}

/// 导入 OPML
#[tauri::command]
pub async fn import_opml(
    state: State<'_, AppState>,
    content: String,
) -> Result<usize, String> {
    let mut engine = state.rss_engine.lock().await;
    let count = engine.import_opml(&content).map_err(|e| e.to_string())?;
    drop(engine);
    info!("导入 {} 个 RSS 订阅", count);
    persist_rss(&state).await;
    Ok(count)
}

/// 导出 OPML
#[tauri::command]
pub async fn export_opml(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let engine = state.rss_engine.lock().await;
    Ok(engine.export_opml())
}

/// 设置 RSS 订阅启用/禁用
#[tauri::command]
pub async fn set_rss_feed_enabled(
    state: State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut engine = state.rss_engine.lock().await;
    if engine.set_feed_enabled(&id, enabled) {
        drop(engine);
        persist_rss(&state).await;
        Ok(())
    } else {
        Err("订阅不存在".to_string())
    }
}

/// 更新 RSS 订阅
#[tauri::command]
pub async fn update_rss_feed(
    state: State<'_, AppState>,
    id: String,
    name: Option<String>,
    url: Option<String>,
    interval: Option<u64>,
    save_dir: Option<String>,
) -> Result<(), String> {
    let mut engine = state.rss_engine.lock().await;
    if engine.update_feed(&id, name, url, interval, save_dir) {
        drop(engine);
        persist_rss(&state).await;
        Ok(())
    } else {
        Err("订阅不存在".to_string())
    }
}
