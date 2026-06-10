// 任务列表组件
// 使用虚拟滚动处理大量任务，支持排序、多选、右键菜单
// 列：状态图标、文件名、协议、大小、进度、速度、来源、剩余时间

import { useVirtualizer } from "@tanstack/react-virtual";
import { useRef, useState, useCallback, useMemo, useEffect } from "react";
import {
  Play,
  Pause,
  Trash2,
  RotateCcw,
  FolderOpen,
  FileText,
  Download,
  RefreshCw,
  File,
  Gauge,
  ArrowUp,
  Minus,
  ArrowDown,
} from "lucide-react";
import { useTaskStore, type SortKey } from "../stores/taskStore";
import { pauseTask, resumeTask, removeTask, openFileLocation, openFile, setTaskPriority, setTaskSpeedLimit, getAllTasks } from "../lib/tauri-api";
import { showToast } from "./Toast";
import {
  formatSize,
  formatSpeed,
  formatEta,
  formatProgress,
  protocolLabel,
} from "../lib/format";
import { withErrorHandling, ErrorCode, extractErrorMessage } from "../lib/errors";
import type { TaskStatus } from "../lib/types";
import { useI18n } from "../hooks/useI18n";

/** 状态图标映射 */
/** 状态图标组件（带颜色圆点 + 图标） */
function StateIcon({ state }: { state: string }) {
  const dotColor: Record<string, string> = {
    downloading: "bg-accent",
    paused: "bg-warning",
    done: "bg-success",
    error: "bg-error",
    seeding: "bg-success",
    queued: "bg-text-muted",
  };
  const iconClass = "w-3.5 h-3.5";
  const stateClass = `state-${state}`;
  const icon = (() => {
    switch (state) {
      case "downloading": return <Play className={`${iconClass} ${stateClass}`} />;
      case "paused": return <Pause className={`${iconClass} ${stateClass}`} />;
      case "done": return <FileText className={`${iconClass} ${stateClass}`} />;
      case "error": return <RotateCcw className={`${iconClass} ${stateClass}`} />;
      case "seeding": return <Play className={`${iconClass} ${stateClass}`} />;
      default: return <Pause className={`${iconClass} ${stateClass}`} />;
    }
  })();

  return (
    <div className="relative flex items-center justify-center w-5 h-5">
      <div className={`absolute bottom-0 right-0 w-2 h-2 rounded-full ${dotColor[state] || "bg-text-muted"} ${state === "downloading" ? "animate-pulse" : ""}`} />
      {icon}
    </div>
  );
}

/** 状态徽章样式映射（模块级常量，避免每次渲染重建） */
const BADGE_CLS: Record<string, string> = {
  paused: "bg-warning/15 text-warning border-warning/30",
  done: "bg-success/15 text-success border-success/30",
  error: "bg-error/15 text-error border-error/30",
  seeding: "bg-success/15 text-success border-success/30",
  queued: "bg-text-muted/15 text-text-muted border-text-muted/30",
};

/** 状态徽章 i18n 键映射 */
const BADGE_I18N: Record<string, string> = {
  paused: "taskState.paused",
  done: "taskState.done",
  error: "taskState.error",
  seeding: "taskState.seeding",
  queued: "taskState.queued",
};

/** 状态徽章（显示在文件名右侧，仅非 downloading 状态显示） */
function StateBadge({ state, error }: { state: string; error?: string | null }) {
  // downloading 状态不显示徽章（已有进度条和动画指示）
  if (state === "downloading") return null;
  const cls = BADGE_CLS[state];
  if (!cls) return null;
  const { t } = useI18n();

  return (
    <span
      className={`inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium border shrink-0 ${cls}`}
      title={state === "error" && error ? error : undefined}
    >
      {state === "error" && error ? error : t(BADGE_I18N[state] || "taskState.queued")}
    </span>
  );
}

/** 状态左边框颜色映射（模块级常量，避免每次渲染重建） */
const STATE_BORDER_CLS: Record<string, string> = {
  downloading: "border-l-accent",
  paused: "border-l-warning",
  done: "border-l-success",
  error: "border-l-error",
  seeding: "border-l-success",
  queued: "border-l-text-muted",
};

/** 协议标签颜色 */
function protocolColor(protocol: string): string {
  switch (protocol) {
    case "HTTP":
      return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
    case "FTP":
      return "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400";
    case "BT":
    case "MAGNET":
      return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400";
    case "ED2K":
      return "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400";
    default:
      return "bg-gray-100 text-gray-700 dark:bg-gray-900/30 dark:text-gray-400";
  }
}

export function TaskList() {
  const { t } = useI18n();
  const getSortedTasks = useTaskStore((s) => s.getSortedTasks);
  const selectedIds = useTaskStore((s) => s.selectedIds);
  const selectTask = useTaskStore((s) => s.selectTask);
  const removeTaskFromStore = useTaskStore((s) => s.removeTask);
  const sortKey = useTaskStore((s) => s.sortKey);
  const sortAsc = useTaskStore((s) => s.sortAsc);
  const setSort = useTaskStore((s) => s.setSort);
  const allTasks = useTaskStore((s) => s.tasks);
  const searchQuery = useTaskStore((s) => s.searchQuery);
  const statusFilter = useTaskStore((s) => s.statusFilter);
  const setSearchQuery = useTaskStore((s) => s.setSearchQuery);
  const setStatusFilter = useTaskStore((s) => s.setStatusFilter);

  const [contextMenu, setContextMenu] = useState<{
    x: number;
    y: number;
    task: TaskStatus;
  } | null>(null);

  const parentRef = useRef<HTMLDivElement>(null);

  // 获取排序后的任务（memoize 避免每次渲染重新排序）
  const tasks = useMemo(() => getSortedTasks(), [allTasks, sortKey, sortAsc, searchQuery, statusFilter]);

  // 虚拟滚动
  const virtualizer = useVirtualizer({
    count: tasks.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 52,
    overscan: 10,
  });

  // 打开文件所在目录
  const handleOpenLocation = useCallback(async (id: string) => {
    await withErrorHandling(
      () => openFileLocation(id),
      t("contextMenu.openFolder"),
      ErrorCode.PERMISSION_DENIED
    );
  }, [t]);

  // 打开右键菜单（右键时自动选中该任务）
  const handleContextMenu = useCallback(
    (e: React.MouseEvent, task: TaskStatus) => {
      e.preventDefault();
      setContextMenu({ x: e.clientX, y: e.clientY, task });
      selectTask(task.id, e.ctrlKey || e.metaKey, false);
    },
    [selectTask]
  );

  // 关闭右键菜单
  const closeContextMenu = useCallback(() => setContextMenu(null), []);

  // Escape 关闭右键菜单
  useEffect(() => {
    if (!contextMenu) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") setContextMenu(null);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [contextMenu]);

  // 暂停任务（后端会发送 task-update 事件自动同步 UI）
  const handlePause = useCallback(async (id: string) => {
    try {
      await pauseTask(id);
    } catch (e) {
      showToast(`${t("app.pauseTask")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [t]);

  // 恢复任务（后端会发送 task-update 事件自动同步 UI）
  const handleResume = useCallback(async (id: string) => {
    try {
      await resumeTask(id);
    } catch (e) {
      showToast(`${t("app.resumeTask")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [t]);

  // 删除任务（成功后从 UI 移除）
  const handleDelete = useCallback(
    async (id: string, deleteFiles: boolean = false) => {
      await withErrorHandling(
        () => removeTask(id, deleteFiles),
        t("app.deleteTask"),
        ErrorCode.DOWNLOAD_FAILED
      );
      // withErrorHandling 返回 undefined 表示成功（void 函数）
      // 返回 undefined 也可能是失败，但 withErrorHandling 内部已处理错误 toast
      // 这里无条件移除：如果后端失败，toast 已提示；如果成功，UI 需要同步
      removeTaskFromStore(id);
    },
    [removeTaskFromStore, t]
  );

  // 排序（委托给 store）
  const handleSort = useCallback(
    (key: SortKey) => {
      setSort(key);
    },
    [setSort]
  );

  return (
    <div className="h-full flex flex-col" onClick={closeContextMenu}>
      {/* 表头（列宽与数据行一致） */}
      <div className="flex items-center px-3 py-2 text-xs font-medium text-text-secondary bg-secondary border-b border-border select-none">
        <div className="w-8 text-center">{t("taskList.columns.status")}</div>
        <div className="flex-1 min-w-0 cursor-pointer hover:text-text-primary transition-colors" onClick={() => handleSort("name")}>
          {t("taskList.columns.name")} {sortKey === "name" && (sortAsc ? "↑" : "↓")}
        </div>
        <div className="w-16 text-center">{t("taskList.columns.protocol")}</div>
        <div className="w-20 text-right cursor-pointer hover:text-text-primary transition-colors" onClick={() => handleSort("size")}>
          {t("taskList.columns.size")} {sortKey === "size" && (sortAsc ? "↑" : "↓")}
        </div>
        <div className="w-36 text-center cursor-pointer hover:text-text-primary transition-colors" onClick={() => handleSort("progress")}>
          {t("taskList.columns.progress")} {sortKey === "progress" && (sortAsc ? "↑" : "↓")}
        </div>
        <div className="w-24 text-right cursor-pointer hover:text-text-primary transition-colors" onClick={() => handleSort("speed")}>
          {t("taskList.columns.speed")} {sortKey === "speed" && (sortAsc ? "↑" : "↓")}
        </div>
        <div className="w-16 text-center">{t("taskList.columns.peers")}</div>
        <div className="w-20 text-right">{t("taskList.columns.eta")}</div>
      </div>

      {/* 任务列表（虚拟滚动） */}
      <div ref={parentRef} className="flex-1 overflow-auto">
        {tasks.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full text-text-muted">
            <Download size={48} className="mb-3 opacity-30" />
            {allTasks.size === 0 ? (
              <>
                <p className="text-sm">{t("taskList.empty")}</p>
                <p className="text-xs mt-1">{t("taskList.emptyHint")}</p>
              </>
            ) : (
              <>
                <p className="text-sm">{t("taskList.noMatch")}</p>
                <p className="text-xs mt-1">
                  {searchQuery && `${t("common.search")} "${searchQuery}" `}
                  {statusFilter !== "all" && `${t("taskList.columns.status")}: ${statusFilter} `}
                  {t("taskList.noResult")}
                </p>
                <button
                  onClick={() => {
                    setSearchQuery("");
                    setStatusFilter("all");
                  }}
                  className="mt-2 text-xs text-accent hover:underline"
                >
                  {t("taskList.clearFilters")}
                </button>
              </>
            )}
          </div>
        )}
        <div
          style={{ height: `${virtualizer.getTotalSize()}px`, position: "relative" }}
        >
          {virtualizer.getVirtualItems().map((virtualItem) => {
            const task = tasks[virtualItem.index];
            const isSelected = selectedIds.has(task.id);

            return (
              <div
                key={task.id}
                data-index={virtualItem.index}
                onClick={(e) => selectTask(task.id, e.ctrlKey || e.metaKey, e.shiftKey)}
                onContextMenu={(e) => handleContextMenu(e, task)}
                className={`absolute top-0 left-0 w-full flex items-center px-3 text-sm cursor-pointer
                  border-b border-border/50 border-l-[3px] transition-colors
                  ${STATE_BORDER_CLS[task.state] || "border-l-text-muted"}
                  ${isSelected
                    ? "bg-accent/10"
                    : "hover:bg-tertiary"
                  }`}
                style={{
                  height: `${virtualItem.size}px`,
                  transform: `translateY(${virtualItem.start}px)`,
                }}
              >
                {/* 状态图标 */}
                <div className="w-8 flex justify-center">
                  <StateIcon state={task.state} />
                </div>

                {/* 文件名 + 状态标签 */}
                <div className="flex-1 min-w-0 flex items-center gap-2">
                  <span className="truncate text-text-primary" title={task.name}>
                    {task.name}
                  </span>
                  <StateBadge state={task.state} error={task.error} />
                </div>

                {/* 协议标签 */}
                <div className="w-16 text-center">
                  <span
                    className={`inline-block px-1.5 py-0.5 rounded text-xs font-medium ${protocolColor(
                      task.protocol
                    )}`}
                  >
                    {protocolLabel(task.protocol)}
                  </span>
                </div>

                {/* 大小 */}
                <div className="w-20 text-right text-text-secondary font-mono text-xs">
                  {task.totalSize > 0 ? formatSize(task.totalSize) : t("taskList.unknown")}
                </div>

                {/* 进度条 */}
                <div className="w-36 px-2">
                  <div className="progress-bar">
                    <div
                      className={`progress-bar-fill state-${task.state}`}
                      style={{ width: formatProgress(task.progress) }}
                    />
                    {/* 百分比文字嵌入进度条 */}
                    <div className="absolute inset-0 flex items-center justify-center">
                      <span className="text-[10px] font-mono font-bold text-white drop-shadow-[0_1px_1px_rgba(0,0,0,0.5)]">
                        {formatProgress(task.progress)}
                      </span>
                    </div>
                  </div>
                </div>

                {/* 速度 */}
                <div className="w-24 text-right text-text-secondary font-mono text-xs">
                  {task.state === "downloading" ? (
                    <div>
                      <div>{formatSpeed(task.downloadSpeed)}</div>
                      {task.downloadLimit > 0 && (
                        <div className="text-text-muted text-[10px]">
                          {t("taskList.limit")}: {formatSpeed(task.downloadLimit)}
                        </div>
                      )}
                    </div>
                  ) : task.state === "seeding" ? (
                    <div>
                      <div>↑{formatSpeed(task.uploadSpeed)}</div>
                      {task.uploadLimit > 0 && (
                        <div className="text-text-muted text-[10px]">
                          {t("taskList.limit")}: {formatSpeed(task.uploadLimit)}
                        </div>
                      )}
                    </div>
                  ) : task.state === "done"
                    ? t("taskState.done")
                    : task.state === "error"
                    ? t("taskState.error")
                    : "--"}
                </div>

                {/* 来源（peer 数） */}
                <div className="w-16 text-center text-text-secondary text-xs">
                  {task.peers > 0 ? task.peers : "--"}
                </div>

                {/* 剩余时间 */}
                <div className="w-20 text-right text-text-secondary text-xs">
                  {(task.state === "downloading" || task.state === "seeding") && task.eta
                    ? formatEta(task.eta)
                    : task.state === "done"
                    ? t("taskState.done")
                    : "--"}
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* 右键菜单 */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          task={contextMenu.task}
          selectedIds={selectedIds}
          allTasks={allTasks}
          onPause={handlePause}
          onResume={handleResume}
          onDelete={handleDelete}
          onOpenLocation={handleOpenLocation}
          onClose={closeContextMenu}
        />
      )}
    </div>
  );
}

/** 右键菜单组件 */
function ContextMenu({
  x,
  y,
  task,
  selectedIds,
  allTasks,
  onPause,
  onResume,
  onDelete,
  onOpenLocation,
  onClose,
}: {
  x: number;
  y: number;
  task: TaskStatus;
  selectedIds: Set<string>;
  allTasks: Map<string, TaskStatus>;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onDelete: (id: string, deleteFiles: boolean) => void;
  onOpenLocation: (id: string) => void;
  onClose: () => void;
}) {
  const { t } = useI18n();
  const isPaused = task.state === "paused";
  const isDone = task.state === "done";

  // Adjust position to stay within viewport
  const menuWidth = 200;
  const menuHeight = 320;
  const adjustedX = x + menuWidth > window.innerWidth ? x - menuWidth : x;
  const adjustedY = y + menuHeight > window.innerHeight ? y - menuHeight : y;

  const handlePriority = async (priority: number) => {
    try {
      await setTaskPriority(task.id, priority);
      const fresh = await getAllTasks();
      useTaskStore.getState().setTasks(fresh);
      showToast(t("contextMenu.priorityUpdated"), "success");
    } catch (e) {
      showToast(`${t("contextMenu.priorityUpdateFailed")}: ${extractErrorMessage(e)}`, "error");
    }
    onClose();
  };

  // 打开文件
  const handleOpenFile = async () => {
    try {
      await openFile(task.id);
    } catch (e) {
      showToast(`${t("contextMenu.openFileFailed")}: ${extractErrorMessage(e)}`, "error");
    }
    onClose();
  };

  // 设置速度限制
  const handleSpeedLimit = async (downloadLimit: number, uploadLimit: number) => {
    try {
      await setTaskSpeedLimit(task.id, downloadLimit, uploadLimit);
      const fresh = await getAllTasks();
      useTaskStore.getState().setTasks(fresh);
      showToast(t("contextMenu.speedLimitUpdated"), "success");
    } catch (e) {
      showToast(`${t("contextMenu.speedLimitUpdateFailed")}: ${extractErrorMessage(e)}`, "error");
    }
    onClose();
  };

  return (
    <div
      role="menu"
      className="fixed z-50 bg-secondary border border-border rounded-md shadow-lg py-1 min-w-[180px]"
      style={{ left: Math.max(0, adjustedX), top: Math.max(0, adjustedY) }}
      onClick={(e) => e.stopPropagation()}
    >
      {/* 任务控制 */}
      {task.state === "downloading" && (
        <MenuItem icon={<Pause size={14} />} label={t("contextMenu.pause")} onClick={() => { onPause(task.id); onClose(); }} />
      )}
      {task.state === "seeding" && (
        <MenuItem icon={<Pause size={14} />} label={t("contextMenu.stopSeeding") || "Stop Seeding"} onClick={() => { onPause(task.id); onClose(); }} />
      )}
      {isPaused && (
        <MenuItem icon={<Play size={14} />} label={t("contextMenu.resume")} onClick={() => { onResume(task.id); onClose(); }} />
      )}
      {task.state === "error" && (
        <MenuItem icon={<RefreshCw size={14} />} label={t("contextMenu.retry")} onClick={() => { onResume(task.id); onClose(); }} />
      )}

      <div className="h-px bg-border my-1" />

      {/* 文件操作 */}
      {isDone && (
        <MenuItem
          icon={<File size={14} />}
          label={t("contextMenu.openFile")}
          onClick={handleOpenFile}
        />
      )}
      <MenuItem
        icon={<FolderOpen size={14} />}
        label={t("contextMenu.openFolder")}
        onClick={() => { onOpenLocation(task.id); onClose(); }}
      />
      <MenuItem
        icon={<FileText size={14} />}
        label={selectedIds.size > 1 ? `${t("contextMenu.copyLinks")} (${selectedIds.size})` : t("contextMenu.copyLink")}
        onClick={() => {
          const urls = selectedIds.size > 1
            ? Array.from(selectedIds).map(id => allTasks.get(id)?.url).filter(Boolean).join("\n")
            : task.url;
          navigator.clipboard.writeText(urls)
            .then(() => showToast(t("contextMenu.copyLinkSuccess"), "success"))
            .catch(() => showToast(t("contextMenu.copyLinkFailed"), "error"));
          onClose();
        }}
      />

      <div className="h-px bg-border my-1" />

      {/* 速度限制（非交互标题） */}
      <div className="flex items-center gap-2 px-3 py-1.5 text-xs text-text-muted">
        <Gauge size={14} />
        <span>{t("contextMenu.speedLimit")}</span>
      </div>
      <div className="pl-6 pr-2 py-1 space-y-1">
        {(() => {
          const PRESET_SPEEDS = [0, 102400, 512000, 1048576, 5242880, 10485760];
          return (
            <>
              <div className="flex items-center gap-2 text-xs text-text-secondary">
                <span className="w-12">{t("contextMenu.download")}:</span>
                <select
                  value={task.downloadLimit}
                  onChange={(e) => handleSpeedLimit(Number(e.target.value), task.uploadLimit)}
                  className="flex-1 px-2 py-1 rounded border border-border bg-primary text-text-primary text-xs"
                >
                  <option value={0}>{t("contextMenu.unlimited")}</option>
                  <option value={102400}>100 KB/s</option>
                  <option value={512000}>500 KB/s</option>
                  <option value={1048576}>1 MB/s</option>
                  <option value={5242880}>5 MB/s</option>
                  <option value={10485760}>10 MB/s</option>
                  {!PRESET_SPEEDS.includes(task.downloadLimit) && task.downloadLimit > 0 && (
                    <option value={task.downloadLimit}>{formatSpeed(task.downloadLimit)}</option>
                  )}
                </select>
              </div>
              <div className="flex items-center gap-2 text-xs text-text-secondary">
                <span className="w-12">{t("contextMenu.upload")}:</span>
                <select
                  value={task.uploadLimit}
                  onChange={(e) => handleSpeedLimit(task.downloadLimit, Number(e.target.value))}
                  className="flex-1 px-2 py-1 rounded border border-border bg-primary text-text-primary text-xs"
                >
                  <option value={0}>{t("contextMenu.unlimited")}</option>
                  <option value={102400}>100 KB/s</option>
                  <option value={512000}>500 KB/s</option>
                  <option value={1048576}>1 MB/s</option>
                  <option value={5242880}>5 MB/s</option>
                  <option value={10485760}>10 MB/s</option>
                  {!PRESET_SPEEDS.includes(task.uploadLimit) && task.uploadLimit > 0 && (
                    <option value={task.uploadLimit}>{formatSpeed(task.uploadLimit)}</option>
                  )}
                </select>
              </div>
            </>
          );
        })()}
      </div>

      <div className="h-px bg-border my-1" />

      {/* 优先级（不同图标区分级别） */}
      <MenuItem
        icon={<ArrowUp size={14} className={task.priority === 3 ? "text-accent" : ""} />}
        label={`${t("contextMenu.priorityHigh")}${task.priority === 3 ? " ✓" : ""}`}
        onClick={() => handlePriority(3)}
      />
      <MenuItem
        icon={<Minus size={14} className={task.priority === 1 ? "text-text-secondary" : ""} />}
        label={`${t("contextMenu.priorityNormal")}${task.priority === 1 ? " ✓" : ""}`}
        onClick={() => handlePriority(1)}
      />
      <MenuItem
        icon={<ArrowDown size={14} className={task.priority === 0 ? "text-text-muted" : ""} />}
        label={`${t("contextMenu.priorityLow")}${task.priority === 0 ? " ✓" : ""}`}
        onClick={() => handlePriority(0)}
      />

      <div className="h-px bg-border my-1" />

      {/* 删除 */}
      <MenuItem
        icon={<Trash2 size={14} />}
        label={t("contextMenu.delete")}
        onClick={() => { onDelete(task.id, false); onClose(); }}
      />
      <MenuItem
        icon={<Trash2 size={14} />}
        label={t("contextMenu.deleteFiles")}
        onClick={async () => {
          const { confirm } = await import("./ConfirmDialog");
          const ok = await confirm({
            title: t("dialog.deleteTask.title"),
            message: t("dialog.deleteTask.message"),
            variant: "danger",
          });
          if (ok) onDelete(task.id, true);
          onClose();
        }}
        className="text-error"
      />
    </div>
  );
}

/** 菜单项（可复用） */
function MenuItem({
  icon,
  label,
  onClick,
  className = "",
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  className?: string;
}) {
  return (
    <button
      role="menuitem"
      onClick={onClick}
      className={`flex items-center gap-2 w-full px-3 py-1.5 text-sm text-text-primary hover:bg-tertiary transition-colors ${className}`}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}
