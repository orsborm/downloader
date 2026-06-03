// 插件管理 IPC 命令

use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// 插件信息（前端展示用）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfoCmd {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: String,
    pub state: String,
}

/// 获取插件列表
#[tauri::command]
pub async fn list_plugins(
    state: State<'_, AppState>,
) -> Result<Vec<PluginInfoCmd>, String> {
    let manager = state.plugin_manager.lock().await;
    let plugins: Vec<PluginInfoCmd> = manager
        .list_plugins()
        .iter()
        .map(|p| PluginInfoCmd {
            id: p.id.clone(),
            name: p.name.clone(),
            version: p.version.clone(),
            description: p.description.clone(),
            author: p.author.clone(),
            plugin_type: format!("{:?}", p.plugin_type).to_lowercase(),
            state: format!("{:?}", p.state).to_lowercase(),
        })
        .collect();
    Ok(plugins)
}

/// 安装插件
#[tauri::command]
pub async fn install_plugin(
    state: State<'_, AppState>,
    wasm_path: String,
) -> Result<String, String> {
    let path = std::path::Path::new(&wasm_path);

    if !path.exists() {
        return Err(format!("文件不存在: {}", wasm_path));
    }

    let mut manager = state.plugin_manager.lock().await;

    if let Err(e) = manager.validate_wasm_plugin(path) {
        return Err(format!("无效的 WASM 插件: {}", e));
    }

    match manager.install_plugin(path) {
        Ok(id) => Ok(id),
        Err(e) => Err(format!("安装插件失败: {}", e)),
    }
}

/// 卸载插件
#[tauri::command]
pub async fn uninstall_plugin(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock().await;
    match manager.uninstall_plugin(&id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("卸载插件失败: {}", e)),
    }
}

/// 启用插件
#[tauri::command]
pub async fn enable_plugin(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock().await;
    match manager.enable_plugin(&id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("启用插件失败: {}", e)),
    }
}

/// 禁用插件
#[tauri::command]
pub async fn disable_plugin(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let mut manager = state.plugin_manager.lock().await;
    match manager.disable_plugin(&id) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("禁用插件失败: {}", e)),
    }
}
