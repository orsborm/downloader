// 顶部工具栏组件
// 包含：添加链接、暂停全部、开始全部、搜索过滤、RSS、插件、解压、速度图表、设置、调度

import { useCallback, useRef, useEffect, useMemo } from "react";
import {
  Plus,
  Pause,
  Play,
  Settings,
  BarChart3,
  Download,
  Trash2,
  Rss,
  Package,
  Archive,
  Clock,
  Search,
  X,
  Filter,
  Layers,
  Timer,
} from "lucide-react";
import { useTaskStore, type StatusFilter } from "../stores/taskStore";
import { pauseAllTasks, resumeAllTasks, removeCompletedTasks } from "../lib/tauri-api";
import { showToast } from "./Toast";
import { extractErrorMessage } from "../lib/errors";
import { useI18n } from "../hooks/useI18n";

interface ToolbarProps {
  onAddTask: () => void;
  onOpenBatchImport: () => void;
  onOpenSettings: () => void;
  onToggleSpeedChart: () => void;
  onOpenRss: () => void;
  onOpenPlugins: () => void;
  onOpenArchive: () => void;
  onOpenHistory: () => void;
  onOpenSchedule: () => void;
}

export function Toolbar({
  onAddTask,
  onOpenBatchImport,
  onOpenSettings,
  onToggleSpeedChart,
  onOpenRss,
  onOpenPlugins,
  onOpenArchive,
  onOpenHistory,
  onOpenSchedule,
}: ToolbarProps) {
  const { t } = useI18n();
  const tasks = useTaskStore((s) => s.tasks);
  const selectedIds = useTaskStore((s) => s.selectedIds);
  const setTasks = useTaskStore((s) => s.setTasks);
  const searchQuery = useTaskStore((s) => s.searchQuery);
  const setSearchQuery = useTaskStore((s) => s.setSearchQuery);
  const statusFilter = useTaskStore((s) => s.statusFilter);
  const setStatusFilter = useTaskStore((s) => s.setStatusFilter);
  const searchInputRef = useRef<HTMLInputElement>(null);

  // 缓存任务数组，避免每次渲染都重新创建
  const taskArray = useMemo(() => Array.from(tasks.values()), [tasks]);
  const hasActive = useMemo(
    () => taskArray.some((t) => t.state === "downloading" || t.state === "seeding"),
    [taskArray]
  );

  // Ctrl+F 聚焦搜索框
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "f") {
        e.preventDefault();
        searchInputRef.current?.focus();
        searchInputRef.current?.select();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // 计算过滤后的任务数量
  const isFiltering = searchQuery.trim() !== "" || statusFilter !== "all";
  const filteredCount = useMemo(() => {
    if (!isFiltering) return tasks.size;
    return taskArray.filter((t) => {
      if (statusFilter !== "all" && t.state !== statusFilter) return false;
      if (searchQuery.trim()) {
        const q = searchQuery.trim().toLowerCase();
        if (!t.name.toLowerCase().includes(q) && !t.url.toLowerCase().includes(q)) return false;
      }
      return true;
    }).length;
  }, [isFiltering, taskArray, statusFilter, searchQuery, tasks.size]);

  const reloadTasks = useCallback(async () => {
    const { getAllTasks } = await import("../lib/tauri-api");
    const fresh = await getAllTasks();
    setTasks(fresh);
  }, [setTasks]);

  const handlePauseAll = useCallback(async () => {
    try {
      await pauseAllTasks();
      await reloadTasks();
      showToast(t("toolbar.pauseAll"), "info");
    } catch (e) {
      showToast(`${t("toolbar.pauseAll")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [reloadTasks]);

  const handleResumeAll = useCallback(async () => {
    try {
      await resumeAllTasks();
      await reloadTasks();
      showToast(t("toolbar.startAll"), "info");
    } catch (e) {
      showToast(`${t("toolbar.startAll")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [reloadTasks]);

  const handleCleanCompleted = useCallback(async () => {
    const { confirm } = await import("./ConfirmDialog");
    const ok = await confirm({
      title: t("dialog.clearCompleted.title"),
      message: t("dialog.clearCompleted.message"),
    });
    if (!ok) return;
    try {
      await removeCompletedTasks();
      await reloadTasks();
      showToast(t("common.success"), "success");
    } catch (e) {
      showToast(`${t("common.error")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [reloadTasks, t]);

  return (
    <div className="flex items-center gap-0.5 px-2 py-1.5 border-b border-border bg-secondary">
      {/* 左侧操作区 */}
      <div className="flex items-center gap-0.5 shrink-0">
        {/* 添加链接 */}
        <button
          onClick={onAddTask}
          className="inline-flex items-center gap-1 px-2.5 py-1.5 rounded-md text-xs leading-none
                     bg-accent text-white hover:bg-accent-hover transition-colors"
        >
          <Plus size={14} className="shrink-0" />
          <span className="whitespace-nowrap hidden sm:inline">{t("toolbar.addTask")}</span>
        </button>

        {/* 批量导入 */}
        <button
          onClick={onOpenBatchImport}
          title={t("batchImport.title")}
          className="inline-flex items-center p-1.5 rounded-md text-xs leading-none
                     text-text-secondary hover:text-text-primary hover:bg-tertiary transition-colors"
        >
          <Layers size={14} className="shrink-0" />
        </button>

        {/* 分隔线 */}
        <div className="w-px h-4 bg-border mx-0.5" />

        {/* 暂停全部 / 开始全部 */}
        {hasActive ? (
          <ToolbarButton icon={<Pause size={14} />} label={t("toolbar.pauseAll")} onClick={handlePauseAll} />
        ) : (
          <ToolbarButton icon={<Play size={14} />} label={t("toolbar.startAll")} onClick={handleResumeAll} />
        )}

        {/* 清理已完成 */}
        <ToolbarButton icon={<Trash2 size={14} />} label={t("dialog.clearCompleted.title")} onClick={handleCleanCompleted} />
      </div>

      {/* 分隔线 */}
      <div className="w-px h-4 bg-border mx-0.5 shrink-0" />

      {/* 搜索 + 过滤 */}
      <div className="flex items-center gap-0.5 shrink-0">
        <div className="relative flex items-center">
          <Search size={12} className="absolute left-1.5 text-text-muted pointer-events-none" />
          <input
            ref={searchInputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={`${t("toolbar.searchPlaceholder")} (Ctrl+F)`}
            className="pl-6 pr-6 py-1 text-xs bg-tertiary border border-border rounded-md
                       text-text-primary placeholder:text-text-muted
                       focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/30 w-28 sm:w-36"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="absolute right-1.5 p-0.5 text-text-muted hover:text-text-primary rounded-sm transition-colors"
            >
              <X size={12} />
            </button>
          )}
        </div>

        <div className="relative flex items-center">
          <Filter size={12} className="absolute left-1.5 text-text-muted pointer-events-none" />
          <select
            aria-label={t("toolbar.filterAll")}
            value={statusFilter}
            onChange={(e) => setStatusFilter(e.target.value as StatusFilter)}
            className="pl-6 pr-2 py-1 text-xs bg-tertiary border border-border rounded-md
                       text-text-primary appearance-none cursor-pointer
                       hover:border-text-muted transition-colors
                       focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/30"
          >
            <option value="all">{t("common.all")}</option>
            <option value="downloading">{t("taskState.downloading")}</option>
            <option value="paused">{t("taskState.paused")}</option>
            <option value="done">{t("taskState.done")}</option>
            <option value="error">{t("taskState.error")}</option>
            <option value="seeding">{t("taskState.seeding")}</option>
            <option value="queued">{t("taskState.queued")}</option>
          </select>
        </div>
      </div>

      {/* 分隔线 */}
      <div className="w-px h-4 bg-border mx-0.5 shrink-0" />

      {/* 功能按钮组 */}
      <div className="flex items-center gap-0.5 shrink-0">
        <ToolbarButton icon={<Rss size={16} />} label="RSS" onClick={onOpenRss} />
        <ToolbarButton icon={<Package size={16} />} label={t("plugin.title")} onClick={onOpenPlugins} />
        <ToolbarButton icon={<Archive size={16} />} label={t("archive.title")} onClick={onOpenArchive} />
        <ToolbarButton icon={<Clock size={16} />} label={t("downloadHistory.title")} onClick={onOpenHistory} />
        <ToolbarButton icon={<Timer size={16} />} label={t("toolbar.schedule")} onClick={onOpenSchedule} />
        <ToolbarButton icon={<BarChart3 size={16} />} label={t("statusBar.downloadSpeed")} onClick={onToggleSpeedChart} />
      </div>

      {/* 弹性占位 */}
      <div className="flex-1 min-w-0" />

      {/* 右侧固定区 */}
      <div className="flex items-center gap-1 shrink-0">
        {/* 任务计数 */}
        {isFiltering && (
          <span className="text-[10px] text-text-muted whitespace-nowrap">
            {t("taskList.filteredCount", { filtered: filteredCount, total: tasks.size })}
          </span>
        )}

        {/* 选中计数 */}
        {selectedIds.size > 1 && (
          <span className="text-[10px] text-accent whitespace-nowrap">
            {t("taskList.selectedCount", { count: selectedIds.size })}
          </span>
        )}

        {/* 应用标题（小屏隐藏） */}
        <div className="hidden xl:inline-flex items-center gap-1 text-text-secondary text-xs leading-none">
          <Download size={14} className="shrink-0" />
          <span className="font-medium whitespace-nowrap">Downloader</span>
        </div>

        {/* 分隔线 */}
        <div className="w-px h-4 bg-border" />

        {/* 设置（始终可见） */}
        <ToolbarButton icon={<Settings size={16} />} label={t("settings.title")} onClick={onOpenSettings} />
      </div>
    </div>
  );
}

/** 工具栏按钮（可复用） */
function ToolbarButton({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      title={label}
      className="inline-flex items-center gap-1 px-1.5 py-1 rounded-md text-xs leading-none
                 text-text-secondary hover:text-text-primary hover:bg-tertiary
                 transition-colors"
    >
      <span className="inline-flex items-center justify-center w-3.5 h-3.5 shrink-0">{icon}</span>
      <span className="hidden lg:inline whitespace-nowrap">{label}</span>
    </button>
  );
}
