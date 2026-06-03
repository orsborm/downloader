// 添加任务对话框组件
// 支持：输入链接（自动识别协议）、选择保存目录、文件名、高级选项
// 支持多链接识别、BT/Magnet 文件选择

import { useState, useCallback, useMemo } from "react";
import { X, Link, FolderOpen, Loader2 } from "lucide-react";
import {
  addTask,
  getAllTasks,
  getTorrentFiles,
  addBtTaskWithFiles,
  batchAddTasks,
  type TorrentFileListResponse,
} from "../lib/tauri-api";
import { isValidDownloadUrl } from "../lib/format";
import { useTaskStore } from "../stores/taskStore";
import { showToast } from "./Toast";
import { extractErrorMessage } from "../lib/errors";
import { useI18n } from "../hooks/useI18n";
import { FileSelectDialog } from "./FileSelectDialog";

interface AddTaskDialogProps {
  initialUrl?: string;
  onClose: () => void;
}

/** 检测是否为 BT/Magnet 链接 */
function isBtOrMagnetLink(url: string): boolean {
  const trimmed = url.trim().toLowerCase();
  return (
    trimmed.startsWith("magnet:") ||
    trimmed.endsWith(".torrent") ||
    trimmed.startsWith("bt://")
  );
}

/** 解析多链接（按行分割，过滤空行） */
function parseMultipleUrls(text: string): string[] {
  return text
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0);
}

export function AddTaskDialog({ initialUrl, onClose }: AddTaskDialogProps) {
  const { t } = useI18n();
  const [url, setUrl] = useState(initialUrl || "");
  const [savePath, setSavePath] = useState("");
  const [fileName, setFileName] = useState("");
  const [proxy, setProxy] = useState("");
  const [speedLimit, setSpeedLimit] = useState("");
  const [startImmediately, setStartImmediately] = useState(true);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 文件选择相关状态
  const [showFileSelect, setShowFileSelect] = useState(false);
  const [fileList, setFileList] = useState<TorrentFileListResponse | null>(null);
  const [pendingBtUrl, setPendingBtUrl] = useState<string | null>(null);

  // 使用 useMemo 缓存解析结果，避免每次渲染重新计算
  const { urls, isMultipleLinks, isSingleBtLink } = useMemo(() => {
    const parsed = parseMultipleUrls(url);
    return {
      urls: parsed,
      isMultipleLinks: parsed.length > 1,
      isSingleBtLink: parsed.length === 1 && isBtOrMagnetLink(parsed[0]),
    };
  }, [url]);

  // 获取种子文件列表
  const handleFetchFileList = useCallback(async (source: string) => {
    setLoading(true);
    setError(null);

    try {
      const response = await getTorrentFiles(source);
      setFileList(response);
      setPendingBtUrl(source);
      setShowFileSelect(true);
    } catch (e) {
      setError(extractErrorMessage(e));
    } finally {
      setLoading(false);
    }
  }, []);

  // 处理文件选择确认
  const handleFileSelectConfirm = useCallback(
    async (selectedIndices: number[]) => {
      if (!pendingBtUrl) return;

      setLoading(true);
      setShowFileSelect(false);

      try {
        await addBtTaskWithFiles(
          {
            url: pendingBtUrl,
            savePath: savePath.trim(),
            fileName: fileName.trim() || undefined,
            proxy: proxy.trim() || undefined,
            speedLimit: speedLimit ? (parseInt(speedLimit) || 0) * 1024 : undefined,
            startImmediately,
          },
          selectedIndices,
          fileList?.files
        );

        // 重新加载任务列表
        const tasks = await getAllTasks();
        useTaskStore.getState().setTasks(tasks);

        showToast(t("addTask.addSuccess"), "success");
        onClose();
      } catch (e) {
        setError(extractErrorMessage(e));
      } finally {
        setLoading(false);
        setPendingBtUrl(null);
      }
    },
    [pendingBtUrl, savePath, fileName, proxy, speedLimit, startImmediately, onClose]
  );

  // 处理文件选择取消
  const handleFileSelectCancel = useCallback(() => {
    setShowFileSelect(false);
    setFileList(null);
    setPendingBtUrl(null);
  }, []);

  const handleSubmit = useCallback(async () => {
    if (loading) return; // 双重提交防护
    if (urls.length === 0) {
      setError(t("addTask.enterUrl"));
      return;
    }

    // 验证所有链接
    const invalidUrls = urls.filter((u) => !isValidDownloadUrl(u));
    if (invalidUrls.length > 0) {
      setError(t("addTask.unsupportedUrl"));
      return;
    }

    // 如果是单个 BT/Magnet 链接，获取文件列表让用户选择
    if (isSingleBtLink) {
      await handleFetchFileList(urls[0]);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // 多链接批量添加
      if (isMultipleLinks) {
        const taskIds = await batchAddTasks(
          urls,
          savePath.trim() || undefined,
          startImmediately
        );
        showToast(
          t("addTask.batchAddSuccess", { count: String(taskIds.length) }),
          "success"
        );
      } else {
        // 单链接添加
        await addTask({
          url: urls[0],
          savePath: savePath.trim(),
          fileName: fileName.trim() || undefined,
          proxy: proxy.trim() || undefined,
          speedLimit: speedLimit ? (parseInt(speedLimit) || 0) * 1024 : undefined,
          startImmediately,
        });
        showToast(t("addTask.addSuccess"), "success");
      }

      // 重新加载任务列表
      const tasks = await getAllTasks();
      useTaskStore.getState().setTasks(tasks);

      onClose();
    } catch (e) {
      setError(extractErrorMessage(e));
    } finally {
      setLoading(false);
    }
  }, [loading, urls, isMultipleLinks, isSingleBtLink, handleFetchFileList, savePath, fileName, proxy, speedLimit, startImmediately, onClose, t]);

  // 快捷键：Ctrl+Enter 提交
  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        handleSubmit();
      } else if (e.key === "Escape") {
        onClose();
      }
    },
    [handleSubmit, onClose]
  );

  return (
    <>
      <div
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
        onClick={onClose}
      >
        <div
          className="bg-primary rounded-lg shadow-xl w-full max-w-lg mx-4 animate-fade-in"
          onClick={(e) => e.stopPropagation()}
          onKeyDown={handleKeyDown}
        >
          {/* 标题栏 */}
          <div className="flex items-center justify-between px-5 py-3 border-b border-border">
            <h2 className="text-lg font-semibold text-text-primary">{t("addTask.title")}</h2>
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted"
            >
              <X size={18} />
            </button>
          </div>

          {/* 表单内容 */}
          <div className="px-5 py-4 space-y-4">
            {/* 链接输入 */}
            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">
                {t("addTask.urlLabel")}
              </label>
              <div className="relative">
                <Link
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
                />
                <textarea
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder={t("addTask.urlPlaceholderMulti")}
                  rows={isMultipleLinks ? 4 : 1}
                  className="w-full pl-9 pr-3 py-2 rounded-md border border-border bg-secondary
                             text-text-primary text-sm font-mono resize-y
                             focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent
                             placeholder:text-text-muted"
                  autoFocus
                />
              </div>
              <div className="flex items-center justify-between mt-1">
                <p className="text-xs text-text-muted">
                  {t("addTask.supportedProtocols")}
                </p>
                {isMultipleLinks && (
                  <p className="text-xs text-accent">
                    {t("addTask.multiLinkCount", { count: String(urls.length) })}
                  </p>
                )}
                {isSingleBtLink && (
                  <p className="text-xs text-accent">
                    {t("addTask.btFileSelectHint")}
                  </p>
                )}
              </div>
            </div>

            {/* 保存目录 */}
            <div>
              <label className="block text-sm font-medium text-text-secondary mb-1">
                {t("addTask.savePath")}
              </label>
              <div className="relative">
                <FolderOpen
                  size={16}
                  className="absolute left-3 top-1/2 -translate-y-1/2 text-text-muted"
                />
                <input
                  type="text"
                  value={savePath}
                  onChange={(e) => setSavePath(e.target.value)}
                  placeholder={t("addTask.defaultDirPlaceholder")}
                  className="w-full pl-9 pr-3 py-2 rounded-md border border-border bg-secondary
                             text-text-primary text-sm
                             focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent
                             placeholder:text-text-muted"
                />
              </div>
            </div>

            {/* 文件名（单链接时显示） */}
            {!isMultipleLinks && (
              <div>
                <label className="block text-sm font-medium text-text-secondary mb-1">
                  {t("addTask.fileNameOptional")}
                </label>
                <input
                  type="text"
                  value={fileName}
                  onChange={(e) => setFileName(e.target.value)}
                  placeholder={t("addTask.autoExtract")}
                  className="w-full px-3 py-2 rounded-md border border-border bg-secondary
                             text-text-primary text-sm
                             focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-accent
                             placeholder:text-text-muted"
                />
              </div>
            )}

            {/* 高级选项 */}
            <div>
              <button
                onClick={() => setShowAdvanced(!showAdvanced)}
                className="text-sm text-accent hover:underline"
              >
                {showAdvanced ? t("addTask.collapseAdvanced") : t("addTask.advanced")}
              </button>

              {showAdvanced && (
                <div className="mt-2 space-y-3 p-3 bg-tertiary rounded-md">
                  {/* 代理 */}
                  <div>
                    <label className="block text-xs text-text-secondary mb-1">
                      {t("addTask.proxyHint")}
                    </label>
                    <input
                      type="text"
                      value={proxy}
                      onChange={(e) => setProxy(e.target.value)}
                      placeholder="socks5://127.0.0.1:1080"
                      className="w-full px-3 py-1.5 rounded border border-border bg-primary
                                 text-text-primary text-sm
                                 focus:outline-none focus:ring-1 focus:ring-accent/50"
                    />
                  </div>

                  {/* 限速 */}
                  <div>
                    <label className="block text-xs text-text-secondary mb-1">
                      {t("addTask.speedLimitHint")}
                    </label>
                    <input
                      type="number"
                      value={speedLimit}
                      onChange={(e) => setSpeedLimit(Math.max(0, parseInt(e.target.value) || 0).toString())}
                      min={0}
                      placeholder="0"
                      className="w-full px-3 py-1.5 rounded border border-border bg-primary
                                 text-text-primary text-sm
                                 focus:outline-none focus:ring-1 focus:ring-accent/50"
                    />
                  </div>
                </div>
              )}
            </div>

            {/* 添加后立即开始 */}
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={startImmediately}
                onChange={(e) => setStartImmediately(e.target.checked)}
                className="w-4 h-4 rounded border-border text-accent
                           focus:ring-accent/50"
              />
              <span className="text-sm text-text-primary">{t("addTask.startImmediatelyDownload")}</span>
            </label>

            {/* 错误提示 */}
            {error && (
              <div className="text-sm text-error bg-error/10 px-3 py-2 rounded">
                {error}
              </div>
            )}
          </div>

          {/* 底部按钮 */}
          <div className="flex items-center justify-between px-5 py-3 border-t border-border">
            <span className="text-xs text-text-muted">
              {t("addTask.shortcutHint")}
            </span>
            <div className="flex gap-2">
              <button
                onClick={onClose}
                className="px-4 py-1.5 rounded-md text-sm text-text-secondary
                           hover:bg-tertiary transition-colors"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleSubmit}
                disabled={loading || urls.length === 0}
                className="px-4 py-1.5 rounded-md text-sm text-white
                           bg-accent hover:bg-accent-hover disabled:opacity-50
                           transition-colors flex items-center gap-1.5"
              >
                {loading && <Loader2 size={14} className="animate-spin" />}
                {loading
                  ? t("addTask.adding")
                  : isSingleBtLink
                  ? t("addTask.selectFiles")
                  : isMultipleLinks
                  ? t("addTask.batchAdd", { count: String(urls.length) })
                  : t("common.confirm")}
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* 文件选择对话框 */}
      {showFileSelect && fileList && (
        <FileSelectDialog
          fileList={fileList}
          onConfirm={handleFileSelectConfirm}
          onCancel={handleFileSelectCancel}
        />
      )}
    </>
  );
}
