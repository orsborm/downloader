// 文件选择对话框组件
// 用于 BT/Magnet 任务的文件选择，支持全选/反选/单选

import { useState, useCallback, useEffect } from "react";
import { X, File, Folder, Check, Minus, Loader2 } from "lucide-react";
import { formatSize } from "../lib/format";
import { useI18n } from "../hooks/useI18n";
import type { TorrentFileInfo, TorrentFileListResponse } from "../lib/tauri-api";

interface FileSelectDialogProps {
  /** 种子文件列表响应 */
  fileList: TorrentFileListResponse;
  /** 确认选择回调 */
  onConfirm: (selectedIndices: number[]) => void;
  /** 取消回调 */
  onCancel: () => void;
}

export function FileSelectDialog({ fileList, onConfirm, onCancel }: FileSelectDialogProps) {
  const { t } = useI18n();
  const [files, setFiles] = useState<TorrentFileInfo[]>(fileList.files);
  const [loading, setLoading] = useState(false);

  // 计算选中状态
  const selectedCount = files.filter((f) => f.selected).length;
  const allSelected = files.length > 0 && selectedCount === files.length;
  const someSelected = selectedCount > 0 && selectedCount < files.length;
  const selectedSize = files
    .filter((f) => f.selected)
    .reduce((sum, f) => sum + f.size, 0);

  // 全选/取消全选
  const handleToggleAll = useCallback(() => {
    setFiles((prev) =>
      prev.map((f) => ({ ...f, selected: !allSelected }))
    );
  }, [allSelected]);

  // 切换单个文件
  const handleToggleFile = useCallback((index: number) => {
    setFiles((prev) =>
      prev.map((f) => (f.index === index ? { ...f, selected: !f.selected } : f))
    );
  }, []);

  // 反选
  const handleInvertSelection = useCallback(() => {
    setFiles((prev) => prev.map((f) => ({ ...f, selected: !f.selected })));
  }, []);

  // 确认选择
  const handleConfirm = useCallback(() => {
    if (loading) return; // 双重提交防护
    const selectedIndices = files
      .filter((f) => f.selected)
      .map((f) => f.index);

    if (selectedIndices.length === 0) {
      return;
    }

    setLoading(true);
    onConfirm(selectedIndices);
  }, [files, onConfirm, loading]);

  // 快捷键
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onCancel();
      } else if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
        handleConfirm();
      } else if (e.key === "a" && (e.ctrlKey || e.metaKey)) {
        e.preventDefault();
        handleToggleAll();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [onCancel, handleConfirm, handleToggleAll]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onCancel}>
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col animate-fade-in"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <div className="flex items-center gap-2">
            <Folder size={18} className="text-accent" />
            <h2 className="text-lg font-semibold text-text-primary truncate max-w-md" title={fileList.name}>
              {fileList.name}
            </h2>
          </div>
          <button onClick={onCancel} aria-label="Close" className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted">
            <X size={18} />
          </button>
        </div>

        {/* 工具栏 */}
        <div className="flex items-center justify-between px-5 py-2 border-b border-border bg-secondary">
          <div className="flex items-center gap-3">
            <button
              onClick={handleToggleAll}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded text-sm text-text-secondary hover:bg-tertiary transition-colors"
            >
              {allSelected ? (
                <Check size={14} className="text-accent" />
              ) : someSelected ? (
                <Minus size={14} className="text-accent" />
              ) : (
                <div className="w-3.5 h-3.5 rounded border border-border" />
              )}
              {t("fileSelect.selectAll")}
            </button>
            <button
              onClick={handleInvertSelection}
              className="px-3 py-1.5 rounded text-sm text-text-secondary hover:bg-tertiary transition-colors"
            >
              {t("fileSelect.invertSelection")}
            </button>
          </div>
          <div className="text-sm text-text-muted">
            {t("fileSelect.selectedCount", {
              selected: String(selectedCount),
              total: String(files.length),
            })}
          </div>
        </div>

        {/* 文件列表 */}
        <div className="flex-1 overflow-auto px-5 py-2">
          {files.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-32 text-text-muted">
              <File size={32} className="mb-2 opacity-30" />
              <p className="text-sm">{t("fileSelect.noFiles")}</p>
            </div>
          ) : (
            <div className="space-y-1">
              {files.map((file) => (
                <div
                  key={file.index}
                  onClick={() => handleToggleFile(file.index)}
                  className={`flex items-center gap-3 px-3 py-2 rounded cursor-pointer transition-colors ${
                    file.selected
                      ? "bg-accent/10 hover:bg-accent/15"
                      : "hover:bg-tertiary"
                  }`}
                >
                  {/* 复选框 */}
                  <div
                    className={`w-4 h-4 rounded border flex items-center justify-center flex-shrink-0 ${
                      file.selected
                        ? "bg-accent border-accent"
                        : "border-border"
                    }`}
                  >
                    {file.selected && <Check size={12} className="text-white" />}
                  </div>

                  {/* 文件图标 */}
                  <File size={16} className="text-text-muted flex-shrink-0" />

                  {/* 文件名 */}
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-text-primary truncate" title={file.name}>
                      {file.name}
                    </div>
                  </div>

                  {/* 文件大小 */}
                  <div className="text-xs text-text-muted font-mono flex-shrink-0">
                    {formatSize(file.size)}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* 底部信息和按钮 */}
        <div className="flex items-center justify-between px-5 py-3 border-t border-border bg-secondary">
          <div className="text-sm text-text-secondary">
            {t("fileSelect.totalSize")}: <span className="font-mono">{formatSize(selectedSize)}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-text-muted">
              Ctrl+Enter {t("common.confirm")} | Esc {t("common.cancel")}
            </span>
            <button
              onClick={onCancel}
              className="px-4 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary transition-colors"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleConfirm}
              disabled={selectedCount === 0 || loading}
              className="px-4 py-1.5 rounded-md text-sm text-white bg-accent hover:bg-accent-hover disabled:opacity-50 transition-colors flex items-center gap-1.5"
            >
              {loading && <Loader2 size={14} className="animate-spin" />}
              {loading ? t("fileSelect.adding") : t("fileSelect.download")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
