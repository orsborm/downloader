// 任务管理 IPC 命令
// 提供给前端的任务 CRUD 和控制操作

use crate::storage::db::{Protocol, TaskParams, TaskState, TaskStatus};
use crate::util;
use crate::AppState;
use tauri::State;
use tracing::{info, warn};

/// 添加下载任务
#[tauri::command]
pub async fn add_task(
    state: State<'_, AppState>,
    params: TaskParams,
) -> Result<String, String> {
    let task_id = uuid::Uuid::new_v4().to_string();

    // 自动检测协议类型
    let protocol = match util::detect_protocol(&params.url) {
        "MAGNET" => Protocol::Magnet,
        "ED2K" => Protocol::Ed2k,
        "FTP" => Protocol::Ftp,
        "BT" => Protocol::Bt,
        _ => Protocol::Http,
    };

    // 提取文件名
    let raw_file_name = params.file_name.clone().unwrap_or_else(|| {
        util::extract_filename(&params.url)
    });

    // 验证文件名安全性（防止路径遍历攻击）
    let file_name = util::validate_filename(&raw_file_name)
        .map_err(|e| format!("文件名无效: {}", e))?;

    // 确定保存路径
    let save_path = if params.save_path.is_empty() {
        let config = state.config.lock().await;
        config.download.default_dir.clone()
    } else {
        params.save_path.clone()
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let task = TaskStatus {
        id: task_id.clone(),
        name: file_name,
        protocol,
        state: if params.start_immediately { TaskState::Downloading } else { TaskState::Queued },
        url: params.url.clone(),
        save_path,
        total_size: 0,
        downloaded: 0,
        uploaded: 0,
        download_speed: 0,
        upload_speed: 0,
        progress: 0.0,
        peers: 0,
        seeds: 0,
        eta: None,
        files: Vec::new(),
        error: None,
        added_at: now,
        completed_at: None,
        priority: 1,
        download_limit: 0,
        upload_limit: 0,
    };

    // 保存到数据库
    {
        let db = state.db.lock().await;
        db.insert_task(&task).map_err(|e| e.to_string())?;
        // 持久化下载参数（用于重启后恢复代理/认证/限速等配置）
        if let Err(e) = db.save_task_params(&task_id, &params) {
            tracing::warn!("保存任务参数失败（重启后可能丢失代理/认证配置）: {} - {}", task_id, e);
        }
    }

    // 启动下载
    if params.start_immediately {
        let mut manager = state.task_manager.lock().await;
        manager.add_task(task.clone(), &params).await.map_err(|e| e.to_string())?;
    }

    info!("任务已添加: {} ({})", task.name, task_id);
    Ok(task_id)
}

/// 暂停任务
#[tauri::command]
pub async fn pause_task(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // 更新数据库状态
    {
        let db = state.db.lock().await;
        db.update_task_state(&id, &TaskState::Paused).map_err(|e| e.to_string())?;
    }

    // 通知任务管理器
    {
        let mut manager = state.task_manager.lock().await;
        manager.pause_task(&id).await.map_err(|e| e.to_string())?;
    }

    info!("任务已暂停: {}", id);
    Ok(())
}

/// 恢复任务
#[tauri::command]
pub async fn resume_task(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    // 更新数据库状态（先获取 db 锁，再获取 task_manager 锁，统一锁顺序）
    {
        let db = state.db.lock().await;
        db.update_task_state(&id, &TaskState::Downloading).map_err(|e| e.to_string())?;
    }

    // 尝试恢复任务（先尝试 resume，如果需要重新添加则释放 manager 锁后再操作）
    let needs_re_add = {
        let mut manager = state.task_manager.lock().await;
        match manager.resume_task(&id).await {
            Ok(()) => false,
            Err(e) if e.to_string().contains("需要重新启动") => true,
            Err(e) => return Err(e.to_string()),
        }
    };

    // 任务已退出，需要重新添加（释放 manager 锁后获取 db 锁，避免死锁）
    if needs_re_add {
        let (task, params) = {
            let db = state.db.lock().await;
            let task = db.get_task(&id).map_err(|e| e.to_string())?;
            match task {
                Some(t) => {
                    let params = db.get_task_params(&id)
                        .ok()
                        .flatten()
                        .unwrap_or_else(|| TaskParams {
                            url: t.url.clone(),
                            save_path: t.save_path.clone(),
                            file_name: Some(t.name.clone()),
                            proxy: None,
                            speed_limit: None,
                            start_immediately: true,
                            only_files: None,
                            mirror_urls: None,
                            http_auth: None,
                            http_cookie: None,
                            http_headers: None,
                        });
                    (t, params)
                }
                None => return Err("任务不存在".to_string()),
            }
        };
        // 重新获取 manager 锁添加任务
        let mut manager = state.task_manager.lock().await;
        manager.add_task(task, &params).await.map_err(|e| e.to_string())?;
        info!("任务已重新启动: {}", id);
    }

    info!("任务已恢复: {}", id);
    Ok(())
}

/// 删除任务
#[tauri::command]
pub async fn remove_task(
    state: State<'_, AppState>,
    id: String,
    delete_files: bool,
) -> Result<(), String> {
    // 通知任务管理器取消下载
    {
        let mut manager = state.task_manager.lock().await;
        manager.remove_task(&id).await.map_err(|e| e.to_string())?;
    }

    // 如果需要删除文件
    if delete_files {
        let file_path = {
            let db = state.db.lock().await;
            if let Ok(Some(task)) = db.get_task(&id) {
                Some(std::path::Path::new(&task.save_path).join(&task.name))
            } else {
                None
            }
        };
        if let Some(file_path) = file_path {
            if file_path.exists() {
                tokio::fs::remove_file(&file_path).await.ok();
            }
        }
    }

    // 从数据库删除
    {
        let db = state.db.lock().await;
        db.delete_task(&id).map_err(|e| e.to_string())?;
    }

    info!("任务已删除: {}", id);
    Ok(())
}

/// 获取单个任务状态
#[tauri::command]
pub async fn get_task(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<TaskStatus>, String> {
    let db = state.db.lock().await;
    db.get_task(&id).map_err(|e| e.to_string())
}

/// 获取所有任务
#[tauri::command]
pub async fn get_all_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<TaskStatus>, String> {
    let db = state.db.lock().await;
    db.get_all_tasks().map_err(|e| e.to_string())
}

/// 设置任务优先级
#[tauri::command]
pub async fn set_priority(
    state: State<'_, AppState>,
    id: String,
    priority: i32,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.update_task_priority(&id, priority).map_err(|e| e.to_string())?;
    Ok(())
}

/// 打开文件所在目录（系统文件管理器）
#[tauri::command]
pub async fn open_file_location(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let task = db.get_task(&id).map_err(|e| e.to_string())?;
    drop(db);

    match task {
        Some(task) => {
            let save_path = std::path::Path::new(&task.save_path);
            // 规范化路径（处理相对路径和 .. 等）
            let save_dir = if save_path.is_absolute() {
                save_path.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(save_path)
            };
            let file_path = save_dir.join(&task.name);
            // 确定要打开的目录：如果文件存在则选中文件，否则打开目录
            let target = if file_path.exists() {
                file_path.clone()
            } else {
                save_dir.clone()
            };

            #[cfg(target_os = "windows")]
            {
                // 使用 /select 参数打开资源管理器并选中文件，复用已有窗口
                let select_arg = format!("/select,{}", target.to_string_lossy());
                std::process::Command::new("explorer")
                    .arg(select_arg)
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }

            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(save_dir.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(save_dir.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| format!("打开目录失败: {}", e))?;
            }

            info!("已打开目录: {:?}", save_dir);
            Ok(())
        }
        None => Err("任务不存在".to_string()),
    }
}

/// 获取下载历史
#[tauri::command]
pub async fn get_download_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<crate::storage::db::DownloadHistory>, String> {
    let db = state.db.lock().await;
    let limit = limit.unwrap_or(100);
    db.get_download_history(limit).map_err(|e| e.to_string())
}

/// 清空下载历史
#[tauri::command]
pub async fn clear_download_history(
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().await;
    db.clear_download_history().map_err(|e| e.to_string())
}

/// 暂停所有任务
#[tauri::command]
pub async fn pause_all_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let tasks = {
        let db = state.db.lock().await;
        db.get_all_tasks().map_err(|e| e.to_string())?
    };

    let mut paused_ids = Vec::new();

    // 先通过 manager 暂停任务（不持有 db 锁）
    {
        let mut manager = state.task_manager.lock().await;
        for task in &tasks {
            if task.state == TaskState::Downloading || task.state == TaskState::Queued {
                if let Err(e) = manager.pause_task(&task.id).await {
                    tracing::warn!("暂停任务失败: {} - {}", task.id, e);
                }
                paused_ids.push(task.id.clone());
            }
        }
    }

    // 再批量更新数据库状态
    if !paused_ids.is_empty() {
        let db = state.db.lock().await;
        for id in &paused_ids {
            if let Err(e) = db.update_task_state(id, &TaskState::Paused) {
                tracing::warn!("更新任务状态失败: {} - {}", id, e);
            }
        }
    }

    if !paused_ids.is_empty() {
        info!("已暂停 {} 个任务", paused_ids.len());
    }
    Ok(paused_ids)
}

/// 恢复所有暂停的任务
#[tauri::command]
pub async fn resume_all_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let tasks = {
        let db = state.db.lock().await;
        db.get_all_tasks().map_err(|e| e.to_string())?
    };

    let mut resumed_ids = Vec::new();

    // 收集需要恢复的任务，分为两类（批量获取 manager 锁，避免逐个锁定的 TOCTOU 竞争）
    let mut to_resume: Vec<String> = Vec::new();
    let mut to_read: Vec<&TaskStatus> = Vec::new();

    {
        let manager = state.task_manager.lock().await;
        for task in &tasks {
            if task.state == TaskState::Paused {
                if manager.has_task(&task.id) {
                    to_resume.push(task.id.clone());
                } else {
                    to_read.push(task);
                }
            }
        }
    }

    // 恢复仍在 manager 中跟踪的任务（不持有 db 锁）
    if !to_resume.is_empty() {
        let mut manager = state.task_manager.lock().await;
        for id in &to_resume {
            if let Err(e) = manager.resume_task(id).await {
                tracing::warn!("恢复任务失败: {} - {}", id, e);
            }
        }
        resumed_ids.extend(to_resume);
    }

    // 重新添加已丢失句柄的任务（不持有 db 锁）
    if !to_read.is_empty() {
        let mut manager = state.task_manager.lock().await;
        for task in to_read {
            let params = TaskParams {
                url: task.url.clone(),
                save_path: task.save_path.clone(),
                file_name: Some(task.name.clone()),
                proxy: None,
                speed_limit: None,
                start_immediately: true,
                only_files: None,
                mirror_urls: None,
                http_auth: None,
                http_cookie: None,
                http_headers: None,
            };
            match manager.add_task(task.clone(), &params).await {
                Ok(_) => {
                    resumed_ids.push(task.id.clone());
                }
                Err(e) => {
                    warn!("恢复任务失败: {} - {}", task.name, e);
                }
            }
        }
    }

    // 批量更新数据库状态
    if !resumed_ids.is_empty() {
        let db = state.db.lock().await;
        for id in &resumed_ids {
            let _ = db.update_task_state(id, &TaskState::Downloading);
        }
    }

    if !resumed_ids.is_empty() {
        info!("已恢复 {} 个任务", resumed_ids.len());
    }
    Ok(resumed_ids)
}

/// 删除所有已完成/出错的任务
#[tauri::command]
pub async fn remove_completed_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().await;
    let tasks = db.get_all_tasks().map_err(|e| e.to_string())?;
    drop(db);

    let mut removed_ids = Vec::new();
    for task in &tasks {
        if task.state == TaskState::Done || task.state == TaskState::Error {
            removed_ids.push(task.id.clone());
        }
    }

    if !removed_ids.is_empty() {
        let db = state.db.lock().await;
        for id in &removed_ids {
            if let Err(e) = db.delete_task(id) {
                tracing::warn!("删除任务失败: {} - {}", id, e);
            }
        }
    }

    if !removed_ids.is_empty() {
        info!("已清理 {} 个已完成任务", removed_ids.len());
    }
    Ok(removed_ids)
}

/// 程序启动时恢复未完成的任务
#[tauri::command]
pub async fn resume_unfinished_tasks(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().await;
    let unfinished = db.get_unfinished_tasks().map_err(|e| e.to_string())?;
    drop(db);

    let mut resumed_ids = Vec::new();

    // 通过 manager 恢复任务（不持有 db 锁）
    {
        let mut manager = state.task_manager.lock().await;
        let db = state.db.lock().await;
        for task in unfinished {
            if task.state == TaskState::Downloading || task.state == TaskState::Queued {
                // 从数据库读取持久化的下载参数
                let params = db.get_task_params(&task.id)
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| TaskParams {
                        url: task.url.clone(),
                        save_path: task.save_path.clone(),
                        file_name: Some(task.name.clone()),
                        proxy: None,
                        speed_limit: None,
                        start_immediately: true,
                        only_files: None,
                        mirror_urls: None,
                        http_auth: None,
                        http_cookie: None,
                        http_headers: None,
                    });

                match manager.add_task(task.clone(), &params).await {
                    Ok(_) => {
                        resumed_ids.push(task.id.clone());
                        info!("已恢复任务: {} ({})", task.name, task.id);
                    }
                    Err(e) => {
                        warn!("恢复任务失败: {} - {}", task.name, e);
                    }
                }
            }
        }
    }

    // 批量更新数据库状态（manager 锁已释放）
    if !resumed_ids.is_empty() {
        let db = state.db.lock().await;
        for id in &resumed_ids {
            if let Err(e) = db.update_task_state(id, &TaskState::Downloading) {
                tracing::warn!("更新恢复任务状态失败: {} - {}", id, e);
            }
        }
    }

    Ok(resumed_ids)
}

/// Peer 连接信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerInfo {
    pub id: String,
    pub ip: String,
    pub port: u16,
    pub download_speed: u64,
    pub upload_speed: u64,
    pub client: String,
    pub progress: f32,
}

/// Tracker 信息
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackerInfo {
    pub url: String,
    pub status: String,
    pub peers: u32,
    pub seeders: u32,
    pub leechers: u32,
}

/// 日志条目
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// 获取任务的 Peer 连接列表
#[tauri::command]
pub async fn get_task_peers(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<PeerInfo>, String> {
    let manager = state.task_manager.lock().await;
    let peers = manager.get_task_peers(&id).await;
    Ok(peers)
}

/// 获取任务的 Tracker 列表
#[tauri::command]
pub async fn get_task_trackers(
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<TrackerInfo>, String> {
    let manager = state.task_manager.lock().await;
    let trackers = manager.get_task_trackers(&id).await;
    Ok(trackers)
}

/// 获取任务日志
#[tauri::command]
pub async fn get_task_logs(
    state: State<'_, AppState>,
    id: String,
    limit: Option<usize>,
) -> Result<Vec<LogEntry>, String> {
    let manager = state.task_manager.lock().await;
    let limit = limit.unwrap_or(100);
    let logs = manager.get_task_logs(&id, limit).await;
    Ok(logs)
}

/// 打开文件（使用系统默认程序）
#[tauri::command]
pub async fn open_file(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    let task = db.get_task(&id).map_err(|e| e.to_string())?;
    drop(db);

    match task {
        Some(task) => {
            let file_path = std::path::Path::new(&task.save_path).join(&task.name);

            if !file_path.exists() {
                return Err(format!("文件不存在: {:?}", file_path));
            }

            // 使用系统默认程序打开文件，避免 shell 命令注入风险
            #[cfg(target_os = "windows")]
            {
                // 使用 explorer 直接打开文件，不经过 cmd.exe
                std::process::Command::new("explorer")
                    .arg(file_path.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| format!("打开文件失败: {}", e))?;
            }

            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("open")
                    .arg(file_path.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| format!("打开文件失败: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            {
                std::process::Command::new("xdg-open")
                    .arg(file_path.to_string_lossy().to_string())
                    .spawn()
                    .map_err(|e| format!("打开文件失败: {}", e))?;
            }

            info!("已打开文件: {:?}", file_path);
            Ok(())
        }
        None => Err("任务不存在".to_string()),
    }
}

/// 设置单任务速度限制
/// download_limit 和 upload_limit 单位为 bytes/sec，0 表示不限速
#[tauri::command]
pub async fn set_task_speed_limit(
    state: State<'_, AppState>,
    id: String,
    download_limit: u64,
    upload_limit: u64,
) -> Result<(), String> {
    // 更新数据库中的速度限制
    {
        let db = state.db.lock().await;
        db.update_task_speed_limit(&id, download_limit, upload_limit).map_err(|e| e.to_string())?;
    }

    info!("任务 {} 速度限制已更新: 下载={} 上传={} bytes/sec", id, download_limit, upload_limit);
    Ok(())
}

/// 获取种子文件列表（不启动下载）
/// 支持 magnet 链接和 .torrent 文件路径
#[tauri::command]
pub async fn get_torrent_files(
    state: State<'_, AppState>,
    source: String,
) -> Result<crate::engine::bt::TorrentFileListResponse, String> {
    let mut manager = state.task_manager.lock().await;
    manager.get_torrent_file_list(&source).await.map_err(|e| e.to_string())
}

/// 添加 BT/Magnet 任务（支持文件选择）
#[tauri::command]
pub async fn add_bt_task_with_files(
    state: State<'_, AppState>,
    params: TaskParams,
    only_files: Option<Vec<usize>>,
    torrent_files: Option<Vec<crate::engine::bt::TorrentFileInfo>>,
) -> Result<String, String> {
    let task_id = uuid::Uuid::new_v4().to_string();

    // 自动检测协议类型
    let protocol = match util::detect_protocol(&params.url) {
        "MAGNET" => Protocol::Magnet,
        "BT" => Protocol::Bt,
        _ => return Err("不支持的协议类型，仅支持 magnet 和 torrent 文件".to_string()),
    };

    // 提取并验证文件名
    let raw_file_name = params.file_name.clone().unwrap_or_else(|| {
        util::extract_filename(&params.url)
    });
    let file_name = util::validate_filename(&raw_file_name)
        .unwrap_or_else(|_| format!("download_{}", &task_id[..8]));

    // 确定保存路径
    let save_path = if params.save_path.is_empty() {
        let config = state.config.lock().await;
        config.download.default_dir.clone()
    } else {
        params.save_path.clone()
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let task = TaskStatus {
        id: task_id.clone(),
        name: file_name,
        protocol,
        state: if params.start_immediately { TaskState::Downloading } else { TaskState::Queued },
        url: params.url.clone(),
        save_path,
        total_size: 0,
        downloaded: 0,
        uploaded: 0,
        download_speed: 0,
        upload_speed: 0,
        progress: 0.0,
        peers: 0,
        seeds: 0,
        eta: None,
        files: Vec::new(),
        error: None,
        added_at: now,
        completed_at: None,
        priority: 1,
        download_limit: 0,
        upload_limit: 0,
    };

    // 保存到数据库
    {
        let db = state.db.lock().await;
        db.insert_task(&task).map_err(|e| e.to_string())?;
        // 保存种子文件列表到数据库（用于重启后恢复 FilesTab）
        if let Some(ref files) = torrent_files {
            let file_infos: Vec<crate::storage::db::FileInfo> = files.iter().map(|f| {
                crate::storage::db::FileInfo {
                    index: f.index as i32,
                    path: f.name.clone(),
                    size: f.size,
                    priority: if only_files.as_ref().map_or(true, |sel| sel.contains(&f.index)) { 1 } else { 0 },
                }
            }).collect();
            if let Err(e) = db.insert_task_files(&task_id, &file_infos) {
                tracing::warn!("保存种子文件列表失败: {} - {}", task_id, e);
            }
        }
    }

    // 启动下载（带文件选择）
    if params.start_immediately {
        let mut manager = state.task_manager.lock().await;
        manager.add_bt_task_with_files(task.clone(), &params, only_files).await.map_err(|e| e.to_string())?;
    }

    info!("BT 任务已添加: {} ({})", task.name, task_id);
    Ok(task_id)
}

/// 批量添加任务（支持多链接）
#[tauri::command]
pub async fn batch_add_tasks(
    state: State<'_, AppState>,
    urls: Vec<String>,
    save_path: Option<String>,
    start_immediately: bool,
) -> Result<Vec<String>, String> {
    let mut task_ids = Vec::new();

    // 在循环外获取默认保存路径，避免重复加锁
    let default_save_path = if let Some(ref path) = save_path {
        path.clone()
    } else {
        let config = state.config.lock().await;
        config.download.default_dir.clone()
    };

    for url in urls {
        let task_id = uuid::Uuid::new_v4().to_string();

        // 自动检测协议类型
        let protocol = match util::detect_protocol(&url) {
            "MAGNET" => Protocol::Magnet,
            "ED2K" => Protocol::Ed2k,
            "FTP" => Protocol::Ftp,
            "BT" => Protocol::Bt,
            _ => Protocol::Http,
        };

        // 提取并验证文件名
        let raw_file_name = util::extract_filename(&url);
        let file_name = util::validate_filename(&raw_file_name)
            .unwrap_or_else(|_| format!("download_{}", &task_id[..8]));

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let task = TaskStatus {
            id: task_id.clone(),
            name: file_name,
            protocol,
            state: if start_immediately { TaskState::Downloading } else { TaskState::Queued },
            url: url.clone(),
            save_path: default_save_path.clone(),
            total_size: 0,
            downloaded: 0,
            uploaded: 0,
            download_speed: 0,
            upload_speed: 0,
            progress: 0.0,
            peers: 0,
            seeds: 0,
            eta: None,
            files: Vec::new(),
            error: None,
            added_at: now,
            completed_at: None,
            priority: 1,
            download_limit: 0,
            upload_limit: 0,
        };

        // 保存到数据库
        {
            let db = state.db.lock().await;
            if let Err(e) = db.insert_task(&task) {
                warn!("保存任务失败: {} - {}", url, e);
                continue;
            }
        }

        // 启动下载
        if start_immediately {
            let params = TaskParams {
                url: url.clone(),
                save_path: task.save_path.clone(),
                file_name: Some(task.name.clone()),
                proxy: None,
                speed_limit: None,
                start_immediately: true,
                only_files: None,
                mirror_urls: None,
                http_auth: None,
                http_cookie: None,
                http_headers: None,
            };

            let mut manager = state.task_manager.lock().await;
            if let Err(e) = manager.add_task(task.clone(), &params).await {
                warn!("启动任务失败: {} - {}", url, e);
                continue;
            }
        }

        task_ids.push(task_id);
    }

    info!("批量添加了 {} 个任务", task_ids.len());
    Ok(task_ids)
}

/// 添加镜像URL到现有任务
#[tauri::command]
pub async fn add_mirror_url(
    state: State<'_, AppState>,
    task_id: String,
    mirror_url: String,
) -> Result<(), String> {
    // 验证URL格式
    if !mirror_url.starts_with("http://") && !mirror_url.starts_with("https://") {
        return Err("镜像URL必须以 http:// 或 https:// 开头".to_string());
    }

    let db = state.db.lock().await;
    db.add_mirror_url(&task_id, &mirror_url).map_err(|e| e.to_string())?;
    info!("已添加镜像URL到任务 {}: {}", task_id, mirror_url);
    Ok(())
}

/// 移除任务的镜像URL
#[tauri::command]
pub async fn remove_mirror_url(
    state: State<'_, AppState>,
    task_id: String,
    mirror_url: String,
) -> Result<(), String> {
    let db = state.db.lock().await;
    db.remove_mirror_url(&task_id, &mirror_url).map_err(|e| e.to_string())?;
    info!("已从任务 {} 移除镜像URL: {}", task_id, mirror_url);
    Ok(())
}

/// 获取任务的镜像URL列表
#[tauri::command]
pub async fn get_mirror_urls(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<String>, String> {
    let db = state.db.lock().await;
    db.get_mirror_urls(&task_id).map_err(|e| e.to_string())
}
