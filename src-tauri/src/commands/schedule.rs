// 调度管理 IPC 命令
// 每次变更后自动持久化到数据库

use crate::schedule::{BandwidthSchedule, ScheduleRule};
use crate::AppState;
use tauri::State;
use tracing::{info, warn};

/// 辅助函数：将调度规则和带宽计划持久化到数据库
async fn persist_schedule(state: &State<'_, AppState>) {
    let manager = state.schedule_manager.lock().await;
    let db = state.db.lock().await;
    if let Err(e) = db.save_schedule_rules(manager.get_rules()) {
        warn!("持久化调度规则失败: {}", e);
    }
    if let Err(e) = db.save_bandwidth_schedules(manager.get_bandwidth_schedules()) {
        warn!("持久化带宽计划失败: {}", e);
    }
}

/// 添加调度规则
#[tauri::command]
pub async fn add_schedule_rule(
    state: State<'_, AppState>,
    rule: ScheduleRule,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.add_rule(rule)?;
    drop(manager);
    info!("调度规则已添加");
    persist_schedule(&state).await;
    Ok(())
}

/// 删除调度规则
#[tauri::command]
pub async fn remove_schedule_rule(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.remove_rule(&rule_id)?;
    drop(manager);
    info!("调度规则已删除: {}", rule_id);
    persist_schedule(&state).await;
    Ok(())
}

/// 更新调度规则
#[tauri::command]
pub async fn update_schedule_rule(
    state: State<'_, AppState>,
    rule: ScheduleRule,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.update_rule(rule)?;
    drop(manager);
    info!("调度规则已更新");
    persist_schedule(&state).await;
    Ok(())
}

/// 启用/禁用调度规则
#[tauri::command]
pub async fn set_schedule_rule_enabled(
    state: State<'_, AppState>,
    rule_id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.set_rule_enabled(&rule_id, enabled)?;
    drop(manager);
    info!("调度规则 {} 已{}", rule_id, if enabled { "启用" } else { "禁用" });
    persist_schedule(&state).await;
    Ok(())
}

/// 获取所有调度规则
#[tauri::command]
pub async fn get_schedule_rules(
    state: State<'_, AppState>,
) -> Result<Vec<ScheduleRule>, String> {
    let manager = state.schedule_manager.lock().await;
    Ok(manager.get_rules().to_vec())
}

/// 获取指定调度规则
#[tauri::command]
pub async fn get_schedule_rule(
    state: State<'_, AppState>,
    rule_id: String,
) -> Result<Option<ScheduleRule>, String> {
    let manager = state.schedule_manager.lock().await;
    Ok(manager.get_rule(&rule_id).cloned())
}

/// 添加带宽计划
#[tauri::command]
pub async fn add_bandwidth_schedule(
    state: State<'_, AppState>,
    schedule: BandwidthSchedule,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.add_bandwidth_schedule(schedule)?;
    drop(manager);
    info!("带宽计划已添加");
    persist_schedule(&state).await;
    Ok(())
}

/// 删除带宽计划
#[tauri::command]
pub async fn remove_bandwidth_schedule(
    state: State<'_, AppState>,
    index: usize,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.remove_bandwidth_schedule(index)?;
    drop(manager);
    info!("带宽计划已删除: 索引 {}", index);
    persist_schedule(&state).await;
    Ok(())
}

/// 更新带宽计划
#[tauri::command]
pub async fn update_bandwidth_schedule(
    state: State<'_, AppState>,
    index: usize,
    schedule: BandwidthSchedule,
) -> Result<(), String> {
    let mut manager = state.schedule_manager.lock().await;
    manager.update_bandwidth_schedule(index, schedule)?;
    drop(manager);
    info!("带宽计划已更新: 索引 {}", index);
    persist_schedule(&state).await;
    Ok(())
}

/// 获取所有带宽计划
#[tauri::command]
pub async fn get_bandwidth_schedules(
    state: State<'_, AppState>,
) -> Result<Vec<BandwidthSchedule>, String> {
    let manager = state.schedule_manager.lock().await;
    Ok(manager.get_bandwidth_schedules().to_vec())
}

/// 获取当前带宽限制
#[tauri::command]
pub async fn get_current_bandwidth_limit(
    state: State<'_, AppState>,
) -> Result<Option<(u64, u64)>, String> {
    let manager = state.schedule_manager.lock().await;
    Ok(manager.get_current_bandwidth_limit())
}
