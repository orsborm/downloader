// WebUI 主组件
// 响应式布局，适配桌面/平板/手机
// 通过 JSON-RPC API 与下载器后端通信

import { useState, useEffect, useCallback, useRef } from "react";
import {
  addTask,
  tellActive,
  tellWaiting,
  tellStopped,
  getVersion,
  getGlobalStat,
  pauseTask,
  resumeTask,
  removeTask,
  pauseAll,
  unpauseAll,
  purgeCompleted,
  createWebSocket,
} from "./lib/rpc-api";
import type { TaskInfo, GlobalStat } from "@shared/types";

/** 任务标签页 */
type TaskTab = "active" | "waiting" | "stopped";

export function WebUIApp() {
  const [tasks, setTasks] = useState<TaskInfo[]>([]);
  const [version, setVersion] = useState<string>("");
  const [connected, setConnected] = useState(false);
  const [addUrl, setAddUrl] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TaskTab>("active");
  const [globalStat, setGlobalStat] = useState<GlobalStat | null>(null);
  const [selectedTask, setSelectedTask] = useState<TaskInfo | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

  // 刷新当前标签页任务
  const refreshTasks = useCallback(async () => {
    try {
      let data: Record<string, unknown>[];
      switch (activeTab) {
        case "active":
          data = await tellActive();
          break;
        case "waiting":
          data = await tellWaiting();
          break;
        case "stopped":
          data = await tellStopped();
          break;
      }
      setTasks(data as unknown as TaskInfo[]);

      const stat = await getGlobalStat();
      setGlobalStat(stat as unknown as GlobalStat);
    } catch {
      // ignore
    }
  }, [activeTab]);

  // 初始化连接
  useEffect(() => {
    const init = async () => {
      try {
        const v = await getVersion();
        setVersion(v.version);
        setConnected(true);
        await refreshTasks();
      } catch {
        setConnected(false);
      }
    };

    init();

    const ws = createWebSocket((event) => {
      if (
        event.event === "task-state-changed" ||
        event.event === "task-completed" ||
        event.event === "task-progress"
      ) {
        refreshTasks();
      }
    });
    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, [refreshTasks]);

  // 切换标签时刷新
  useEffect(() => {
    if (connected) refreshTasks();
  }, [activeTab, connected, refreshTasks]);

  // 添加任务
  const handleAddTask = useCallback(async () => {
    if (!addUrl.trim()) return;
    try {
      await addTask([addUrl.trim()]);
      setAddUrl("");
      setActiveTab("active");
      await refreshTasks();
    } catch (e) {
      setError(`添加失败: ${e}`);
      setTimeout(() => setError(null), 5000);
    }
  }, [addUrl, refreshTasks]);

  // 任务操作
  const handlePause = useCallback(
    async (taskId: string) => {
      try {
        await pauseTask(taskId);
        await refreshTasks();
      } catch (e) {
        alert(`暂停失败: ${e}`);
      }
    },
    [refreshTasks]
  );

  const handleResume = useCallback(
    async (taskId: string) => {
      try {
        await resumeTask(taskId);
        await refreshTasks();
      } catch (e) {
        alert(`恢复失败: ${e}`);
      }
    },
    [refreshTasks]
  );

  const handleDelete = useCallback(
    async (taskId: string) => {
      if (!confirm("确定删除此任务？")) return;
      try {
        await removeTask(taskId);
        if (selectedTask?.taskId === taskId) setSelectedTask(null);
        await refreshTasks();
      } catch (e) {
        alert(`删除失败: ${e}`);
      }
    },
    [refreshTasks, selectedTask]
  );

  // 全局操作
  const handlePauseAll = useCallback(async () => {
    await pauseAll();
    await refreshTasks();
  }, [refreshTasks]);

  const handleResumeAll = useCallback(async () => {
    await unpauseAll();
    await refreshTasks();
  }, [refreshTasks]);

  const handlePurge = useCallback(async () => {
    await purgeCompleted();
    await refreshTasks();
  }, [refreshTasks]);

  const tabs: { key: TaskTab; label: string; count: number }[] = [
    { key: "active", label: "下载中", count: globalStat?.numActive ?? 0 },
    { key: "waiting", label: "等待中", count: globalStat?.numWaiting ?? 0 },
    { key: "stopped", label: "已完成", count: globalStat?.numStopped ?? 0 },
  ];

  return (
    <div className="min-h-screen bg-slate-50 text-slate-800">
      {/* 顶部导航 */}
      <header className="bg-blue-600 text-white px-4 py-3 flex items-center justify-between shadow-md">
        <div className="flex items-center gap-3">
          <h1 className="text-lg font-semibold">全协议下载器</h1>
          {globalStat && (
            <div className="text-xs text-blue-200 flex gap-3">
              <span>{formatSpeed(globalStat.downloadSpeed)}</span>
              <span>{formatSpeed(globalStat.uploadSpeed)} UL</span>
            </div>
          )}
        </div>
        <div className="flex items-center gap-3">
          <div className="flex gap-1">
            <button
              onClick={handlePauseAll}
              className="px-2 py-1 text-xs bg-blue-500 rounded hover:bg-blue-400 transition-colors"
              title="全部暂停"
            >
              暂停全部
            </button>
            <button
              onClick={handleResumeAll}
              className="px-2 py-1 text-xs bg-blue-500 rounded hover:bg-blue-400 transition-colors"
              title="全部恢复"
            >
              恢复全部
            </button>
            <button
              onClick={handlePurge}
              className="px-2 py-1 text-xs bg-blue-500 rounded hover:bg-blue-400 transition-colors"
              title="清除已完成"
            >
              清除
            </button>
          </div>
          <div className="text-sm">
            {connected ? (
              <span className="text-green-200">v{version}</span>
            ) : (
              <span className="text-red-200">未连接</span>
            )}
          </div>
        </div>
      </header>

      {/* 错误提示 */}
      {error && (
        <div className="bg-red-50 border-l-4 border-red-400 p-3 mx-4 mt-2 rounded">
          <p className="text-sm text-red-700">{error}</p>
        </div>
      )}

      {/* 添加任务 */}
      <div className="p-4 max-w-5xl mx-auto">
        <div className="flex gap-2 mb-4">
          <input
            type="text"
            value={addUrl}
            onChange={(e) => setAddUrl(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && handleAddTask()}
            placeholder="输入下载链接（HTTP/FTP/Magnet/ed2k）..."
            className="flex-1 px-3 py-2 border border-slate-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <button
            onClick={handleAddTask}
            className="px-5 py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 active:bg-blue-800 transition-colors"
          >
            添加下载
          </button>
        </div>

        {/* 标签页 */}
        <div className="flex border-b border-slate-200 mb-3">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => {
                setActiveTab(tab.key);
                setSelectedTask(null);
              }}
              className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab.key
                  ? "border-blue-600 text-blue-600"
                  : "border-transparent text-slate-500 hover:text-slate-700"
              }`}
            >
              {tab.label}
              {tab.count > 0 && (
                <span
                  className={`ml-1.5 px-1.5 py-0.5 text-xs rounded-full ${
                    activeTab === tab.key
                      ? "bg-blue-100 text-blue-600"
                      : "bg-slate-100 text-slate-500"
                  }`}
                >
                  {tab.count}
                </span>
              )}
            </button>
          ))}
        </div>

        {/* 任务列表 */}
        <div className="bg-white rounded-lg shadow overflow-hidden">
          <div className="grid grid-cols-12 gap-2 px-4 py-2 bg-slate-50 text-xs font-medium text-slate-500 uppercase tracking-wider">
            <div className="col-span-5">文件名</div>
            <div className="col-span-2 text-right">大小</div>
            <div className="col-span-2">进度</div>
            <div className="col-span-1 text-right">速度</div>
            <div className="col-span-2 text-right">操作</div>
          </div>

          {tasks.length === 0 ? (
            <div className="text-center text-slate-400 py-16 text-sm">
              {connected
                ? activeTab === "active"
                  ? "暂无下载中的任务"
                  : activeTab === "waiting"
                  ? "暂无等待中的任务"
                  : "暂无已完成的任务"
                : "连接中..."}
            </div>
          ) : (
            tasks.map((task) => (
              <TaskRow
                key={task.taskId}
                task={task}
                selected={selectedTask?.taskId === task.taskId}
                onSelect={() =>
                  setSelectedTask(
                    selectedTask?.taskId === task.taskId ? null : task
                  )
                }
                onPause={() => handlePause(task.taskId)}
                onResume={() => handleResume(task.taskId)}
                onDelete={() => handleDelete(task.taskId)}
              />
            ))
          )}
        </div>

        {/* 任务详情 */}
        {selectedTask && (
          <TaskDetailPanel
            task={selectedTask}
            onClose={() => setSelectedTask(null)}
          />
        )}
      </div>
    </div>
  );
}

/** 任务行 */
function TaskRow({
  task,
  selected,
  onSelect,
  onPause,
  onResume,
  onDelete,
}: {
  task: TaskInfo;
  selected: boolean;
  onSelect: () => void;
  onPause: () => void;
  onResume: () => void;
  onDelete: () => void;
}) {
  const isActive = task.status === "active" || task.status === "downloading";
  const isPaused = task.status === "paused";
  const isCompleted = task.status === "completed";
  const isError = task.status === "error";

  return (
    <div
      onClick={onSelect}
      className={`grid grid-cols-12 gap-2 px-4 py-3 border-t text-sm cursor-pointer transition-colors ${
        selected
          ? "bg-blue-50 border-l-2 border-l-blue-500"
          : "hover:bg-slate-50"
      }`}
    >
      <div className="col-span-5 truncate flex items-center gap-2">
        <StatusIcon status={task.status} />
        <span className="truncate">{task.name || task.taskId.slice(0, 8)}</span>
      </div>
      <div className="col-span-2 text-right text-slate-500 font-mono text-xs">
        <div>{formatSize(task.totalLength)}</div>
        <div className="text-slate-400">{formatSize(task.completedLength)}</div>
      </div>
      <div className="col-span-2">
        <div className="h-2 bg-slate-200 rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all ${
              isError
                ? "bg-red-500"
                : isCompleted
                ? "bg-green-500"
                : isPaused
                ? "bg-yellow-500"
                : "bg-blue-500"
            }`}
            style={{ width: `${(task.progress * 100).toFixed(1)}%` }}
          />
        </div>
        <div className="text-xs text-slate-400 mt-0.5">
          {(task.progress * 100).toFixed(1)}%
        </div>
      </div>
      <div className="col-span-1 text-right text-slate-500 font-mono text-xs">
        {isActive ? formatSpeed(task.downloadSpeed) : "-"}
      </div>
      <div className="col-span-2 text-right flex items-center justify-end gap-1">
        {isActive && (
          <ActionBtn onClick={onPause} title="暂停">
            <PauseIcon />
          </ActionBtn>
        )}
        {(isPaused || isError) && (
          <ActionBtn onClick={onResume} title="恢复">
            <PlayIcon />
          </ActionBtn>
        )}
        <ActionBtn onClick={onDelete} title="删除" danger>
          <TrashIcon />
        </ActionBtn>
      </div>
    </div>
  );
}

/** 任务详情面板 */
function TaskDetailPanel({
  task,
  onClose,
}: {
  task: TaskInfo;
  onClose: () => void;
}) {
  return (
    <div className="mt-3 bg-white rounded-lg shadow p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-medium text-sm truncate flex-1">{task.name}</h3>
        <button
          onClick={onClose}
          className="text-slate-400 hover:text-slate-600 ml-2"
        >
          ✕
        </button>
      </div>
      <div className="grid grid-cols-2 gap-x-6 gap-y-2 text-xs">
        <DetailRow label="任务 ID" value={task.taskId.slice(0, 12) + "..."} />
        <DetailRow label="状态" value={statusLabel(task.status)} />
        <DetailRow label="总大小" value={formatSize(task.totalLength)} />
        <DetailRow label="已下载" value={formatSize(task.completedLength)} />
        <DetailRow label="下载速度" value={formatSpeed(task.downloadSpeed)} />
        <DetailRow label="上传速度" value={formatSpeed(task.uploadSpeed)} />
        <DetailRow label="连接数" value={String(task.connections)} />
        <DetailRow
          label="剩余时间"
          value={task.eta > 0 ? formatEta(task.eta) : "-"}
        />
        {task.dir && <DetailRow label="保存目录" value={task.dir} />}
      </div>
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex gap-2">
      <span className="text-slate-400 w-16 shrink-0">{label}</span>
      <span className="text-slate-700 truncate">{value}</span>
    </div>
  );
}

/** 状态图标 */
function StatusIcon({ status }: { status: string }) {
  const colors: Record<string, string> = {
    active: "bg-blue-500",
    downloading: "bg-blue-500",
    paused: "bg-yellow-500",
    waiting: "bg-slate-400",
    completed: "bg-green-500",
    error: "bg-red-500",
    seeding: "bg-purple-500",
  };
  return (
    <span
      className={`w-2 h-2 rounded-full shrink-0 ${
        colors[status] || "bg-slate-300"
      }`}
    />
  );
}

function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    active: "下载中",
    downloading: "下载中",
    paused: "已暂停",
    waiting: "等待中",
    completed: "已完成",
    error: "出错",
    seeding: "做种中",
  };
  return labels[status] || status;
}

/** 操作按钮 */
function ActionBtn({
  onClick,
  title,
  danger,
  children,
}: {
  onClick: (e: React.MouseEvent) => void;
  title: string;
  danger?: boolean;
  children: React.ReactNode;
}) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation();
        onClick(e);
      }}
      title={title}
      className={`p-1.5 rounded transition-colors ${
        danger
          ? "hover:bg-red-50 text-slate-400 hover:text-red-500"
          : "hover:bg-slate-100 text-slate-400 hover:text-slate-600"
      }`}
    >
      {children}
    </button>
  );
}

// ==================== SVG 图标 ====================

function PauseIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
      <rect x="6" y="4" width="4" height="16" />
      <rect x="14" y="4" width="4" height="16" />
    </svg>
  );
}

function PlayIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
      <path d="M8 5v14l11-7z" />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <polyline points="3 6 5 6 21 6" />
      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
    </svg>
  );
}

// 格式化工具从共享模块导入
import { formatSize as _formatSize, formatSpeed as _formatSpeed, formatEta as _formatEta } from "@shared/format";

function formatSize(bytes: number): string {
  return _formatSize(bytes);
}

function formatSpeed(bytesPerSec: number): string {
  return _formatSpeed(bytesPerSec);
}

function formatEta(seconds: number): string {
  return seconds > 0 ? _formatEta(seconds) : "-";
}
