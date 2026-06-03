// JSON-RPC 2.0 方法分发模块
// 实现 aria2 兼容 API 和自定义 API

use super::{JsonRpcRequest, JsonRpcResponse};
use crate::AppState;
use serde_json::{json, Value};

/// 分发 JSON-RPC 方法
pub async fn dispatch_method(request: JsonRpcRequest, state: &AppState) -> JsonRpcResponse {
    let id = request.id.clone();

    match request.method.as_str() {
        // ==================== 任务管理 ====================
        "downloader.addTask" | "aria2.addUri" => {
            handle_add_task(id, request.params, state).await
        }
        "downloader.removeTask" | "aria2.remove" => {
            handle_remove_task(id, request.params, state).await
        }
        "downloader.pauseTask" | "aria2.pause" => {
            handle_pause_task(id, request.params, state).await
        }
        "downloader.resumeTask" | "aria2.unpause" => {
            handle_resume_task(id, request.params, state).await
        }

        // ==================== 状态查询 ====================
        "downloader.tellStatus" | "aria2.tellStatus" => {
            handle_tell_status(id, request.params, state).await
        }
        "downloader.tellActive" | "aria2.tellActive" => {
            handle_tell_active(id, state).await
        }
        "downloader.tellWaiting" | "aria2.tellWaiting" => {
            handle_tell_waiting(id, state).await
        }
        "downloader.tellStopped" | "aria2.tellStopped" => {
            handle_tell_stopped(id, state).await
        }

        // ==================== 全局信息 ====================
        "downloader.getGlobalStat" | "aria2.getGlobalStat" => {
            handle_get_global_stat(id, state).await
        }
        "downloader.getVersion" | "aria2.getVersion" => {
            handle_get_version(id).await
        }

        // ==================== 批量操作 ====================
        "downloader.pauseAll" | "aria2.pauseAll" => {
            handle_pause_all(id, state).await
        }
        "downloader.unpauseAll" | "aria2.unpauseAll" => {
            handle_unpause_all(id, state).await
        }
        "downloader.purgeCompleted" | "aria2.purgeDownloadResult" => {
            handle_purge_completed(id, state).await
        }

        // ==================== 未知方法 ====================
        _ => JsonRpcResponse::error(
            id,
            -32601,
            &format!("未知方法: {}", request.method),
        ),
    }
}

/// 解析 params 数组中的 task_id
fn extract_task_id(params: &Value) -> &str {
    params
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .unwrap_or("")
}

/// 添加任务（通过 Tauri 命令层）
async fn handle_add_task(id: Option<Value>, params: Value, state: &AppState) -> JsonRpcResponse {
    let urls: Vec<String> = if let Some(arr) = params.as_array() {
        arr.first()
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    if urls.is_empty() {
        return JsonRpcResponse::error(id, -32602, "缺少 URL 参数");
    }

    let url = &urls[0];
    let config = state.config.lock().await;
    let save_path = config.download.default_dir.clone();
    drop(config);

    // Create task via Tauri command
    let task_id = uuid::Uuid::new_v4().to_string();
    let params = crate::storage::db::TaskParams {
        url: url.clone(),
        save_path,
        file_name: None,
        proxy: None,
        speed_limit: None,
        start_immediately: true,
        only_files: None,
        mirror_urls: None,
        http_auth: None,
        http_cookie: None,
        http_headers: None,
    };

    // Create TaskStatus for the task manager
    let raw_name = crate::util::extract_filename(url);
    let safe_name = crate::util::validate_filename(&raw_name)
        .unwrap_or_else(|_| "download".to_string());
    let task_status = crate::storage::db::TaskStatus {
        id: task_id.clone(),
        name: safe_name,
        protocol: crate::storage::db::Protocol::from_str(crate::util::detect_protocol(url)),
        url: url.clone(),
        save_path: params.save_path.clone(),
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
        state: crate::storage::db::TaskState::Queued,
        error: None,
        priority: 1,
        added_at: chrono::Utc::now().to_rfc3339(),
        completed_at: None,
        download_limit: 0,
        upload_limit: 0,
    };

    // Save to DB
    let db = state.db.lock().await;
    if let Err(e) = db.insert_task(&task_status) {
        return JsonRpcResponse::error(id, -32603, &format!("保存任务失败: {}", e));
    }
    drop(db);

    // Add to task manager
    let mut manager = state.task_manager.lock().await;
    match manager.add_task(task_status, &params).await {
        Ok(_) => JsonRpcResponse::success(id, json!(task_id)),
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("添加任务失败: {}", e)),
    }
}

/// 删除任务
async fn handle_remove_task(id: Option<Value>, params: Value, state: &AppState) -> JsonRpcResponse {
    let task_id = extract_task_id(&params);
    if task_id.is_empty() {
        return JsonRpcResponse::error(id, -32602, "缺少任务 ID");
    }

    let mut manager = state.task_manager.lock().await;
    match manager.remove_task(task_id).await {
        Ok(_) => {
            drop(manager);
            let db = state.db.lock().await;
            if let Err(e) = db.delete_task(task_id) {
                tracing::warn!("RPC 删除任务 DB 记录失败: {} - {}", task_id, e);
            }
            JsonRpcResponse::success(id, json!(task_id))
        }
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("删除任务失败: {}", e)),
    }
}

/// 暂停任务
async fn handle_pause_task(id: Option<Value>, params: Value, state: &AppState) -> JsonRpcResponse {
    let task_id = extract_task_id(&params);
    if task_id.is_empty() {
        return JsonRpcResponse::error(id, -32602, "缺少任务 ID");
    }

    let mut manager = state.task_manager.lock().await;
    match manager.pause_task(task_id).await {
        Ok(_) => {
            drop(manager);
            let db = state.db.lock().await;
            if let Err(e) = db.update_task_state(task_id, &crate::storage::db::TaskState::Paused) {
                tracing::warn!("RPC 暂停任务 DB 状态更新失败: {} - {}", task_id, e);
            }
            JsonRpcResponse::success(id, json!(task_id))
        }
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("暂停任务失败: {}", e)),
    }
}

/// 恢复任务
async fn handle_resume_task(id: Option<Value>, params: Value, state: &AppState) -> JsonRpcResponse {
    let task_id = extract_task_id(&params);
    if task_id.is_empty() {
        return JsonRpcResponse::error(id, -32602, "缺少任务 ID");
    }

    let mut manager = state.task_manager.lock().await;
    match manager.resume_task(task_id).await {
        Ok(_) => {
            drop(manager);
            let db = state.db.lock().await;
            if let Err(e) = db.update_task_state(task_id, &crate::storage::db::TaskState::Downloading) {
                tracing::warn!("RPC 恢复任务 DB 状态更新失败: {} - {}", task_id, e);
            }
            JsonRpcResponse::success(id, json!(task_id))
        }
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("恢复任务失败: {}", e)),
    }
}

/// 查询任务状态
async fn handle_tell_status(id: Option<Value>, params: Value, state: &AppState) -> JsonRpcResponse {
    let task_id = extract_task_id(&params);
    if task_id.is_empty() {
        return JsonRpcResponse::error(id, -32602, "缺少任务 ID");
    }

    let db = state.db.lock().await;
    match db.get_task(task_id) {
        Ok(Some(task)) => JsonRpcResponse::success(id, json!({
            "taskId": task.id,
            "status": format!("{:?}", task.state).to_lowercase(),
            "totalLength": task.total_size,
            "completedLength": task.downloaded,
            "downloadSpeed": task.download_speed,
            "uploadSpeed": task.upload_speed,
            "progress": task.progress,
            "peers": task.peers,
            "error": task.error,
        })),
        Ok(None) => JsonRpcResponse::error(id, -32603, &format!("任务不存在: {}", task_id)),
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("查询失败: {}", e)),
    }
}

/// 查询活跃任务（使用索引查询，避免全表扫描）
async fn handle_tell_active(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    match db.get_tasks_by_state(&crate::storage::db::TaskState::Downloading) {
        Ok(tasks) => {
            let items: Vec<Value> = tasks
                .iter()
                .map(|t| {
                    json!({
                        "taskId": t.id,
                        "status": "active",
                        "totalLength": t.total_size,
                        "completedLength": t.downloaded,
                        "downloadSpeed": t.download_speed,
                    })
                })
                .collect();
            JsonRpcResponse::success(id, json!(items))
        }
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("查询失败: {}", e)),
    }
}

/// 查询等待中的任务
async fn handle_tell_waiting(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    match db.get_tasks_by_state(&crate::storage::db::TaskState::Queued) {
        Ok(tasks) => {
            let items: Vec<Value> = tasks
                .iter()
                .map(|t| json!({ "taskId": t.id, "status": "waiting" }))
                .collect();
            JsonRpcResponse::success(id, json!(items))
        }
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("查询失败: {}", e)),
    }
}

/// 查询已停止的任务
async fn handle_tell_stopped(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    // 使用索引查询 Done 和 Error 状态，避免全表扫描
    let mut items: Vec<Value> = Vec::new();
    if let Ok(tasks) = db.get_tasks_by_state(&crate::storage::db::TaskState::Done) {
        items.extend(tasks.iter().map(|t| json!({ "taskId": t.id, "status": "complete", "totalLength": t.total_size })));
    }
    if let Ok(tasks) = db.get_tasks_by_state(&crate::storage::db::TaskState::Error) {
        items.extend(tasks.iter().map(|t| json!({ "taskId": t.id, "status": "error", "totalLength": t.total_size })));
    }
    JsonRpcResponse::success(id, json!(items))
}

/// 获取全局统计（使用 SQL 聚合避免全表扫描）
async fn handle_get_global_stat(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    // 使用 SQL 聚合获取任务计数
    let stats = db.get_task_stats().unwrap_or((0, 0, 0, 0, 0, 0));
    let (queued, downloading, _paused, seeding, done, error) = stats;

    // 仅对活跃任务获取速度（使用索引查询）
    let mut download_speed = 0u64;
    let mut upload_speed = 0u64;
    if let Ok(active) = db.get_tasks_by_state(&crate::storage::db::TaskState::Downloading) {
        for t in &active {
            download_speed += t.download_speed;
            upload_speed += t.upload_speed;
        }
    }
    // 也包含 seeding 任务的速度
    if let Ok(seeders) = db.get_tasks_by_state(&crate::storage::db::TaskState::Seeding) {
        for t in &seeders {
            download_speed += t.download_speed;
            upload_speed += t.upload_speed;
        }
    }

    JsonRpcResponse::success(
        id,
        json!({
            "downloadSpeed": download_speed,
            "uploadSpeed": upload_speed,
            "numActive": downloading + seeding,
            "numWaiting": queued,
            "numStopped": done + error,
        }),
    )
}

/// 获取版本
async fn handle_get_version(id: Option<Value>) -> JsonRpcResponse {
    JsonRpcResponse::success(
        id,
        json!({
            "version": env!("CARGO_PKG_VERSION"),
            "enabledFeatures": ["HTTP", "FTP", "BT", "MAGNET", "ED2K"],
        }),
    )
}

/// 暂停所有任务
async fn handle_pause_all(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    let tasks = db.get_all_tasks().unwrap_or_default();
    drop(db);

    let mut manager = state.task_manager.lock().await;
    let mut paused = 0;
    for task in &tasks {
        if task.state == crate::storage::db::TaskState::Downloading {
            if manager.pause_task(&task.id).await.is_ok() {
                paused += 1;
            }
        }
    }
    JsonRpcResponse::success(id, json!({ "paused": paused }))
}

/// 恢复所有任务
async fn handle_unpause_all(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    let tasks = db.get_all_tasks().unwrap_or_default();
    drop(db);

    let mut manager = state.task_manager.lock().await;
    let mut resumed = 0;
    for task in &tasks {
        if task.state == crate::storage::db::TaskState::Paused {
            if manager.resume_task(&task.id).await.is_ok() {
                resumed += 1;
            }
        }
    }
    JsonRpcResponse::success(id, json!({ "resumed": resumed }))
}

/// 清除已完成和出错任务（单条 SQL 批量删除）
async fn handle_purge_completed(id: Option<Value>, state: &AppState) -> JsonRpcResponse {
    let db = state.db.lock().await;
    match db.purge_completed_tasks() {
        Ok(purged) => JsonRpcResponse::success(id, json!({ "purged": purged })),
        Err(e) => JsonRpcResponse::error(id, -32603, &format!("清除失败: {}", e)),
    }
}
