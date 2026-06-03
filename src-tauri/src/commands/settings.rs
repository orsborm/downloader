// 设置管理 IPC 命令
// 提供配置的读取和更新操作

use crate::storage::config::AppConfig;
use crate::AppState;
use tauri::State;
use tracing::info;

/// 获取当前配置
#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

/// 更新配置
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    new_config: AppConfig,
) -> Result<(), String> {
    // 保存新配置到文件
    new_config.save(&state.config_path).map_err(|e| e.to_string())?;

    // 同步配置到 TaskManager
    {
        let mut manager = state.task_manager.lock().await;
        manager.set_global_download_speed_limit(new_config.download.max_download_speed);
        manager.set_global_upload_speed_limit(new_config.download.max_upload_speed);
        manager.set_bt_speed_limits(new_config.download.max_download_speed, new_config.download.max_upload_speed);
        manager.set_max_concurrent(new_config.download.max_concurrent_tasks);
        manager.set_retry_config(
            new_config.download.auto_retry_count,
            new_config.download.auto_retry_interval,
        );
        // 同步 ed2k 自定义服务器列表
        manager.set_ed2k_servers(new_config.connection.ed2k_servers.clone());
        // 同步 BT 做种配置
        manager.set_bt_seeding_config(
            new_config.bt.seed_ratio_limit,
            new_config.bt.seed_time_limit,
            new_config.bt.stop_seeding,
        );
    }

    // 更新内存中的配置
    {
        let mut config = state.config.lock().await;
        *config = new_config;
    }

    info!("配置已更新");
    Ok(())
}
