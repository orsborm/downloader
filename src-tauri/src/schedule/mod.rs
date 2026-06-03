// 任务调度模块
// 支持定时开始/暂停任务，带宽计划，cron 表达式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// 调度规则类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScheduleRuleType {
    /// 定时开始任务
    StartTask,
    /// 定时暂停任务
    PauseTask,
    /// 带宽计划（按时段调整速度限制）
    BandwidthPlan,
    /// 做种计划
    SeedPlan,
}

/// 调度规则
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleRule {
    /// 规则 ID
    pub id: String,
    /// 规则名称
    pub name: String,
    /// 规则类型
    pub rule_type: ScheduleRuleType,
    /// cron 表达式（秒 分 时 日 月 周）
    pub cron_expression: String,
    /// 关联的任务 ID（None 表示全局规则）
    pub task_id: Option<String>,
    /// 规则参数
    pub params: ScheduleParams,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: String,
    /// 上次执行时间
    pub last_executed_at: Option<String>,
}

/// 调度规则参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleParams {
    /// 下载速度限制（bytes/sec，0=不限速）
    pub download_speed: Option<u64>,
    /// 上传速度限制（bytes/sec，0=不限速）
    pub upload_speed: Option<u64>,
    /// 执行的命令/脚本
    pub command: Option<String>,
    /// 自定义参数
    pub custom: HashMap<String, String>,
}

impl Default for ScheduleParams {
    fn default() -> Self {
        Self {
            download_speed: None,
            upload_speed: None,
            command: None,
            custom: HashMap::new(),
        }
    }
}

/// 带宽计划时段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BandwidthSchedule {
    /// 开始时间（HH:MM 格式）
    pub start_time: String,
    /// 结束时间（HH:MM 格式）
    pub end_time: String,
    /// 下载速度限制（bytes/sec，0=不限速）
    pub download_speed: u64,
    /// 上传速度限制（bytes/sec，0=不限速）
    pub upload_speed: u64,
    /// 星期几有效（1-7，None 表示每天）
    pub weekdays: Option<Vec<u8>>,
}

/// 调度管理器
pub struct ScheduleManager {
    /// 调度规则列表
    rules: Vec<ScheduleRule>,
    /// 带宽计划列表
    bandwidth_schedules: Vec<BandwidthSchedule>,
}

impl ScheduleManager {
    /// 创建新的调度管理器
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            bandwidth_schedules: Vec::new(),
        }
    }

    /// 添加调度规则
    pub fn add_rule(&mut self, rule: ScheduleRule) -> Result<(), String> {
        // 验证 cron 表达式
        if !is_valid_cron(&rule.cron_expression) {
            return Err(format!("无效的 cron 表达式: {}", rule.cron_expression));
        }

        // 检查 ID 是否重复
        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(format!("规则 ID 已存在: {}", rule.id));
        }

        info!("添加调度规则: {} ({})", rule.name, rule.id);
        self.rules.push(rule);
        Ok(())
    }

    /// 删除调度规则
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<(), String> {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);

        if self.rules.len() == initial_len {
            return Err(format!("规则不存在: {}", rule_id));
        }

        info!("删除调度规则: {}", rule_id);
        Ok(())
    }

    /// 更新调度规则
    pub fn update_rule(&mut self, rule: ScheduleRule) -> Result<(), String> {
        // 验证 cron 表达式
        if !is_valid_cron(&rule.cron_expression) {
            return Err(format!("无效的 cron 表达式: {}", rule.cron_expression));
        }

        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule.clone();
            info!("更新调度规则: {} ({})", rule.name, rule.id);
            Ok(())
        } else {
            Err(format!("规则不存在: {}", rule.id))
        }
    }

    /// 启用/禁用规则
    pub fn set_rule_enabled(&mut self, rule_id: &str, enabled: bool) -> Result<(), String> {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = enabled;
            info!("{}调度规则: {}", if enabled { "启用" } else { "禁用" }, rule_id);
            Ok(())
        } else {
            Err(format!("规则不存在: {}", rule_id))
        }
    }

    /// 获取所有规则
    pub fn get_rules(&self) -> &[ScheduleRule] {
        &self.rules
    }

    /// 获取指定规则
    pub fn get_rule(&self, rule_id: &str) -> Option<&ScheduleRule> {
        self.rules.iter().find(|r| r.id == rule_id)
    }

    /// 添加带宽计划
    pub fn add_bandwidth_schedule(&mut self, schedule: BandwidthSchedule) -> Result<(), String> {
        // 验证时间格式
        if !is_valid_time(&schedule.start_time) || !is_valid_time(&schedule.end_time) {
            return Err("无效的时间格式，应为 HH:MM".to_string());
        }

        // 验证星期几
        if let Some(ref weekdays) = schedule.weekdays {
            for &day in weekdays {
                if day < 1 || day > 7 {
                    return Err(format!("无效的星期几: {}，应为 1-7", day));
                }
            }
        }

        info!("添加带宽计划: {} - {}", schedule.start_time, schedule.end_time);
        self.bandwidth_schedules.push(schedule);
        Ok(())
    }

    /// 删除带宽计划
    pub fn remove_bandwidth_schedule(&mut self, index: usize) -> Result<(), String> {
        if index >= self.bandwidth_schedules.len() {
            return Err("带宽计划索引越界".to_string());
        }

        self.bandwidth_schedules.remove(index);
        info!("删除带宽计划: 索引 {}", index);
        Ok(())
    }

    /// 获取所有带宽计划
    pub fn get_bandwidth_schedules(&self) -> &[BandwidthSchedule] {
        &self.bandwidth_schedules
    }

    /// 更新带宽计划
    pub fn update_bandwidth_schedule(&mut self, index: usize, schedule: BandwidthSchedule) -> Result<(), String> {
        if index >= self.bandwidth_schedules.len() {
            return Err("带宽计划索引越界".to_string());
        }

        // 验证时间格式
        if !is_valid_time(&schedule.start_time) || !is_valid_time(&schedule.end_time) {
            return Err("无效的时间格式，应为 HH:MM".to_string());
        }

        self.bandwidth_schedules[index] = schedule;
        info!("更新带宽计划: 索引 {}", index);
        Ok(())
    }

    /// 检查当前时间是否在带宽计划时段内
    pub fn get_current_bandwidth_limit(&self) -> Option<(u64, u64)> {
        let now = chrono::Local::now();
        let current_time = now.format("%H:%M").to_string();
        let current_weekday = now.format("%u").to_string().parse::<u8>().unwrap_or(1);

        for schedule in &self.bandwidth_schedules {
            // 检查星期几
            if let Some(ref weekdays) = schedule.weekdays {
                if !weekdays.contains(&current_weekday) {
                    continue;
                }
            }

            // 检查时间范围
            if is_time_in_range(&current_time, &schedule.start_time, &schedule.end_time) {
                return Some((schedule.download_speed, schedule.upload_speed));
            }
        }

        None
    }

    /// 获取应该执行的规则（基于当前时间，防重复执行）
    pub fn get_pending_rules(&mut self) -> Vec<ScheduleRule> {
        let now = chrono::Local::now();
        let current_time = now.format("%H:%M").to_string();
        let current_minute = now.format("%Y-%m-%d %H:%M").to_string();
        let current_weekday = now.format("%u").to_string().parse::<u8>().unwrap_or(1);

        let mut pending = Vec::new();

        for rule in &mut self.rules {
            if !rule.enabled {
                continue;
            }

            // 防重复：如果本分钟内已执行过，跳过
            if let Some(ref last) = rule.last_executed_at {
                if last.starts_with(&current_minute) {
                    continue;
                }
            }

            // 简化的 cron 匹配
            if should_execute_rule(&rule.cron_expression, &current_time, current_weekday) {
                rule.last_executed_at = Some(now.format("%Y-%m-%d %H:%M:%S").to_string());
                pending.push(rule.clone());
            }
        }

        pending
    }
}

/// 验证 cron 表达式格式（简化版）
fn is_valid_cron(expression: &str) -> bool {
    // 简化验证：检查格式为 "分 时 日 月 周"
    let parts: Vec<&str> = expression.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }

    // 验证每个部分
    for (i, part) in parts.iter().enumerate() {
        if !is_valid_cron_part(part, i) {
            return false;
        }
    }

    true
}

/// 验证 cron 表达式的单个部分
fn is_valid_cron_part(part: &str, index: usize) -> bool {
    if part == "*" {
        return true;
    }

    // 处理逗号分隔的多个值
    for sub_part in part.split(',') {
        if !is_valid_cron_sub_part(sub_part, index) {
            return false;
        }
    }

    true
}

/// 验证 cron 表达式的子部分
fn is_valid_cron_sub_part(part: &str, index: usize) -> bool {
    if part == "*" {
        return true;
    }

    // 处理范围 (如 1-5)
    if let Some((start, end)) = part.split_once('-') {
        return is_valid_cron_value(start, index) && is_valid_cron_value(end, index);
    }

    // 处理步长 (如 */5)
    if let Some((base, step)) = part.split_once('/') {
        if base != "*" {
            return false;
        }
        return step.parse::<u32>().is_ok();
    }

    // 处理单个值
    is_valid_cron_value(part, index)
}

/// 验证 cron 表达式的值
fn is_valid_cron_value(value: &str, index: usize) -> bool {
    let num = match value.parse::<u32>() {
        Ok(n) => n,
        Err(_) => return false,
    };

    match index {
        0 => num < 60,  // 分钟
        1 => num < 24,  // 小时
        2 => num >= 1 && num <= 31,  // 日
        3 => num >= 1 && num <= 12,  // 月
        4 => num >= 1 && num <= 7,   // 周
        _ => false,
    }
}

/// 验证时间格式 (HH:MM)
fn is_valid_time(time: &str) -> bool {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    let hours = parts[0].parse::<u32>();
    let minutes = parts[1].parse::<u32>();

    match (hours, minutes) {
        (Ok(h), Ok(m)) => h < 24 && m < 60,
        _ => false,
    }
}

/// 检查时间是否在范围内
fn is_time_in_range(current: &str, start: &str, end: &str) -> bool {
    let current_minutes = time_to_minutes(current);
    let start_minutes = time_to_minutes(start);
    let end_minutes = time_to_minutes(end);

    if start_minutes <= end_minutes {
        // 同一天内
        current_minutes >= start_minutes && current_minutes <= end_minutes
    } else {
        // 跨天
        current_minutes >= start_minutes || current_minutes <= end_minutes
    }
}

/// 将时间转换为分钟数
fn time_to_minutes(time: &str) -> u32 {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return 0;
    }

    let hours = parts[0].parse::<u32>().unwrap_or(0);
    let minutes = parts[1].parse::<u32>().unwrap_or(0);

    hours * 60 + minutes
}

/// 检查是否应该执行规则（简化版 cron 匹配）
fn should_execute_rule(cron: &str, current_time: &str, current_weekday: u8) -> bool {
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() != 5 {
        return false;
    }

    let current_parts: Vec<u32> = current_time
        .split(':')
        .filter_map(|s| s.parse().ok())
        .collect();

    if current_parts.len() != 2 {
        return false;
    }

    // current_time 格式为 "HH:MM"，split(':') 得到 [HH, MM]
    let current_hour = current_parts[0];
    let current_minute = current_parts[1];

    // 检查分钟
    if !matches_cron_part(parts[0], current_minute) {
        return false;
    }

    // 检查小时
    if !matches_cron_part(parts[1], current_hour) {
        return false;
    }

    // 检查日（简化：不检查具体日期）
    if parts[2] != "*" {
        return false;
    }

    // 检查月（简化：不检查具体月份）
    if parts[3] != "*" {
        return false;
    }

    // 检查星期
    if parts[4] != "*" {
        if !matches_cron_part(parts[4], current_weekday as u32) {
            return false;
        }
    }

    true
}

/// 检查 cron 表达式部分是否匹配当前值
fn matches_cron_part(part: &str, current: u32) -> bool {
    if part == "*" {
        return true;
    }

    // 处理逗号分隔的多个值
    for sub_part in part.split(',') {
        if matches_cron_sub_part(sub_part, current) {
            return true;
        }
    }

    false
}

/// 检查 cron 表达式子部分是否匹配
fn matches_cron_sub_part(part: &str, current: u32) -> bool {
    if part == "*" {
        return true;
    }

    // 处理范围 (如 1-5)
    if let Some((start, end)) = part.split_once('-') {
        if let (Ok(s), Ok(e)) = (start.parse::<u32>(), end.parse::<u32>()) {
            return current >= s && current <= e;
        }
        return false;
    }

    // 处理步长 (如 */5)
    if let Some((base, step)) = part.split_once('/') {
        if base == "*" {
            if let Ok(step) = step.parse::<u32>() {
                return step > 0 && current % step == 0;
            }
        }
        return false;
    }

    // 处理单个值
    if let Ok(value) = part.parse::<u32>() {
        return current == value;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_cron_expressions() {
        assert!(is_valid_cron("* * * * *"));
        assert!(is_valid_cron("0 * * * *"));
        assert!(is_valid_cron("0 9 * * 1-5"));
        assert!(is_valid_cron("*/5 * * * *"));
        assert!(is_valid_cron("0 0 1 * *"));
        assert!(is_valid_cron("30 8 * * 1,3,5"));
    }

    #[test]
    fn test_invalid_cron_expressions() {
        assert!(!is_valid_cron(""));
        assert!(!is_valid_cron("* * *"));
        assert!(!is_valid_cron("60 * * * *"));
        assert!(!is_valid_cron("* 24 * * *"));
        assert!(!is_valid_cron("* * 0 * *"));
        assert!(!is_valid_cron("* * 32 * *"));
        assert!(!is_valid_cron("* * * 0 *"));
        assert!(!is_valid_cron("* * * 13 *"));
        assert!(!is_valid_cron("* * * * 0"));
        assert!(!is_valid_cron("* * * * 8"));
    }

    #[test]
    fn test_valid_time_format() {
        assert!(is_valid_time("00:00"));
        assert!(is_valid_time("12:30"));
        assert!(is_valid_time("23:59"));
    }

    #[test]
    fn test_invalid_time_format() {
        assert!(!is_valid_time(""));
        assert!(!is_valid_time("12"));
        assert!(!is_valid_time("24:00"));
        assert!(!is_valid_time("12:60"));
        assert!(!is_valid_time("ab:cd"));
    }

    #[test]
    fn test_time_in_range_same_day() {
        assert!(is_time_in_range("12:00", "09:00", "18:00"));
        assert!(is_time_in_range("09:00", "09:00", "18:00"));
        assert!(is_time_in_range("18:00", "09:00", "18:00"));
        assert!(!is_time_in_range("08:00", "09:00", "18:00"));
        assert!(!is_time_in_range("19:00", "09:00", "18:00"));
    }

    #[test]
    fn test_time_in_range_cross_day() {
        assert!(is_time_in_range("23:00", "22:00", "06:00"));
        assert!(is_time_in_range("03:00", "22:00", "06:00"));
        assert!(!is_time_in_range("12:00", "22:00", "06:00"));
    }

    #[test]
    fn test_cron_part_matching() {
        assert!(matches_cron_part("*", 0));
        assert!(matches_cron_part("*", 59));
        assert!(matches_cron_part("0", 0));
        assert!(!matches_cron_part("0", 1));
        assert!(matches_cron_part("1-5", 3));
        assert!(!matches_cron_part("1-5", 6));
        assert!(matches_cron_part("*/5", 0));
        assert!(matches_cron_part("*/5", 5));
        assert!(matches_cron_part("*/5", 10));
        assert!(!matches_cron_part("*/5", 7));
        assert!(matches_cron_part("1,3,5", 3));
        assert!(!matches_cron_part("1,3,5", 2));
    }

    #[test]
    fn test_schedule_manager_add_rule() {
        let mut manager = ScheduleManager::new();

        let rule = ScheduleRule {
            id: "test-1".to_string(),
            name: "测试规则".to_string(),
            rule_type: ScheduleRuleType::StartTask,
            cron_expression: "0 9 * * *".to_string(),
            task_id: Some("task-1".to_string()),
            params: ScheduleParams::default(),
            enabled: true,
            created_at: "2026-01-01 00:00:00".to_string(),
            last_executed_at: None,
        };

        assert!(manager.add_rule(rule).is_ok());
        assert_eq!(manager.get_rules().len(), 1);
    }

    #[test]
    fn test_schedule_manager_duplicate_id() {
        let mut manager = ScheduleManager::new();

        let rule1 = ScheduleRule {
            id: "test-1".to_string(),
            name: "规则1".to_string(),
            rule_type: ScheduleRuleType::StartTask,
            cron_expression: "0 9 * * *".to_string(),
            task_id: None,
            params: ScheduleParams::default(),
            enabled: true,
            created_at: "2026-01-01 00:00:00".to_string(),
            last_executed_at: None,
        };

        let rule2 = ScheduleRule {
            id: "test-1".to_string(),
            name: "规则2".to_string(),
            rule_type: ScheduleRuleType::PauseTask,
            cron_expression: "0 18 * * *".to_string(),
            task_id: None,
            params: ScheduleParams::default(),
            enabled: true,
            created_at: "2026-01-01 00:00:00".to_string(),
            last_executed_at: None,
        };

        assert!(manager.add_rule(rule1).is_ok());
        assert!(manager.add_rule(rule2).is_err());
    }

    #[test]
    fn test_schedule_manager_remove_rule() {
        let mut manager = ScheduleManager::new();

        let rule = ScheduleRule {
            id: "test-1".to_string(),
            name: "测试规则".to_string(),
            rule_type: ScheduleRuleType::StartTask,
            cron_expression: "0 9 * * *".to_string(),
            task_id: None,
            params: ScheduleParams::default(),
            enabled: true,
            created_at: "2026-01-01 00:00:00".to_string(),
            last_executed_at: None,
        };

        manager.add_rule(rule).unwrap();
        assert_eq!(manager.get_rules().len(), 1);

        assert!(manager.remove_rule("test-1").is_ok());
        assert_eq!(manager.get_rules().len(), 0);

        assert!(manager.remove_rule("nonexistent").is_err());
    }

    #[test]
    fn test_bandwidth_schedule() {
        let mut manager = ScheduleManager::new();

        let schedule = BandwidthSchedule {
            start_time: "22:00".to_string(),
            end_time: "06:00".to_string(),
            download_speed: 1024 * 1024,  // 1 MB/s
            upload_speed: 512 * 1024,     // 512 KB/s
            weekdays: None,
        };

        assert!(manager.add_bandwidth_schedule(schedule).is_ok());
        assert_eq!(manager.get_bandwidth_schedules().len(), 1);
    }

    #[test]
    fn test_invalid_bandwidth_schedule() {
        let mut manager = ScheduleManager::new();

        let schedule = BandwidthSchedule {
            start_time: "25:00".to_string(),
            end_time: "06:00".to_string(),
            download_speed: 1024 * 1024,
            upload_speed: 512 * 1024,
            weekdays: None,
        };

        assert!(manager.add_bandwidth_schedule(schedule).is_err());
    }
}
