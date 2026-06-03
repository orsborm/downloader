// WebSocket 实时推送模块
// 向连接的客户端推送任务状态变化事件

use super::ApiEvent;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info};

/// WebSocket 连接 ID
pub type ConnectionId = u64;

/// WebSocket 连接信息
#[derive(Debug)]
pub struct WsConnection {
    pub id: ConnectionId,
    pub subscribed_events: Vec<String>,
}

/// WebSocket 连接管理器
pub struct WsManager {
    /// 广播通道（发送事件给所有连接的客户端）
    broadcast_tx: broadcast::Sender<WsEvent>,
    /// 活跃连接列表
    connections: Arc<RwLock<HashMap<ConnectionId, WsConnection>>>,
    /// 连接计数器
    next_id: std::sync::atomic::AtomicU64,
}

/// WebSocket 事件格式
#[derive(Debug, Clone, Serialize)]
pub struct WsEvent {
    /// 事件类型
    pub event: String,
    /// 事件数据
    pub data: serde_json::Value,
    /// 时间戳
    pub timestamp: u64,
}

impl WsManager {
    /// 创建新的 WebSocket 管理器
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);

        WsManager {
            broadcast_tx,
            connections: Arc::new(RwLock::new(HashMap::new())),
            next_id: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 注册新连接
    pub async fn add_connection(&self) -> ConnectionId {
        let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let conn = WsConnection {
            id,
            subscribed_events: vec![
                "task-state-changed".to_string(),
                "task-completed".to_string(),
                "task-error".to_string(),
                "speed-update".to_string(),
            ],
        };

        self.connections.write().await.insert(id, conn);
        info!("WebSocket 连接已添加: {}", id);
        id
    }

    /// 移除连接
    pub async fn remove_connection(&self, id: ConnectionId) {
        self.connections.write().await.remove(&id);
        info!("WebSocket 连接已移除: {}", id);
    }

    /// 获取活跃连接数
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// 广播事件给所有连接
    pub fn broadcast(&self, event: WsEvent) {
        let _ = self.broadcast_tx.send(event);
    }

    /// 获取事件接收端
    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.broadcast_tx.subscribe()
    }

    /// 将 API 事件转换为 WebSocket 事件并广播
    pub fn broadcast_api_event(&self, api_event: ApiEvent) {
        let ws_event = match api_event {
            ApiEvent::TaskStateChanged { task_id, state } => WsEvent {
                event: "task-state-changed".to_string(),
                data: serde_json::json!({ "taskId": task_id, "state": state }),
                timestamp: now_millis(),
            },
            ApiEvent::TaskCompleted { task_id } => WsEvent {
                event: "task-completed".to_string(),
                data: serde_json::json!({ "taskId": task_id }),
                timestamp: now_millis(),
            },
            ApiEvent::TaskError { task_id, error } => WsEvent {
                event: "task-error".to_string(),
                data: serde_json::json!({ "taskId": task_id, "error": error }),
                timestamp: now_millis(),
            },
            ApiEvent::SpeedUpdate { download_speed, upload_speed } => WsEvent {
                event: "speed-update".to_string(),
                data: serde_json::json!({
                    "downloadSpeed": download_speed,
                    "uploadSpeed": upload_speed,
                }),
                timestamp: now_millis(),
            },
        };

        self.broadcast(ws_event);
    }
}

/// 获取当前时间戳（毫秒）
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
