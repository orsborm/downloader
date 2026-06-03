// 格式化工具函数
// 统一处理速度、大小、时间等显示格式

import { useI18nStore } from "../hooks/useI18n";
import { zh } from "./locales/zh";
import { en } from "./locales/en";

/** 获取当前语言包 */
function getLocale() {
  const lang = useI18nStore.getState().language;
  return lang === "en" ? en : zh;
}

/** 格式化文件大小（自动选择 B/KB/MB/GB/TB） */
export function formatSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B";

  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  const TB = GB * 1024;

  if (bytes >= TB) return `${(bytes / TB).toFixed(1)} TB`;
  if (bytes >= GB) return `${(bytes / GB).toFixed(1)} GB`;
  if (bytes >= MB) return `${(bytes / MB).toFixed(1)} MB`;
  if (bytes >= KB) return `${(bytes / KB).toFixed(1)} KB`;
  return `${bytes} B`;
}

/** 格式化下载/上传速度 */
export function formatSpeed(bytesPerSec: number): string {
  if (!Number.isFinite(bytesPerSec) || bytesPerSec <= 0) return "0 B/s";
  return `${formatSize(bytesPerSec)}/s`;
}

/** 格式化剩余时间 */
export function formatEta(seconds: number | null): string {
  if (seconds === null || seconds === undefined || !Number.isFinite(seconds)) return "∞";
  if (seconds < 0) return "--";
  if (seconds === 0) return "0s";

  if (seconds < 60) return `${Math.floor(seconds)}s`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}m${secs}s`;
  }

  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return `${hours}h${mins}m`;
}

/** 格式化进度百分比 */
export function formatProgress(progress: number): string {
  if (!Number.isFinite(progress)) return "0.0%";
  return `${(progress * 100).toFixed(1)}%`;
}

/** 格式化日期时间 */
export function formatDateTime(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    const lang = useI18nStore.getState().language;
    return date.toLocaleString(lang === "en" ? "en-US" : "zh-CN", {
      year: "numeric",
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return dateStr;
  }
}

/** 根据协议类型返回对应的标签 */
export function protocolLabel(protocol: string): string {
  const locale = getLocale();
  const map: Record<string, string> = {
    HTTP: "HTTP",
    FTP: "FTP",
    BT: "BT",
    MAGNET: locale.protocol.magnet,
    ED2K: "ed2k",
    HLS: "HLS",
    DASH: "DASH",
  };
  return map[protocol] || protocol;
}

/** 检测链接是否为有效的下载链接 */
export function isValidDownloadUrl(url: string): boolean {
  if (!url || url.trim().length === 0) return false;
  const trimmed = url.trim();
  // 支持的协议前缀
  const validPrefixes = [
    "http://",
    "https://",
    "ftp://",
    "ftps://",
    "magnet:",
    "ed2k://",
    "file://",
  ];
  if (validPrefixes.some((p) => trimmed.toLowerCase().startsWith(p))) return true;
  // 支持 m3u8/mpd 流媒体链接（URL 路径以 .m3u8/.mpd 结尾或带查询参数）
  try {
    const url = new URL(trimmed);
    const pathname = url.pathname.toLowerCase();
    if (pathname.endsWith(".m3u8") || pathname.endsWith(".mpd")) return true;
  } catch {
    // not a valid URL, skip
  }
  // 支持 .torrent 文件路径
  if (trimmed.toLowerCase().endsWith(".torrent")) return true;
  return false;
}

/** 格式化时间区间（秒 → 可读时长） */
export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "--";
  const locale = getLocale();
  const f = locale.format;
  if (seconds < 60) return `${Math.floor(seconds)}${f.seconds}`;
  if (seconds < 3600) {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return secs > 0 ? `${mins}${f.minutes}${secs}${f.seconds}` : `${mins}${f.minutes}`;
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  return mins > 0 ? `${hours}${f.hours}${mins}${f.minutes}` : `${hours}${f.hours}`;
}

/** 根据任务状态返回对应的状态标签 */
export function stateLabel(state: string): string {
  const locale = getLocale();
  const map: Record<string, string> = {
    queued: locale.taskState.queued,
    downloading: locale.taskState.downloading,
    paused: locale.taskState.paused,
    seeding: locale.taskState.seeding,
    done: locale.taskState.done,
    error: locale.taskState.error,
  };
  return map[state] || state;
}

/** 下载历史记录（与 DownloadHistory 类型兼容的最小接口） */
interface HistoryLike {
  name?: string | null;
  url?: string | null;
  protocol: string;
  totalSize?: number;
  downloaded?: number;
  averageSpeed?: number;
  duration?: number;
}

/** 按关键词过滤下载历史（匹配文件名、URL、协议） */
export function filterHistory<T extends HistoryLike>(items: T[], query: string): T[] {
  if (!query.trim()) return items;
  const q = query.toLowerCase();
  return items.filter(
    (h) =>
      (h.name || "").toLowerCase().includes(q) ||
      (h.url || "").toLowerCase().includes(q) ||
      protocolLabel(h.protocol).toLowerCase().includes(q)
  );
}

/** 下载历史统计 */
export interface HistoryStats {
  count: number;
  totalSize: number;
  totalDownloaded: number;
  avgSpeed: number;
  totalDuration: number;
}

/** 计算下载历史统计摘要 */
export function computeHistoryStats<T extends HistoryLike>(items: T[]): HistoryStats {
  const count = items.length;
  if (count === 0) return { count: 0, totalSize: 0, totalDownloaded: 0, avgSpeed: 0, totalDuration: 0 };
  const totalSize = items.reduce((s, i) => s + (i.totalSize || 0), 0);
  const totalDownloaded = items.reduce((s, i) => s + (i.downloaded || 0), 0);
  const avgSpeed = items.reduce((s, i) => s + (i.averageSpeed || 0), 0) / count;
  const totalDuration = items.reduce((s, i) => s + (i.duration || 0), 0);
  return { count, totalSize, totalDownloaded, avgSpeed, totalDuration };
}

/** 每日流量统计 */
export interface DailyTraffic {
  date: string;       // YYYY-MM-DD
  downloads: number;  // 任务数
  totalSize: number;  // 总大小 (bytes)
}

/** 按天聚合下载历史（最近 N 天） */
export function aggregateByDay<T extends { completedAt?: string | null; totalSize?: number }>(
  items: T[],
  days: number = 30
): DailyTraffic[] {
  const now = new Date();
  const cutoff = new Date(now);
  cutoff.setDate(cutoff.getDate() - days);

  // 初始化每天的计数器（包含今天）
  const map = new Map<string, { downloads: number; totalSize: number }>();
  for (let i = 0; i <= days; i++) {
    const d = new Date(cutoff);
    d.setDate(d.getDate() + i);
    const key = d.toISOString().slice(0, 10);
    map.set(key, { downloads: 0, totalSize: 0 });
  }

  // 聚合
  for (const item of items) {
    if (!item.completedAt) continue;
    const dateKey = item.completedAt.slice(0, 10);
    const entry = map.get(dateKey);
    if (entry) {
      entry.downloads++;
      entry.totalSize += item.totalSize || 0;
    }
  }

  return Array.from(map.entries()).map(([date, data]) => ({
    date,
    downloads: data.downloads,
    totalSize: data.totalSize,
  }));
}
