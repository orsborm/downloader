// 系统信息 IPC 命令
// 提供应用版本、平台信息、BT 状态、API 状态等

use crate::AppState;
use tauri::State;

/// 应用信息
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub platform: String,
    pub arch: String,
}

/// API 服务状态
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiStatus {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub endpoint: String,
    pub ws_connections: usize,
}

/// 获取应用信息
#[tauri::command]
pub async fn get_app_info() -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

/// 获取 BT 引擎状态（DHT 节点数等）
#[tauri::command]
pub async fn get_bt_status(
    state: State<'_, AppState>,
) -> Result<crate::engine::bt::BtStatus, String> {
    let manager = state.task_manager.lock().await;
    Ok(manager.get_bt_status().await)
}

/// 获取 JSON-RPC API 服务状态
#[tauri::command]
pub async fn get_api_status(
    state: State<'_, AppState>,
) -> Result<ApiStatus, String> {
    let (port, host) = {
        let config = state.config.lock().await;
        (config.connection.http_port, "127.0.0.1".to_string())
    };
    let ws_connections = state.ws_manager.connection_count().await;

    Ok(ApiStatus {
        enabled: port > 0,
        host: host.clone(),
        port,
        endpoint: if port > 0 {
            format!("http://{}:{}/jsonrpc", host, port)
        } else {
            String::new()
        },
        ws_connections,
    })
}

/// 获取 KAD（ed2k Kademlia DHT）状态
#[tauri::command]
pub async fn get_kad_status(
    state: State<'_, AppState>,
) -> Result<Option<crate::engine::ed2k::kad::KadStatus>, String> {
    let manager = state.task_manager.lock().await;
    Ok(manager.get_kad_status())
}
