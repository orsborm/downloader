// 全协议下载器 - Tauri 应用入口
// 避免 Windows 控制台弹窗
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod archive;
mod commands;
mod engine;
mod plugin;
mod rss;
mod schedule;
mod storage;
mod util;

use api::{ApiConfig, ApiService};
use api::websocket::WsManager;
use archive::ArchiveManager;
use engine::task_manager::TaskManager;
use plugin::PluginManager;
use rss::RssEngine;
use schedule::ScheduleManager;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use storage::db::{Database, TaskState};
use storage::config::AppConfig;
use tauri::{Manager, Emitter};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

/// 应用全局状态，通过 Tauri 管理（Manage）共享给所有 IPC 命令
pub struct AppState {
    pub db: Arc<TokioMutex<Database>>,
    pub config: Arc<TokioMutex<AppConfig>>,
    pub config_path: std::path::PathBuf,
    pub task_manager: Arc<TokioMutex<TaskManager>>,
    pub rss_engine: Arc<TokioMutex<RssEngine>>,
    pub archive_manager: Arc<TokioMutex<ArchiveManager>>,
    pub plugin_manager: Arc<TokioMutex<PluginManager>>,
    pub schedule_manager: Arc<TokioMutex<ScheduleManager>>,
    pub ws_manager: Arc<WsManager>,
}

/// 执行下载完成后的动作
fn execute_post_action(action: &str, command: &str) {
    match action {
        "shutdown" => {
            info!("所有下载完成，执行关机...");
            #[cfg(target_os = "windows")]
            { let _ = std::process::Command::new("shutdown").args(["/s", "/t", "60"]).spawn(); }
            #[cfg(target_os = "linux")]
            { let _ = std::process::Command::new("shutdown").args(["-h", "+1"]).spawn(); }
            #[cfg(target_os = "macos")]
            { let _ = std::process::Command::new("osascript").args(["-e", "tell app \"System Events\" to shut down"]).spawn(); }
        }
        "hibernate" => {
            info!("所有下载完成，执行休眠...");
            #[cfg(target_os = "windows")]
            { let _ = std::process::Command::new("rundll32.exe").args(["powrprof.dll,SetSuspendState", "0,1,0"]).spawn(); }
            #[cfg(target_os = "linux")]
            { let _ = std::process::Command::new("systemctl").args(["hibernate"]).spawn(); }
            #[cfg(target_os = "macos")]
            { let _ = std::process::Command::new("pmset").args(["-a", "hibernatemode", "25"]).spawn(); }
        }
        "sleep" => {
            info!("所有下载完成，执行睡眠...");
            #[cfg(target_os = "windows")]
            { let _ = std::process::Command::new("rundll32.exe").args(["powrprof.dll,SetSuspendState", "0,0,0"]).spawn(); }
            #[cfg(target_os = "linux")]
            { let _ = std::process::Command::new("systemctl").args(["suspend"]).spawn(); }
            #[cfg(target_os = "macos")]
            { let _ = std::process::Command::new("pmset").args(["sleepnow"]).spawn(); }
        }
        "run_command" => {
            if !command.is_empty() {
                // 安全验证：白名单方式——仅允许字母、数字、空格和有限安全符号
                // 阻断所有 shell 元字符：|;&$`(){}<>!\"'#*?\%~ 以及控制字符
                let is_safe_char = |c: char| -> bool {
                    c.is_ascii_alphanumeric() || " .-_/:=@[],+~".contains(c)
                };
                if !command.chars().all(is_safe_char) {
                    warn!("⚠️ 拒绝执行包含非法字符的命令: {}", command);
                } else if command.len() > 500 {
                    warn!("⚠️ 拒绝执行过长的命令（{} 字符）", command.len());
                } else {
                    warn!("⚠️ 执行用户自定义命令: {}", command);
                    #[cfg(target_os = "windows")]
                    { if let Err(e) = std::process::Command::new("cmd").args(["/C", command]).spawn() {
                        warn!("执行命令失败: {}", e);
                    }}
                    #[cfg(not(target_os = "windows"))]
                    { if let Err(e) = std::process::Command::new("sh").args(["-c", command]).spawn() {
                        warn!("执行命令失败: {}", e);
                    }}
                }
            }
        }
        _ => {} // "none" 或其他值不执行
    }
}

fn main() {
    // 初始化日志系统
    init_logging();

    info!("全协议下载器启动中...");

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard::init())
        .setup(|app| {
            // 便携模式：数据存储在 EXE 同级 data/ 目录，关闭无残留
            let app_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("data")))
                .unwrap_or_else(|| std::path::PathBuf::from("data"));

            // 确保数据目录存在
            std::fs::create_dir_all(&app_dir).ok();

            // 初始化数据库
            let db_path = app_dir.join("downloader.db");
            // 使用 ? 传播错误，避免 panic 导致无提示崩溃
            let db = Database::new(&db_path).map_err(|e| {
                tracing::error!("数据库初始化失败: {}", e);
                format!("数据库初始化失败: {}", e)
            })?;

            // 初始化配置
            let config_path = app_dir.join("config.toml");
            let config = AppConfig::load_or_default(&config_path);

            // 构建 BT 引擎配置
            let bt_config = engine::bt::BtConfig {
                listen_port: config.connection.bt_port,
                dht: config.bt.dht,
                pex: config.bt.pex,
                lsd: config.bt.lsd,
                encryption: config.bt.encryption.clone(),
                seed_ratio_limit: config.bt.seed_ratio_limit,
                seed_time_limit: config.bt.seed_time_limit,
                max_download_speed: config.download.max_download_speed,
                max_upload_speed: config.download.max_upload_speed,
                stop_seeding: config.bt.stop_seeding,
            };

            // 初始化任务管理器
            let mut task_manager = TaskManager::new(
                config.download.max_concurrent_tasks,
                config.download.max_upload_speed,
                config.download.max_download_speed,
                bt_config,
            );
            task_manager.set_retry_config(
                config.download.auto_retry_count,
                config.download.auto_retry_interval,
            );
            // 传递 ed2k 自定义服务器列表
            task_manager.set_ed2k_servers(config.connection.ed2k_servers.clone());

            // 启动 KAD 网络（使用 ed2k 服务器作为引导节点）
            tauri::async_runtime::block_on(async {
                task_manager.start_kad().await;
            });

            // 初始化 BT 引擎
            let default_dir = config.download.default_dir.clone();
            tauri::async_runtime::block_on(async {
                if let Err(e) = task_manager.init_bt_engine(&default_dir).await {
                    warn!("BT 引擎初始化失败: {}（BT 下载功能不可用）", e);
                }
            });

            // 初始化插件管理器
            let plugin_dir = app_dir.join("plugins");
            let mut plugin_manager = PluginManager::new(plugin_dir);
            if let Err(e) = plugin_manager.scan_plugins() {
                warn!("扫描插件目录失败: {}", e);
            }

            // 提取 API 服务器配置（在 config 被 move 之前）
            let api_port = config.connection.http_port;
            let api_host = "127.0.0.1".to_string();
            let api_token = if config.connection.api_token.is_empty() {
                None
            } else {
                Some(config.connection.api_token.clone())
            };

            // 初始化 WebSocket 管理器
            let ws_manager = Arc::new(WsManager::new());

            // 从数据库加载持久化的 RSS 订阅
            let rss_feeds = match db.load_rss_feeds() {
                Ok(feeds) => {
                    info!("从数据库加载了 {} 个 RSS 订阅", feeds.len());
                    feeds
                }
                Err(e) => {
                    warn!("加载 RSS 订阅失败: {}", e);
                    Vec::new()
                }
            };
            let mut rss_engine = RssEngine::new();
            for feed in rss_feeds {
                rss_engine.add_feed(feed);
            }

            // 从数据库加载持久化的调度规则
            let schedule_rules = match db.load_schedule_rules() {
                Ok(rules) => {
                    info!("从数据库加载了 {} 个调度规则", rules.len());
                    rules
                }
                Err(e) => {
                    warn!("加载调度规则失败: {}", e);
                    Vec::new()
                }
            };
            let mut schedule_manager = ScheduleManager::new();
            for rule in schedule_rules {
                if let Err(e) = schedule_manager.add_rule(rule) {
                    warn!("恢复调度规则失败: {}", e);
                }
            }
            // 从数据库加载持久化的带宽计划
            match db.load_bandwidth_schedules() {
                Ok(schedules) => {
                    info!("从数据库加载了 {} 个带宽计划", schedules.len());
                    for s in schedules {
                        if let Err(e) = schedule_manager.add_bandwidth_schedule(s) {
                            warn!("恢复带宽计划失败: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("加载带宽计划失败: {}", e);
                }
            }

            // 从数据库加载持久化的解压密码
            let archive_passwords = match db.load_archive_passwords() {
                Ok(passwords) => {
                    info!("从数据库加载了 {} 个解压密码", passwords.len());
                    passwords
                }
                Err(e) => {
                    warn!("加载解压密码失败: {}", e);
                    Vec::new()
                }
            };
            let mut archive_config = archive::ArchiveConfig::default();
            archive_config.passwords = archive_passwords;

            // 将全局状态注册到 Tauri
            let state = AppState {
                db: Arc::new(TokioMutex::new(db)),
                config: Arc::new(TokioMutex::new(config)),
                config_path: config_path.clone(),
                task_manager: Arc::new(TokioMutex::new(task_manager)),
                rss_engine: Arc::new(TokioMutex::new(rss_engine)),
                archive_manager: Arc::new(TokioMutex::new(ArchiveManager::new(archive_config))),
                plugin_manager: Arc::new(TokioMutex::new(plugin_manager)),
                schedule_manager: Arc::new(TokioMutex::new(schedule_manager)),
                ws_manager: ws_manager.clone(),
            };
            app.manage(state);

            // 取出事件接收端，启动事件转发到前端和 DB 持久化
            let state_ref = app.state::<AppState>();
            let app_handle = app.handle().clone();

            // 启动 JSON-RPC API 服务器
            if api_port > 0 {
                let api_config = ApiConfig {
                    host: api_host,
                    port: api_port,
                    token: api_token,
                    allow_remote: false,
                };
                let api_service = ApiService::new(api_config);
                let api_state = Arc::new(AppState {
                    db: state_ref.db.clone(),
                    config: state_ref.config.clone(),
                    config_path: state_ref.config_path.clone(),
                    task_manager: state_ref.task_manager.clone(),
                    rss_engine: state_ref.rss_engine.clone(),
                    archive_manager: state_ref.archive_manager.clone(),
                    plugin_manager: state_ref.plugin_manager.clone(),
                    schedule_manager: state_ref.schedule_manager.clone(),
                    ws_manager: state_ref.ws_manager.clone(),
                });
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = api_service.start_http(api_state).await {
                        tracing::error!("JSON-RPC API 启动失败: {}", e);
                    }
                });
                info!("JSON-RPC API 服务器将在端口 {} 启动", api_port);
            } else {
                info!("JSON-RPC API 服务器已禁用 (http_port=0)");
            }

            // 任务状态更新 → Tauri 事件 + DB 持久化 + WebSocket 广播 + 下载后动作
            {
                let mut manager = state_ref.task_manager.blocking_lock();
                if let Some(mut update_rx) = manager.take_update_receiver() {
                    let handle = app_handle.clone();
                    let db_ref = state_ref.db.clone();
                    let ws = state_ref.ws_manager.clone();
                    let config_ref = state_ref.config.clone();
                    let tm_ref = state_ref.task_manager.clone();
                    // 用于定期保存进度到 DB（每 10 秒）
                    let mut last_db_save: std::collections::HashMap<String, std::time::Instant> = std::collections::HashMap::new();
                    tauri::async_runtime::spawn(async move {
                        while let Some(event) = update_rx.recv().await {
                            // 发送 Tauri 事件到前端（记录错误，防止 Done 事件丢失）
                            if let Err(e) = handle.emit("task-update", &event) {
                                tracing::error!("发送任务更新事件失败 {}: {}", event.task_id, e);
                            }

                            // WebSocket 广播给 API 客户端
                            let api_event = match &event.state {
                                TaskState::Done => api::ApiEvent::TaskCompleted {
                                    task_id: event.task_id.clone(),
                                },
                                TaskState::Error => api::ApiEvent::TaskError {
                                    task_id: event.task_id.clone(),
                                    error: event.error.clone().unwrap_or_default(),
                                },
                                _ => api::ApiEvent::TaskStateChanged {
                                    task_id: event.task_id.clone(),
                                    state: format!("{:?}", event.state).to_lowercase(),
                                },
                            };
                            ws.broadcast_api_event(api_event);

                            // 定期保存下载进度到 DB（每 10 秒，防止崩溃丢失进度）
                            if event.state == TaskState::Downloading && event.downloaded > 0 {
                                let now = std::time::Instant::now();
                                let should_save = last_db_save
                                    .get(&event.task_id)
                                    .map(|t| now.duration_since(*t).as_secs() >= 10)
                                    .unwrap_or(true);
                                if should_save {
                                    let db = db_ref.lock().await;
                                    if let Err(e) = db.update_task_progress_full(
                                        &event.task_id,
                                        event.downloaded,
                                        event.total_size,
                                    ) {
                                        tracing::error!("定期保存进度失败: {}", e);
                                    }
                                    drop(db);
                                    last_db_save.insert(event.task_id.clone(), now);
                                }
                            }

                            // 完成或出错时持久化到 DB
                            if event.state == TaskState::Done || event.state == TaskState::Error {
                                let db = db_ref.lock().await;
                                if event.state == TaskState::Done {
                                    if let Err(e) = db.complete_task(&event.task_id) {
                                        tracing::error!("完成任务记录失败: {} - {}", event.task_id, e);
                                    }
                                } else {
                                    if let Err(e) = db.set_task_error(
                                        &event.task_id,
                                        event.error.as_deref().unwrap_or("未知错误"),
                                    ) {
                                        tracing::error!("记录任务错误失败: {} - {}", event.task_id, e);
                                    }
                                }
                                if let Err(e) = db.update_task_progress_full(
                                    &event.task_id,
                                    event.downloaded,
                                    event.total_size,
                                ) {
                                    tracing::error!("更新任务进度失败: {} - {}", event.task_id, e);
                                }
                                drop(db);

                                // 直接清理已完成/出错任务的句柄（避免 cleanup_tx 竞态）
                                {
                                    let mut mgr = tm_ref.lock().await;
                                    mgr.process_cleanup(&[event.task_id.clone()]);
                                }

                                // 检查是否所有任务都已完成（用于触发下载后动作）
                                if event.state == TaskState::Done {
                                    let has_active = {
                                        let mgr = tm_ref.lock().await;
                                        mgr.has_active_tasks()
                                    };
                                    if !has_active {
                                        let cfg = config_ref.lock().await;
                                        let action = cfg.advanced.post_download_action.clone();
                                        let cmd = cfg.advanced.post_download_command.clone();
                                        drop(cfg);
                                        execute_post_action(&action, &cmd);
                                    }
                                }
                            }
                        }
                    });
                }

                // DB 持久化通道（定期保存进度）
                if let Some(mut db_persist_rx) = manager.take_db_persist_receiver() {
                    let db_ref = state_ref.db.clone();
                    tauri::async_runtime::spawn(async move {
                        while let Some(event) = db_persist_rx.recv().await {
                            let db = db_ref.lock().await;
                            if let Err(e) = db.update_task_progress_full(
                                &event.task_id,
                                event.downloaded,
                                event.total_size,
                            ) {
                                tracing::error!("定期保存进度失败: {} - {}", event.task_id, e);
                            }
                        }
                    });
                }

                // 任务完成清理通道（自动移除已完成/出错任务的句柄，防止内存泄漏）
                if let Some(mut cleanup_rx) = manager.take_cleanup_receiver() {
                    let tm_ref = state_ref.task_manager.clone();
                    tauri::async_runtime::spawn(async move {
                        while let Some(task_id) = cleanup_rx.recv().await {
                            let mut mgr = tm_ref.lock().await;
                            mgr.process_cleanup(&[task_id]);
                        }
                    });
                }
            }

            // 启动时自动恢复未完成的任务
            let db_for_spawn = state_ref.db.clone();
            let tm_for_spawn = state_ref.task_manager.clone();
            tauri::async_runtime::spawn(async move {
                // 等待一小段时间确保前端准备好
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // 先从 DB 获取未完成任务（不持有 task_manager 锁）
                let unfinished = {
                    let db = db_for_spawn.lock().await;
                    match db.get_unfinished_tasks() {
                        Ok(tasks) => tasks,
                        Err(e) => {
                            warn!("获取未完成任务失败: {}", e);
                            return;
                        }
                    }
                };

                let mut resumed_count = 0;
                let mut resumed_ids = Vec::new();

                // 通过 manager 恢复任务（不持有 db 锁）
                {
                    let mut manager = tm_for_spawn.lock().await;
                    let db = db_for_spawn.lock().await;
                    for task in unfinished {
                        if task.state == TaskState::Downloading || task.state == TaskState::Queued {
                            // 从数据库读取持久化的下载参数（代理/认证/限速等）
                            let params = db.get_task_params(&task.id)
                                .ok()
                                .flatten()
                                .unwrap_or_else(|| storage::db::TaskParams {
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
                                    resumed_count += 1;
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
                    let db = db_for_spawn.lock().await;
                    for id in &resumed_ids {
                        if let Err(e) = db.update_task_state(id, &TaskState::Downloading) {
                            tracing::warn!("启动恢复：更新任务状态失败: {} - {}", id, e);
                        }
                    }
                    info!("启动时恢复了 {} 个未完成任务", resumed_count);
                }
            });

            // 调度自动执行循环：每 30 秒检查一次
            {
                let schedule_mgr = state_ref.schedule_manager.clone();
                let task_mgr = state_ref.task_manager.clone();
                tauri::async_runtime::spawn(async move {
                    loop {
                        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                        // 检查并执行待执行的调度规则
                        let pending = {
                            let mut mgr = schedule_mgr.lock().await;
                            mgr.get_pending_rules()
                        };
                        for rule in pending {
                            match rule.rule_type {
                                crate::schedule::ScheduleRuleType::StartTask => {
                                    if let Some(ref task_id) = rule.task_id {
                                        let mut mgr = task_mgr.lock().await;
                                        if let Err(e) = mgr.resume_task(task_id).await {
                                            warn!("调度规则 {} 恢复任务失败: {}", rule.name, e);
                                        } else {
                                            info!("调度规则 {} 已恢复任务 {}", rule.name, task_id);
                                        }
                                    } else {
                                        // 全局恢复所有暂停任务
                                        let mut mgr = task_mgr.lock().await;
                                        if let Err(e) = mgr.resume_all_paused().await {
                                            warn!("调度规则 {} 恢复所有任务失败: {}", rule.name, e);
                                        } else {
                                            info!("调度规则 {} 已恢复所有暂停任务", rule.name);
                                        }
                                    }
                                }
                                crate::schedule::ScheduleRuleType::PauseTask => {
                                    if let Some(ref task_id) = rule.task_id {
                                        let mut mgr = task_mgr.lock().await;
                                        if let Err(e) = mgr.pause_task(task_id).await {
                                            warn!("调度规则 {} 暂停任务失败: {}", rule.name, e);
                                        } else {
                                            info!("调度规则 {} 已暂停任务 {}", rule.name, task_id);
                                        }
                                    } else {
                                        let mut mgr = task_mgr.lock().await;
                                        mgr.pause_all().await;
                                        info!("调度规则 {} 已暂停所有任务", rule.name);
                                    }
                                }
                                crate::schedule::ScheduleRuleType::BandwidthPlan => {
                                    let mut mgr = task_mgr.lock().await;
                                    if let Some(speed) = rule.params.download_speed {
                                        mgr.set_global_download_speed_limit(speed);
                                        info!("调度规则 {} 设置全局下载限速: {} bytes/s", rule.name, speed);
                                    }
                                    if let Some(speed) = rule.params.upload_speed {
                                        mgr.set_global_upload_speed_limit(speed);
                                        info!("调度规则 {} 设置全局上传限速: {} bytes/s", rule.name, speed);
                                    }
                                }
                                _ => {}
                            }
                        }

                        // 检查带宽计划并应用限速
                        let bandwidth_limit = {
                            let mgr = schedule_mgr.lock().await;
                            mgr.get_current_bandwidth_limit()
                        };
                        if let Some((dl_speed, ul_speed)) = bandwidth_limit {
                            let mut mgr = task_mgr.lock().await;
                            mgr.set_global_download_speed_limit(dl_speed);
                            mgr.set_global_upload_speed_limit(ul_speed);
                        }
                    }
                });
            }

            // KAD 定期维护：每 5 分钟清理过期节点、刷新路由表
            {
                let tm = state_ref.task_manager.clone();
                tauri::async_runtime::spawn(async move {
                    // 等待 KAD 引导完成
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    loop {
                        {
                            let mut mgr = tm.lock().await;
                            mgr.maintain_kad().await;
                        }
                        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
                    }
                });
            }

            info!("应用初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 任务管理命令
            commands::task::add_task,
            commands::task::pause_task,
            commands::task::resume_task,
            commands::task::remove_task,
            commands::task::get_task,
            commands::task::get_all_tasks,
            commands::task::set_priority,
            commands::task::open_file_location,
            commands::task::open_file,
            commands::task::set_task_speed_limit,
            commands::task::get_torrent_files,
            commands::task::add_bt_task_with_files,
            commands::task::batch_add_tasks,
            commands::task::add_mirror_url,
            commands::task::remove_mirror_url,
            commands::task::get_mirror_urls,
            commands::task::get_download_history,
            commands::task::clear_download_history,
            commands::task::pause_all_tasks,
            commands::task::resume_all_tasks,
            commands::task::remove_completed_tasks,
            commands::task::resume_unfinished_tasks,
            commands::task::get_task_peers,
            commands::task::get_task_trackers,
            commands::task::get_task_logs,
            // 设置命令
            commands::settings::get_settings,
            commands::settings::update_settings,
            // 系统命令
            commands::system::get_app_info,
            commands::system::get_bt_status,
            commands::system::get_api_status,
            commands::system::get_kad_status,
            // RSS 订阅命令
            commands::rss::add_rss_feed,
            commands::rss::remove_rss_feed,
            commands::rss::get_rss_feeds,
            commands::rss::import_opml,
            commands::rss::export_opml,
            commands::rss::set_rss_feed_enabled,
            commands::rss::update_rss_feed,
            // 自动解压命令
            commands::archive::get_archive_config,
            commands::archive::update_archive_config,
            commands::archive::extract_archive,
            // 插件管理命令
            commands::plugin::list_plugins,
            commands::plugin::install_plugin,
            commands::plugin::uninstall_plugin,
            commands::plugin::enable_plugin,
            commands::plugin::disable_plugin,
            // 调度管理命令
            commands::schedule::add_schedule_rule,
            commands::schedule::remove_schedule_rule,
            commands::schedule::update_schedule_rule,
            commands::schedule::set_schedule_rule_enabled,
            commands::schedule::get_schedule_rules,
            commands::schedule::get_schedule_rule,
            commands::schedule::add_bandwidth_schedule,
            commands::schedule::remove_bandwidth_schedule,
            commands::schedule::update_bandwidth_schedule,
            commands::schedule::get_bandwidth_schedules,
            commands::schedule::get_current_bandwidth_limit,
        ])
        .run(tauri::generate_context!())
        .expect("启动应用失败");
}

/// 初始化日志系统
fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();
}
