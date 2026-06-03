// 自动解压管理 IPC 命令

use crate::archive::{ArchiveConfig, SavedPassword};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, warn};

/// 辅助函数：将解压密码持久化到数据库
async fn persist_passwords(state: &State<'_, AppState>) {
    let manager = state.archive_manager.lock().await;
    let db = state.db.lock().await;
    if let Err(e) = db.save_archive_passwords(&manager.config().passwords) {
        warn!("持久化解压密码失败: {}", e);
    }
}

/// 解压配置（返回给前端）
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveConfigInfo {
    pub auto_extract: bool,
    pub delete_after_extract: bool,
    pub extract_dir: String,
    pub passwords: Vec<PasswordInfo>,
}

/// 密码信息
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordInfo {
    pub id: String,
    pub name: String,
    pub password: String,
    pub pattern: String,
    pub enabled: bool,
}

impl From<&SavedPassword> for PasswordInfo {
    fn from(p: &SavedPassword) -> Self {
        PasswordInfo {
            id: p.id.clone(),
            name: p.name.clone(),
            password: p.password.clone(),
            pattern: p.pattern.clone(),
            enabled: p.enabled,
        }
    }
}

impl From<&ArchiveConfig> for ArchiveConfigInfo {
    fn from(config: &ArchiveConfig) -> Self {
        ArchiveConfigInfo {
            auto_extract: config.auto_extract,
            delete_after_extract: config.delete_after_extract,
            extract_dir: config.extract_dir.clone(),
            passwords: config.passwords.iter().map(PasswordInfo::from).collect(),
        }
    }
}

/// 获取解压配置
#[tauri::command]
pub async fn get_archive_config(
    state: State<'_, AppState>,
) -> Result<ArchiveConfigInfo, String> {
    let manager = state.archive_manager.lock().await;
    Ok(ArchiveConfigInfo::from(&*manager.config()))
}

/// 更新解压配置
#[tauri::command]
pub async fn update_archive_config(
    state: State<'_, AppState>,
    config: ArchiveConfigInfo,
) -> Result<(), String> {
    let archive_config = ArchiveConfig {
        auto_extract: config.auto_extract,
        delete_after_extract: config.delete_after_extract,
        extract_dir: config.extract_dir,
        passwords: config
            .passwords
            .iter()
            .map(|p| SavedPassword {
                id: if p.id.is_empty() { uuid::Uuid::new_v4().to_string() } else { p.id.clone() },
                name: p.name.clone(),
                password: p.password.clone(),
                pattern: p.pattern.clone(),
                enabled: p.enabled,
                use_count: 0,
                created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect(),
    };

    let mut manager = state.archive_manager.lock().await;
    manager.update_config(archive_config);
    drop(manager);
    info!("解压配置已更新");
    persist_passwords(&state).await;
    Ok(())
}

/// 手动解压文件
#[tauri::command]
pub async fn extract_archive(
    state: State<'_, AppState>,
    file_path: String,
    password: Option<String>,
) -> Result<Vec<String>, String> {
    let result = {
        let manager = state.archive_manager.lock().await;
        manager
            .auto_extract(&file_path)
            .await
            .map_err(|e| e.to_string())?
    };

    if result.success {
        Ok(result.files.iter().map(|p| p.to_string_lossy().to_string()).collect())
    } else {
        Err(result.error.unwrap_or_else(|| "解压失败".to_string()))
    }
}
