// 任务详情面板组件
// 选中任务时在底部展开，包含：概要、文件、连接、Tracker、日志、镜像 Tab

import { useState, useEffect, useCallback } from "react";
import {
  Info,
  FileText,
  Users,
  Radio,
  ScrollText,
  Globe,
  Plus,
  Trash2,
} from "lucide-react";
import type { TaskStatus } from "../lib/types";
import { useI18n } from "../hooks/useI18n";
import { showToast } from "./Toast";
import { extractErrorMessage } from "../lib/errors";
import { getTaskPeers, getTaskTrackers, getTaskLogs, getMirrorUrls, addMirrorUrl, removeMirrorUrl } from "../lib/tauri-api";
import type { PeerInfo, TrackerInfo, LogEntry } from "../lib/tauri-api";
import {
  formatSize,
  formatSpeed,
  formatDateTime,
  formatDuration,
  protocolLabel,
  stateLabel,
} from "../lib/format";

interface TaskDetailProps {
  task: TaskStatus;
}

type TabKey = "summary" | "files" | "connections" | "tracker" | "logs" | "mirrors";

export function TaskDetail({ task }: TaskDetailProps) {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<TabKey>("summary");

  // 切换任务时重置到概要 Tab
  useEffect(() => {
    setActiveTab("summary");
  }, [task.id]);

  // Tab 定义（使用 i18n 翻译）
  const tabs: { key: TabKey; label: string; icon: React.ReactNode }[] = [
    { key: "summary", label: t("taskDetail.tabs.overview"), icon: <Info size={14} /> },
    { key: "files", label: t("taskDetail.tabs.files"), icon: <FileText size={14} /> },
    { key: "connections", label: t("taskDetail.tabs.connections"), icon: <Users size={14} /> },
    { key: "tracker", label: t("taskDetail.tabs.tracker"), icon: <Radio size={14} /> },
    { key: "logs", label: t("taskDetail.tabs.logs"), icon: <ScrollText size={14} /> },
    { key: "mirrors", label: t("taskDetail.tabs.mirrors"), icon: <Globe size={14} /> },
  ];

  return (
    <div className="h-full flex flex-col bg-secondary">
      {/* Tab 导航 */}
      <div className="flex items-center border-b border-border px-2">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => setActiveTab(tab.key)}
            className={`flex items-center gap-1 px-3 py-2 text-sm transition-colors
              ${
                activeTab === tab.key
                  ? "text-accent border-b-2 border-accent"
                  : "text-text-secondary hover:text-text-primary hover:bg-tertiary/50"
              }`}
          >
            {tab.icon}
            <span>{tab.label}</span>
          </button>
        ))}
      </div>

      {/* Tab 内容 */}
      <div className="flex-1 overflow-auto p-3">
        {activeTab === "summary" && <SummaryTab task={task} />}
        {activeTab === "files" && <FilesTab task={task} />}
        {activeTab === "connections" && <ConnectionsTab task={task} />}
        {activeTab === "tracker" && <TrackerTab task={task} />}
        {activeTab === "logs" && <LogsTab task={task} />}
        {activeTab === "mirrors" && <MirrorTab task={task} />}
      </div>
    </div>
  );
}

/** 概要 Tab */
function SummaryTab({ task }: { task: TaskStatus }) {
  const { t } = useI18n();
  return (
    <div className="grid grid-cols-2 gap-x-8 gap-y-2 text-sm">
      <InfoRow label={t("taskDetail.fields.taskName")} value={task.name} />
      <InfoRow label={t("taskDetail.fields.protocol")} value={protocolLabel(task.protocol)} />
      <InfoRow label={t("taskDetail.fields.status")} value={stateLabel(task.state)} />
      <InfoRow label={t("taskDetail.fields.savePath")} value={task.savePath} />
      <InfoRow
        label={t("taskDetail.fields.totalSize")}
        value={task.totalSize > 0 ? formatSize(task.totalSize) : t("taskDetail.unknown")}
      />
      <InfoRow label={t("taskDetail.fields.downloaded")} value={formatSize(task.downloaded)} />
      <InfoRow label={t("taskDetail.fields.downloadSpeed")} value={formatSpeed(task.downloadSpeed)} />
      <InfoRow label={t("taskDetail.fields.uploadSpeed")} value={formatSpeed(task.uploadSpeed)} />
      <InfoRow label={t("taskDetail.fields.peers")} value={String(task.peers)} />
      <InfoRow label={t("taskDetail.fields.url")} value={task.url} className="col-span-2 truncate" title={task.url} />
      <InfoRow label={t("taskDetail.fields.addedAt")} value={formatDateTime(task.addedAt)} />
      {task.completedAt && (
        <>
          <InfoRow label={t("taskDetail.fields.completedAt")} value={formatDateTime(task.completedAt)} />
          <InfoRow
            label={t("taskDetail.fields.duration")}
            value={formatDuration(
              (new Date(task.completedAt).getTime() - new Date(task.addedAt).getTime()) / 1000
            )}
          />
        </>
      )}
      {task.error && (
        <InfoRow label={t("taskDetail.fields.error")} value={task.error} className="text-error" />
      )}
    </div>
  );
}

/** 文件 Tab */
function FilesTab({ task }: { task: TaskStatus }) {
  const { t } = useI18n();

  if (task.files.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.noFiles")}
      </div>
    );
  }

  return (
    <div className="text-sm">
      <div className="flex items-center gap-2 px-2 py-1 text-xs text-text-muted font-medium border-b border-border">
        <div className="flex-1">{t("taskDetail.files.name")}</div>
        <div className="w-24 text-right">{t("taskDetail.files.size")}</div>
        <div className="w-16 text-center">{t("taskDetail.files.priority")}</div>
      </div>
      {task.files.map((file) => (
        <div
          key={file.index}
          className="flex items-center gap-2 px-2 py-1.5 border-b border-border/50 hover:bg-tertiary"
        >
          <div className="flex-1 truncate" title={file.path}>{file.path}</div>
          <div className="w-24 text-right text-text-secondary font-mono text-xs">
            {formatSize(file.size)}
          </div>
          <div className="w-16 text-center text-text-secondary">
            {file.priority === 0 ? t("taskDetail.files.skip") : file.priority === 2 ? t("taskDetail.files.high") : t("taskDetail.files.normal")}
          </div>
        </div>
      ))}
    </div>
  );
}

/** 连接 Tab */
function ConnectionsTab({ task }: { task: TaskStatus }) {
  const { t } = useI18n();
  const [peers, setPeers] = useState<PeerInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchPeers = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getTaskPeers(task.id);
      setPeers(data);
    } catch {
      setPeers([]);
    } finally {
      setLoading(false);
    }
  }, [task.id]);

  useEffect(() => {
    if (task.state === "downloading" || task.state === "seeding") {
      fetchPeers();
      const interval = setInterval(fetchPeers, 5000);
      return () => clearInterval(interval);
    }
  }, [task.id, task.state, fetchPeers]);

  if (task.state === "queued") {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.connections.waitingForDownload")}
      </div>
    );
  }
  if (task.state === "paused") {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskState.paused")}
      </div>
    );
  }

  if (loading && peers.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("common.loading")}
      </div>
    );
  }

  if (peers.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.noConnections")}
      </div>
    );
  }

  return (
    <div className="text-sm">
      <div className="flex items-center gap-2 px-2 py-1 text-xs text-text-muted font-medium border-b border-border">
        <div className="flex-1">Peer</div>
        <div className="w-24 text-right">{t("taskDetail.fields.downloadSpeed")}</div>
        <div className="w-24 text-right">{t("taskDetail.fields.uploadSpeed")}</div>
        <div className="w-16 text-right">{t("taskDetail.fields.progress")}</div>
      </div>
      {peers.map((peer, idx) => (
        <div key={peer.id || idx} className="flex items-center gap-2 px-2 py-1 border-b border-border/50">
          <div className="flex-1 truncate" title={peer.client || peer.ip}>{peer.client || peer.ip}</div>
          <div className="w-24 text-right text-green-500">{formatSpeed(peer.downloadSpeed)}</div>
          <div className="w-24 text-right text-blue-500">{formatSpeed(peer.uploadSpeed)}</div>
          <div className="w-16 text-right">{(peer.progress * 100).toFixed(1)}%</div>
        </div>
      ))}
    </div>
  );
}

/** Tracker Tab */
function TrackerTab({ task }: { task: TaskStatus }) {
  const { t } = useI18n();
  const [trackers, setTrackers] = useState<TrackerInfo[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchTrackers = useCallback(async () => {
    setLoading(true);
    try {
      const data = await getTaskTrackers(task.id);
      setTrackers(data);
    } catch {
      setTrackers([]);
    } finally {
      setLoading(false);
    }
  }, [task.id]);

  // 自动刷新 tracker 数据（每 5 秒）
  useEffect(() => {
    if (task.state === "downloading" || task.state === "seeding") {
      fetchTrackers();
      const interval = setInterval(fetchTrackers, 5000);
      return () => clearInterval(interval);
    }
  }, [task.id, task.state, fetchTrackers]);

  if (task.state === "queued") {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.connections.waitingForDownload")}
      </div>
    );
  }
  if (task.state === "paused") {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskState.paused")}
      </div>
    );
  }

  if (loading && trackers.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("common.loading")}
      </div>
    );
  }

  if (trackers.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.noTrackers")}
      </div>
    );
  }

  return (
    <div className="text-sm">
      <div className="flex items-center gap-2 px-2 py-1 text-xs text-text-muted font-medium border-b border-border">
        <div className="flex-1">Tracker</div>
        <div className="w-20 text-center">{t("taskDetail.fields.status")}</div>
        <div className="w-20 text-right">S/L</div>
      </div>
      {trackers.map((tracker, idx) => {
        const statusInfo: Record<string, { label: string; color: string }> = {
          connected: { label: t("statusBar.dhtConnected"), color: "text-success" },
          updating: { label: "...", color: "text-warning" },
          error: { label: t("taskState.error"), color: "text-error" },
        };
        const info = statusInfo[tracker.status] || { label: tracker.status, color: "text-text-muted" };
        return (
          <div key={idx} className="flex items-center gap-2 px-2 py-1 border-b border-border/50">
            <div className="flex-1 truncate text-xs" title={tracker.url}>{tracker.url}</div>
            <div className={`w-20 text-center text-xs ${info.color}`}>{info.label}</div>
            <div className="w-20 text-right text-xs text-text-muted">
              {tracker.seeders > 0 || tracker.leechers > 0
                ? `${tracker.seeders}/${tracker.leechers}`
                : tracker.peers > 0 ? `${tracker.peers}` : "--"}
            </div>
          </div>
        );
      })}
    </div>
  );
}

/** 日志 Tab */
function LogsTab({ task }: { task: TaskStatus }) {
  const { t, language } = useI18n();
  const locale = language === "zh" ? "zh-CN" : "en-US";
  const [backendLogs, setBackendLogs] = useState<LogEntry[]>([]);

  useEffect(() => {
    getTaskLogs(task.id, 50).then(setBackendLogs).catch(() => setBackendLogs([]));
  }, [task.id]);

  // 合并后端日志和前端生成的日志（使用稳定时间戳，不因重渲染改变）
  const logs: { time: string; level: string; message: string }[] = [];

  // 优先显示后端日志
  for (const log of backendLogs) {
    logs.push({ time: log.timestamp, level: log.level.toUpperCase(), message: log.message });
  }

  // 补充前端状态日志（使用任务时间而非当前时间）
  const taskTime = task.addedAt
    ? new Date(task.addedAt).toLocaleTimeString(locale)
    : new Date().toLocaleTimeString(locale);

  if (task.error) {
    logs.push({ time: taskTime, level: "ERROR", message: task.error });
  }

  if (task.state === "done") {
    logs.push({
      time: task.completedAt
        ? new Date(task.completedAt).toLocaleTimeString(locale)
        : taskTime,
      level: "INFO",
      message: t("taskDetail.logs.downloadComplete"),
    });
  }

  if (task.state === "downloading" || task.state === "seeding") {
    logs.push({
      time: taskTime,
      level: "INFO",
      message: `${t("taskDetail.logs.downloading")} - ${formatSpeed(task.downloadSpeed)} - ${formatSize(task.downloaded)}/${task.totalSize > 0 ? formatSize(task.totalSize) : t("taskDetail.unknown")}`,
    });
  }

  if (task.state === "paused") {
    logs.push({
      time: taskTime,
      level: "INFO",
      message: `${t("taskDetail.logs.paused")} - ${formatSize(task.downloaded)} ${t("taskDetail.logs.downloaded")}`,
    });
  }

  if (logs.length === 0) {
    return (
      <div className="text-sm text-text-muted text-center py-8">
        {t("taskDetail.noLogs")}
      </div>
    );
  }

  return (
    <div className="text-sm font-mono">
      {logs.map((log, i) => (
        <div
          key={i}
          className="flex gap-2 px-2 py-1 border-b border-border/50"
        >
          <span className="text-text-muted w-20 shrink-0">{log.time}</span>
          <span
            className={`w-14 shrink-0 ${
              log.level === "ERROR"
                ? "text-error"
                : log.level === "WARN"
                  ? "text-warning"
                  : "text-text-secondary"
            }`}
          >
            {log.level}
          </span>
          <span className="text-text-primary">{log.message}</span>
        </div>
      ))}
    </div>
  );
}

/** 镜像源管理 Tab */
function MirrorTab({ task }: { task: TaskStatus }) {
  const { t } = useI18n();
  const [mirrors, setMirrors] = useState<string[]>([]);
  const [newUrl, setNewUrl] = useState("");
  const [loading, setLoading] = useState(false);

  const fetchMirrors = useCallback(async () => {
    try {
      const data = await getMirrorUrls(task.id);
      setMirrors(data);
    } catch {
      setMirrors([]);
    }
  }, [task.id]);

  useEffect(() => {
    fetchMirrors();
  }, [fetchMirrors]);

  const handleAdd = useCallback(async () => {
    if (!newUrl.trim()) return;
    if (!newUrl.startsWith("http://") && !newUrl.startsWith("https://")) {
      showToast(t("taskDetail.mirrors.invalidUrl") || "请输入 http:// 或 https:// 开头的 URL", "error");
      return;
    }

    setLoading(true);
    try {
      await addMirrorUrl(task.id, newUrl.trim());
      setNewUrl("");
      await fetchMirrors();
    } catch (e) {
      showToast(`${t("common.error")}: ${extractErrorMessage(e)}`, "error");
    } finally {
      setLoading(false);
    }
  }, [task.id, newUrl, fetchMirrors, t]);

  const handleRemove = useCallback(async (url: string) => {
    try {
      await removeMirrorUrl(task.id, url);
      await fetchMirrors();
    } catch (e) {
      showToast(`${t("common.error")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [task.id, fetchMirrors, t]);

  return (
    <div className="space-y-3">
      {/* 添加镜像URL */}
      <div className="flex gap-2">
        <input
          type="text"
          value={newUrl}
          onChange={(e) => setNewUrl(e.target.value)}
          placeholder={t("taskDetail.mirrors.addPlaceholder")}
          className="flex-1 px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm focus:outline-none focus:ring-1 focus:ring-accent/50"
          onKeyDown={(e) => {
            if (e.key === "Enter") handleAdd();
          }}
        />
        <button
          onClick={handleAdd}
          disabled={loading || !newUrl.trim()}
          className="px-3 py-1.5 rounded bg-accent text-white text-sm hover:bg-accent-hover disabled:opacity-50 transition-colors flex items-center gap-1"
        >
          <Plus size={14} />
          {t("taskDetail.mirrors.add")}
        </button>
      </div>

      {/* 镜像URL列表 */}
      {mirrors.length === 0 ? (
        <div className="text-sm text-text-muted text-center py-8">
          {t("taskDetail.mirrors.empty")}
        </div>
      ) : (
        <div className="space-y-1">
          {mirrors.map((url, idx) => (
            <div
              key={idx}
              className="flex items-center gap-2 px-3 py-2 rounded border border-border/50 bg-primary/50"
            >
              <Globe size={14} className="text-text-muted flex-shrink-0" />
              <div className="flex-1 truncate text-sm text-text-primary" title={url}>{url}</div>
              <button
                onClick={() => handleRemove(url)}
                className="p-1 rounded hover:bg-error/10 text-text-muted hover:text-error transition-colors"
              >
                <Trash2 size={14} />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* 说明 */}
      <div className="text-xs text-text-muted">
        {t("taskDetail.mirrors.description")}
      </div>
    </div>
  );
}

/** 信息行（可复用） */
function InfoRow({
  label,
  value,
  className = "",
  title,
}: {
  label: string;
  value: string;
  className?: string;
  title?: string;
}) {
  return (
    <div className={className} title={title}>
      <span className="text-text-muted">{label}: </span>
      <span className="text-text-primary">{value}</span>
    </div>
  );
}
