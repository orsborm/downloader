// Tauri IPC 调用封装
// 统一管理前端与 Rust 后端的通信接口

import { invoke } from "@tauri-apps/api/core";
import type { TaskParams, TaskStatus, AppConfig, DownloadHistory } from "./types";

/** 添加下载任务 */
export async function addTask(params: TaskParams): Promise<string> {
  return invoke<string>("add_task", { params });
}

/** 暂停任务 */
export async function pauseTask(id: string): Promise<void> {
  return invoke("pause_task", { id });
}

/** 恢复任务 */
export async function resumeTask(id: string): Promise<void> {
  return invoke("resume_task", { id });
}

/** 删除任务 */
export async function removeTask(
  id: string,
  deleteFiles: boolean = false
): Promise<void> {
  return invoke("remove_task", { id, deleteFiles });
}

/** 获取单个任务状态 */
export async function getTask(id: string): Promise<TaskStatus | null> {
  return invoke<TaskStatus | null>("get_task", { id });
}

/** 获取所有任务 */
export async function getAllTasks(): Promise<TaskStatus[]> {
  return invoke<TaskStatus[]>("get_all_tasks");
}

/** 设置任务优先级 */
export async function setTaskPriority(
  id: string,
  priority: number
): Promise<void> {
  return invoke("set_priority", { id, priority });
}

/** 获取应用配置 */
export async function getSettings(): Promise<AppConfig> {
  return invoke<AppConfig>("get_settings");
}

/** 更新应用配置 */
export async function updateSettings(config: AppConfig): Promise<void> {
  return invoke("update_settings", { newConfig: config });
}

/** 获取应用信息 */
export async function getAppInfo(): Promise<{
  version: string;
  platform: string;
  arch: string;
}> {
  return invoke("get_app_info");
}

/** 打开文件所在目录 */
export async function openFileLocation(id: string): Promise<void> {
  return invoke("open_file_location", { id });
}

/** 打开文件（使用系统默认程序） */
export async function openFile(id: string): Promise<void> {
  return invoke("open_file", { id });
}

/** 设置单任务速度限制 */
export async function setTaskSpeedLimit(
  id: string,
  downloadLimit: number,
  uploadLimit: number
): Promise<void> {
  return invoke("set_task_speed_limit", { id, downloadLimit, uploadLimit });
}

/** 种子文件信息 */
export interface TorrentFileInfo {
  index: number;
  name: string;
  size: number;
  selected: boolean;
}

/** 种子文件列表响应 */
export interface TorrentFileListResponse {
  name: string;
  totalSize: number;
  files: TorrentFileInfo[];
}

/** 获取种子文件列表（不启动下载） */
export async function getTorrentFiles(source: string): Promise<TorrentFileListResponse> {
  return invoke("get_torrent_files", { source });
}

/** 添加 BT/Magnet 任务（支持文件选择） */
export async function addBtTaskWithFiles(
  params: TaskParams,
  onlyFiles?: number[],
  torrentFiles?: TorrentFileInfo[]
): Promise<string> {
  return invoke("add_bt_task_with_files", { params, onlyFiles, torrentFiles });
}

/** 批量添加任务（支持多链接） */
export async function batchAddTasks(
  urls: string[],
  savePath?: string,
  startImmediately: boolean = true
): Promise<string[]> {
  return invoke("batch_add_tasks", { urls, savePath, startImmediately });
}

/** 添加镜像URL到现有任务 */
export async function addMirrorUrl(
  taskId: string,
  mirrorUrl: string
): Promise<void> {
  return invoke("add_mirror_url", { taskId, mirrorUrl });
}

/** 移除任务的镜像URL */
export async function removeMirrorUrl(
  taskId: string,
  mirrorUrl: string
): Promise<void> {
  return invoke("remove_mirror_url", { taskId, mirrorUrl });
}

/** 获取任务的镜像URL列表 */
export async function getMirrorUrls(taskId: string): Promise<string[]> {
  return invoke<string[]>("get_mirror_urls", { taskId });
}

/** 获取下载历史 */
export async function getDownloadHistory(
  limit?: number
): Promise<DownloadHistory[]> {
  return invoke("get_download_history", { limit });
}

/** 清空下载历史 */
export async function clearDownloadHistory(): Promise<number> {
  return invoke<number>("clear_download_history");
}

/** 暂停所有任务 */
export async function pauseAllTasks(): Promise<string[]> {
  return invoke("pause_all_tasks");
}

/** 恢复所有暂停的任务 */
export async function resumeAllTasks(): Promise<string[]> {
  return invoke("resume_all_tasks");
}

/** 删除所有已完成/出错的任务 */
export async function removeCompletedTasks(): Promise<string[]> {
  return invoke("remove_completed_tasks");
}

/** 获取 BT 引擎状态（DHT 节点数等） */
export async function getBtStatus(): Promise<{
  dhtNodes: number;
  dhtConnected: boolean;
  uploadSpeed: number;
  downloadSpeed: number;
}> {
  return invoke("get_bt_status");
}

// ---- RSS 订阅管理 ----

/** RSS 订阅信息 */
export interface RssFeedInfo {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  interval: number;
  lastUpdate: string | null;
}

/** 添加 RSS 订阅 */
export async function addRssFeed(
  name: string,
  url: string,
  interval: number
): Promise<string> {
  return invoke<string>("add_rss_feed", { name, url, interval });
}

/** 删除 RSS 订阅 */
export async function removeRssFeed(id: string): Promise<void> {
  return invoke("remove_rss_feed", { id });
}

/** 获取所有 RSS 订阅 */
export async function getRssFeeds(): Promise<RssFeedInfo[]> {
  return invoke<RssFeedInfo[]>("get_rss_feeds");
}

/** 导入 OPML */
export async function importOpml(content: string): Promise<number> {
  return invoke<number>("import_opml", { content });
}

/** 导出 OPML */
export async function exportOpml(): Promise<string> {
  return invoke<string>("export_opml");
}

// ---- 自动解压管理 ----

/** 解压密码信息 */
export interface PasswordInfo {
  id: string;
  name: string;
  password: string;
  pattern: string;
  enabled: boolean;
}

/** 解压配置 */
export interface ArchiveConfigInfo {
  autoExtract: boolean;
  deleteAfterExtract: boolean;
  extractDir: string;
  passwords: PasswordInfo[];
}

/** 获取解压配置 */
export async function getArchiveConfig(): Promise<ArchiveConfigInfo> {
  return invoke<ArchiveConfigInfo>("get_archive_config");
}

/** 更新解压配置 */
export async function updateArchiveConfig(
  config: ArchiveConfigInfo
): Promise<void> {
  return invoke("update_archive_config", { config });
}

// ---- 插件管理 ----

/** 插件信息 */
export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  pluginType: string;
  state: string;
}

/** 安装插件 */
export async function installPlugin(wasmPath: string): Promise<string> {
  return invoke<string>("install_plugin", { wasmPath });
}

/** 卸载插件 */
export async function uninstallPlugin(id: string): Promise<void> {
  return invoke("uninstall_plugin", { id });
}

/** 启用插件 */
export async function enablePlugin(id: string): Promise<void> {
  return invoke("enable_plugin", { id });
}

/** 禁用插件 */
export async function disablePlugin(id: string): Promise<void> {
  return invoke("disable_plugin", { id });
}

// ---- API 状态 ----

/** API 服务状态 */
export interface ApiStatusInfo {
  enabled: boolean;
  host: string;
  port: number;
  endpoint: string;
  wsConnections: number;
}

/** 获取 JSON-RPC API 服务状态 */
export async function getApiStatus(): Promise<ApiStatusInfo> {
  return invoke<ApiStatusInfo>("get_api_status");
}

/** KAD（ed2k Kademlia DHT）状态 */
export interface KadStatusInfo {
  running: boolean;
  nodeCount: number;
  listenPort: number;
  bootstrapDone: boolean;
}

/** 获取 KAD 状态 */
export async function getKadStatus(): Promise<KadStatusInfo | null> {
  return invoke<KadStatusInfo | null>("get_kad_status");
}

// ---- 调度管理 ----

/** 调度规则类型 */
export type ScheduleRuleType = "StartTask" | "PauseTask" | "BandwidthPlan" | "SeedPlan";

/** 调度规则参数 */
export interface ScheduleParams {
  downloadSpeed?: number;
  uploadSpeed?: number;
  command?: string;
  custom?: Record<string, string>;
}

/** 调度规则 */
export interface ScheduleRule {
  id: string;
  name: string;
  ruleType: ScheduleRuleType;
  cronExpression: string;
  taskId?: string;
  params: ScheduleParams;
  enabled: boolean;
  createdAt: string;
  lastExecutedAt?: string;
}

/** 带宽计划 */
export interface BandwidthSchedule {
  startTime: string;
  endTime: string;
  downloadSpeed: number;
  uploadSpeed: number;
  weekdays?: number[];
}

/** 添加调度规则 */
export async function addScheduleRule(rule: ScheduleRule): Promise<void> {
  return invoke("add_schedule_rule", { rule });
}

/** 删除调度规则 */
export async function removeScheduleRule(ruleId: string): Promise<void> {
  return invoke("remove_schedule_rule", { ruleId });
}

/** 更新调度规则 */
export async function updateScheduleRule(rule: ScheduleRule): Promise<void> {
  return invoke("update_schedule_rule", { rule });
}

/** 启用/禁用调度规则 */
export async function setScheduleRuleEnabled(ruleId: string, enabled: boolean): Promise<void> {
  return invoke("set_schedule_rule_enabled", { ruleId, enabled });
}

/** 获取所有调度规则 */
export async function getScheduleRules(): Promise<ScheduleRule[]> {
  return invoke<ScheduleRule[]>("get_schedule_rules");
}

/** 获取指定调度规则 */
export async function getScheduleRule(ruleId: string): Promise<ScheduleRule | null> {
  return invoke<ScheduleRule | null>("get_schedule_rule", { ruleId });
}

/** 添加带宽计划 */
export async function addBandwidthSchedule(schedule: BandwidthSchedule): Promise<void> {
  return invoke("add_bandwidth_schedule", { schedule });
}

/** 删除带宽计划 */
export async function removeBandwidthSchedule(index: number): Promise<void> {
  return invoke("remove_bandwidth_schedule", { index });
}

/** 更新带宽计划 */
export async function updateBandwidthSchedule(index: number, schedule: BandwidthSchedule): Promise<void> {
  return invoke("update_bandwidth_schedule", { index, schedule });
}

/** 获取所有带宽计划 */
export async function getBandwidthSchedules(): Promise<BandwidthSchedule[]> {
  return invoke<BandwidthSchedule[]>("get_bandwidth_schedules");
}

/** 获取任务的 Peer 连接列表 */
export async function getTaskPeers(id: string): Promise<PeerInfo[]> {
  return invoke<PeerInfo[]>("get_task_peers", { id });
}

/** 获取任务的 Tracker 列表 */
export async function getTaskTrackers(id: string): Promise<TrackerInfo[]> {
  return invoke<TrackerInfo[]>("get_task_trackers", { id });
}

/** 获取任务日志 */
export async function getTaskLogs(id: string, limit?: number): Promise<LogEntry[]> {
  return invoke<LogEntry[]>("get_task_logs", { id, limit });
}

/** Peer 连接信息 */
export interface PeerInfo {
  id: string;
  ip: string;
  port: number;
  downloadSpeed: number;
  uploadSpeed: number;
  client: string;
  progress: number;
}

/** Tracker 信息 */
export interface TrackerInfo {
  url: string;
  status: string;
  peers: number;
  seeders: number;
  leechers: number;
}

/** 日志条目 */
export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

/** 刷新全局任务列表（从后端重新加载并更新 Zustand store） */
export async function refreshTasks(): Promise<void> {
  const { useTaskStore } = await import("../stores/taskStore");
  const tasks = await getAllTasks();
  useTaskStore.getState().setTasks(tasks);
}
