// 应用主入口组件
// 布局：顶部工具栏 + 中部任务列表 + 底部状态栏
// 负责初始化事件监听、主题切换、剪贴板监听

import { useEffect, useState, useCallback, Component, lazy, Suspense, type ReactNode, type ErrorInfo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTaskEvents } from "./hooks/useTaskEvents";
import { useClipboard } from "./hooks/useClipboard";
import { useTaskStore } from "./stores/taskStore";
import { getAllTasks, pauseTask, resumeTask, removeTask, addTask, getSettings } from "./lib/tauri-api";
import { TaskList } from "./components/TaskList";
import { AddTaskDialog } from "./components/AddTaskDialog";
import { TaskDetail } from "./components/TaskDetail";
import { SpeedChart } from "./components/SpeedChart";
import { StatusBar } from "./components/StatusBar";
import { Toolbar } from "./components/Toolbar";
import { SettingsDialog } from "./components/SettingsDialog";
import { PluginManager } from "./components/PluginManager";
import { RssManager } from "./components/RssManager";
import { ArchiveDialog } from "./components/ArchiveDialog";
const DownloadHistoryDialog = lazy(() => import("./components/DownloadHistoryDialog").then(m => ({ default: m.DownloadHistoryDialog })));
import { BatchImportDialog } from "./components/BatchImportDialog";
import { ScheduleDialog } from "./components/ScheduleDialog";
import { ToastContainer, showToast } from "./components/Toast";
import { ConfirmDialogContainer } from "./components/ConfirmDialog";
import { handleError, withErrorHandling, ErrorCode } from "./lib/errors";
import { useI18n } from "./hooks/useI18n";

/** Error Boundary：捕获子组件渲染错误，防止白屏 */
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary caught:", error, errorInfo);
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null });
  };

  render() {
    if (this.state.hasError) {
      return (
        <div className="h-screen flex flex-col items-center justify-center bg-primary text-text-primary p-8">
          <h2 className="text-xl font-semibold mb-2">应用发生错误 / Application Error</h2>
          <p className="text-text-secondary mb-4 text-sm max-w-md text-center">
            {this.state.error?.message || "未知错误 / Unknown error"}
          </p>
          <button
            onClick={this.handleReset}
            className="px-4 py-2 bg-accent text-white rounded-md hover:bg-accent-hover transition-colors"
          >
            重试 / Retry
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

export default function App() {
  const { t } = useI18n();

  // 订阅后端任务状态更新事件
  useTaskEvents();

  // 任务列表状态
  const setTasks = useTaskStore((s) => s.setTasks);
  const selectedIds = useTaskStore((s) => s.selectedIds);
  const tasks = useTaskStore((s) => s.tasks);

  // 对话框状态
  const [showAddDialog, setShowAddDialog] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showSpeedChart, setShowSpeedChart] = useState(false);
  const [showPluginManager, setShowPluginManager] = useState(false);
  const [showRssManager, setShowRssManager] = useState(false);
  const [showArchiveDialog, setShowArchiveDialog] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [showBatchImport, setShowBatchImport] = useState(false);
  const [showSchedule, setShowSchedule] = useState(false);
  const [detectedUrl, setDetectedUrl] = useState<string | null>(null);

  // 初始化加载任务列表
  useEffect(() => {
    const loadTasks = async () => {
      const tasks = await withErrorHandling(
        () => getAllTasks(),
        t("app.loadingTasks"),
        ErrorCode.CONFIG_LOAD_FAILED
      );
      if (tasks) {
        setTasks(tasks);
      }
    };
    loadTasks();
  }, [setTasks]);

  // 初始化主题 + 监听系统主题变化
  useEffect(() => {
    const savedTheme = localStorage.getItem("theme") || "system";
    applyTheme(savedTheme);

    // 监听系统主题变化（仅在 "system" 模式下生效）
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleSystemThemeChange = () => {
      const currentTheme = localStorage.getItem("theme") || "system";
      if (currentTheme === "system") {
        applyTheme("system");
      }
    };
    mediaQuery.addEventListener("change", handleSystemThemeChange);
    return () => mediaQuery.removeEventListener("change", handleSystemThemeChange);
  }, []);

  // 初始化字体大小（从后端配置读取）
  useEffect(() => {
    getSettings()
      .then((cfg) => {
        if (cfg.general.fontSize && cfg.general.fontSize !== 14) {
          document.documentElement.style.fontSize = `${cfg.general.fontSize}px`;
        }
      })
      .catch(() => {});
  }, []);

  // 应用主题
  const applyTheme = (theme: string) => {
    const root = document.documentElement;
    if (theme === "system") {
      const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.setAttribute("data-theme", isDark ? "dark" : "light");
    } else {
      root.setAttribute("data-theme", theme);
    }
    localStorage.setItem("theme", theme);
  };

  // 剪贴板检测到下载链接
  const handleClipboardDetect = useCallback((url: string) => {
    setDetectedUrl(url);
    setShowAddDialog(true);
  }, []);

  // 拖拽 .torrent 文件支持（使用 Tauri 原生 API）
  const [isDragging, setIsDragging] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupDragDrop = async () => {
      try {
        unlisten = await getCurrentWindow().onDragDropEvent(async (event) => {
          if (event.payload.type === "enter" || event.payload.type === "over") {
            setIsDragging(true);
          } else if (event.payload.type === "leave") {
            setIsDragging(false);
          } else if (event.payload.type === "drop") {
            setIsDragging(false);
            const paths = event.payload.paths;
            const torrentPaths = paths.filter((p: string) =>
              p.toLowerCase().endsWith(".torrent")
            );

            for (const path of torrentPaths) {
              const result = await withErrorHandling(
                () => addTask({ url: path, savePath: "", startImmediately: true }),
                t("app.addTorrent"),
                ErrorCode.DOWNLOAD_FAILED
              );
              if (result !== undefined) {
                showToast(t("app.torrentAdded"), "success");
              }
            }

            if (torrentPaths.length > 0) {
              const fresh = await withErrorHandling(
                () => getAllTasks(),
                t("app.refreshTasks"),
                ErrorCode.CONFIG_LOAD_FAILED
              );
              if (fresh) {
                setTasks(fresh);
              }
            } else {
              // 拖拽的文件中没有 .torrent 文件
              showToast(t("app.unsupportedDrop") || "仅支持 .torrent 文件拖拽", "info");
            }
          }
        });
      } catch (e) {
        handleError(e, t("app.registerDrag"), ErrorCode.UNKNOWN);
      }
    };

    setupDragDrop();

    return () => {
      if (unlisten) unlisten();
    };
  }, [setTasks]);

  useClipboard(handleClipboardDetect);

  // 全局键盘快捷键
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // 忽略输入框内的按键
      const tag = (e.target as HTMLElement).tagName;
      if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return;

      const store = useTaskStore.getState();
      const selectedIds = store.selectedIds;

      if ((e.key === "n" || e.key === "N") && (e.ctrlKey || e.metaKey)) {
        // Ctrl+N 添加任务
        e.preventDefault();
        setShowAddDialog(true);
      } else if (e.key === "Delete") {
        // Delete 键删除选中任务（需确认）
        if (selectedIds.size > 0) {
          e.preventDefault();
          const idsToDelete = Array.from(selectedIds);
          (async () => {
            const { confirm } = await import("./components/ConfirmDialog");
            const ok = await confirm({
              title: t("dialog.deleteTask.title"),
              message: idsToDelete.length > 1
                ? t("taskList.selectedCount", { count: String(idsToDelete.length) })
                : t("dialog.deleteTask.message"),
              variant: "danger",
            });
            if (!ok) return;
            for (const id of idsToDelete) {
              try {
                await removeTask(id, false);
                useTaskStore.getState().removeTask(id);
              } catch (err) {
                handleError(err, t("app.deleteTask"), ErrorCode.DOWNLOAD_FAILED);
              }
            }
          })();
        }
      } else if (e.key === " ") {
        // 空格键暂停/恢复选中任务
        if (selectedIds.size === 1) {
          e.preventDefault();
          const id = Array.from(selectedIds)[0];
          const task = store.tasks.get(id);
          if (task) {
            if (task.state === "downloading" || task.state === "seeding") {
              pauseTask(id).catch((err) => {
                handleError(err, t("app.pauseTask"), ErrorCode.DOWNLOAD_FAILED);
              });
            } else if (task.state === "paused") {
              resumeTask(id).catch((err) => {
                handleError(err, t("app.resumeTask"), ErrorCode.DOWNLOAD_FAILED);
              });
            }
          }
        }
      } else if (e.key === "a" && (e.ctrlKey || e.metaKey)) {
        // Ctrl+A 全选
        e.preventDefault();
        const allIds = new Set(store.tasks.keys());
        useTaskStore.setState({ selectedIds: allIds });
      } else if (e.key === "Escape") {
        // Escape 清除选择
        store.clearSelection();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // 获取选中的任务
  const selectedTaskId = selectedIds.size === 1 ? Array.from(selectedIds)[0] : null;
  const selectedTask = selectedTaskId ? tasks.get(selectedTaskId) : null;

  return (
    <ErrorBoundary>
    <div className="h-screen flex flex-col bg-primary relative">
      {/* 拖拽遮罩 */}
      {isDragging && (
        <div className="absolute inset-0 z-40 bg-accent/10 border-2 border-dashed border-accent rounded-lg flex items-center justify-center pointer-events-none">
          <div className="bg-primary rounded-lg shadow-xl px-8 py-4 text-center">
            <p className="text-lg font-medium text-text-primary">{t("app.dragToAdd") || "释放以添加种子文件"}</p>
            <p className="text-sm text-text-muted mt-1">{t("app.torrentSupported") || "支持 .torrent 文件"}</p>
          </div>
        </div>
      )}
      {/* 顶部工具栏 */}
      <Toolbar
        onAddTask={() => setShowAddDialog(true)}
        onOpenBatchImport={() => setShowBatchImport(true)}
        onOpenSettings={() => setShowSettings(true)}
        onToggleSpeedChart={() => setShowSpeedChart(!showSpeedChart)}
        onOpenRss={() => setShowRssManager(true)}
        onOpenPlugins={() => setShowPluginManager(true)}
        onOpenArchive={() => setShowArchiveDialog(true)}
        onOpenHistory={() => setShowHistory(true)}
        onOpenSchedule={() => setShowSchedule(true)}
      />

      {/* 主内容区域 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 任务列表 */}
        <div className="flex-1 overflow-hidden">
          <TaskList />
        </div>

        {/* 速度图表（可展开/收起） */}
        {showSpeedChart && (
          <div className="h-48 border-t border-border">
            <SpeedChart />
          </div>
        )}

        {/* 任务详情面板（选中任务时展开） */}
        {selectedTask && (
          <div className="h-64 border-t border-border overflow-hidden">
            <TaskDetail task={selectedTask} />
          </div>
        )}
      </div>

      {/* 底部状态栏 */}
      <StatusBar />

      {/* 添加任务对话框 */}
      {showAddDialog && (
        <AddTaskDialog
          initialUrl={detectedUrl || undefined}
          onClose={() => {
            setShowAddDialog(false);
            setDetectedUrl(null);
          }}
        />
      )}

      {/* 设置对话框 */}
      {showSettings && (
        <SettingsDialog onClose={() => setShowSettings(false)} />
      )}

      {/* 插件管理 */}
      {showPluginManager && (
        <PluginManager onClose={() => setShowPluginManager(false)} />
      )}

      {/* RSS 订阅管理 */}
      {showRssManager && (
        <RssManager onClose={() => setShowRssManager(false)} />
      )}

      {/* 自动解压设置 */}
      {showArchiveDialog && (
        <ArchiveDialog onClose={() => setShowArchiveDialog(false)} />
      )}

      {/* 下载历史（懒加载，含 recharts） */}
      {showHistory && (
        <Suspense fallback={<div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"><div className="bg-primary rounded-lg p-8 text-text-secondary">{t("common.loading")}</div></div>}>
          <DownloadHistoryDialog onClose={() => setShowHistory(false)} />
        </Suspense>
      )}

      {/* 批量导入 */}
      {showBatchImport && (
        <BatchImportDialog onClose={() => setShowBatchImport(false)} />
      )}

      {/* 任务调度 */}
      {showSchedule && (
        <ScheduleDialog
          open={showSchedule}
          onClose={() => setShowSchedule(false)}
        />
      )}

      {/* Toast 通知 */}
      <ToastContainer />
      {/* 确认对话框 */}
      <ConfirmDialogContainer />
    </div>
    </ErrorBoundary>
  );
}
