// 批量导入对话框组件
// 支持：文本文件导入、剪贴板导入、通配符生成、正则替换

import { useState, useCallback } from "react";
import { X, FileText, Clipboard, Wand2 } from "lucide-react";
import { addTask, getAllTasks } from "../lib/tauri-api";
import { useTaskStore } from "../stores/taskStore";
import { showToast } from "./Toast";
import { useI18n } from "../hooks/useI18n";

interface BatchImportDialogProps {
  onClose: () => void;
}

type ImportMode = "text" | "clipboard" | "wildcard";

export function BatchImportDialog({ onClose }: BatchImportDialogProps) {
  const { t } = useI18n();
  const [mode, setMode] = useState<ImportMode>("text");
  const [urls, setUrls] = useState("");
  const [wildcardPattern, setWildcardPattern] = useState("");
  const [rangeStart, setRangeStart] = useState("1");
  const [rangeEnd, setRangeEnd] = useState("100");
  const [padding, setPadding] = useState(0);
  const [savePath, setSavePath] = useState("");
  const [startImmediately, setStartImmediately] = useState(true);
  const [deduplicate, setDeduplicate] = useState(true);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);

  // 解析链接列表
  const parseUrls = useCallback((): string[] => {
    let raw = "";

    if (mode === "wildcard" && wildcardPattern) {
      // 通配符生成
      const start = parseInt(rangeStart) || 1;
      const end = parseInt(rangeEnd) || 100;
      const lines: string[] = [];
      for (let i = start; i <= end; i++) {
        const num = padding > 0 ? String(i).padStart(padding, "0") : String(i);
        lines.push(wildcardPattern.replace(/\*/g, num));
      }
      raw = lines.join("\n");
    } else {
      raw = urls;
    }

    // 按行分割，过滤空行
    let lines = raw
      .split("\n")
      .map((l) => l.trim())
      .filter((l) => l.length > 0);

    // 去重
    if (deduplicate) {
      const seen = new Set<string>();
      lines = lines.filter((l) => {
        if (seen.has(l)) return false;
        seen.add(l);
        return true;
      });
    }

    return lines;
  }, [mode, urls, wildcardPattern, rangeStart, rangeEnd, padding, deduplicate]);

  // 提交批量导入
  const handleSubmit = useCallback(async () => {
    if (loading) return; // 双重提交防护
    const urlList = parseUrls();
    if (urlList.length === 0) {
      setResult(t("batchImport.noValidLinks"));
      return;
    }

    setLoading(true);
    setResult(null);

    let success = 0;
    let failed = 0;

    for (const url of urlList) {
      try {
        await addTask({
          url,
          savePath: savePath.trim(),
          startImmediately,
        });
        success++;
      } catch {
        failed++;
      }
    }

    setResult(t("batchImport.importComplete", { success: String(success), failed: String(failed) }));

    // 刷新任务列表
    if (success > 0) {
      try {
        const fresh = await getAllTasks();
        useTaskStore.getState().setTasks(fresh);
      } catch { /* ignore */ }
      showToast(t("batchImport.importSuccess", { count: String(success) }), "success");
    }

    setLoading(false);
  }, [loading, parseUrls, savePath, startImmediately, t]);

  // 从剪贴板导入
  const handlePaste = useCallback(async () => {
    try {
      const text = await navigator.clipboard.readText();
      setUrls(text);
    } catch {
      setResult(t("batchImport.clipboardError"));
    }
  }, []);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-lg mx-4 animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <h2 className="text-lg font-semibold text-text-primary">{t("batchImport.title")}</h2>
          <button onClick={onClose} className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted">
            <X size={18} />
          </button>
        </div>

        {/* 内容 */}
        <div className="px-5 py-4 space-y-4">
          {/* 导入方式切换 */}
          <div className="flex gap-1 bg-tertiary rounded-lg p-1">
            <button
              onClick={() => setMode("text")}
              className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
                mode === "text" ? "bg-primary text-text-primary shadow-sm" : "text-text-secondary"
              }`}
            >
              <FileText size={14} />
              {t("batchImport.importFromText")}
            </button>
            <button
              onClick={() => setMode("clipboard")}
              className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
                mode === "clipboard" ? "bg-primary text-text-primary shadow-sm" : "text-text-secondary"
              }`}
            >
              <Clipboard size={14} />
              {t("batchImport.importFromClipboard")}
            </button>
            <button
              onClick={() => setMode("wildcard")}
              className={`flex-1 flex items-center justify-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
                mode === "wildcard" ? "bg-primary text-text-primary shadow-sm" : "text-text-secondary"
              }`}
            >
              <Wand2 size={14} />
              {t("batchImport.generateWildcard")}
            </button>
          </div>

          {/* 文本/剪贴板模式 */}
          {(mode === "text" || mode === "clipboard") && (
            <div>
              <div className="flex items-center justify-between mb-1">
                <label className="text-sm text-text-secondary">{t("batchImport.linkList")}</label>
                {mode === "clipboard" && (
                  <button onClick={handlePaste} className="text-xs text-accent hover:underline">
                    {t("batchImport.pasteFromClipboard")}
                  </button>
                )}
              </div>
              <textarea
                value={urls}
                onChange={(e) => setUrls(e.target.value)}
                placeholder="https://example.com/file1.zip&#10;https://example.com/file2.zip&#10;..."
                rows={8}
                className="w-full px-3 py-2 rounded border border-border bg-secondary text-text-primary text-sm font-mono resize-y focus:outline-none focus:ring-1 focus:ring-accent/50"
              />
            </div>
          )}

          {/* 通配符模式 */}
          {mode === "wildcard" && (
            <div className="space-y-3">
              <div>
                <label className="text-sm text-text-secondary mb-1 block">{t("batchImport.urlTemplate")}</label>
                <input
                  type="text"
                  value={wildcardPattern}
                  onChange={(e) => setWildcardPattern(e.target.value)}
                  placeholder="https://example.com/file(*).zip"
                  className="w-full px-3 py-2 rounded border border-border bg-secondary text-text-primary text-sm focus:outline-none focus:ring-1 focus:ring-accent/50"
                />
              </div>
              <div className="flex gap-2">
                <div className="flex-1">
                  <label className="text-xs text-text-muted">{t("batchImport.rangeStart")}</label>
                  <input
                    type="number"
                    value={rangeStart}
                    onChange={(e) => setRangeStart(e.target.value)}
                    className="w-full px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
                  />
                </div>
                <div className="flex-1">
                  <label className="text-xs text-text-muted">{t("batchImport.rangeEnd")}</label>
                  <input
                    type="number"
                    value={rangeEnd}
                    onChange={(e) => setRangeEnd(e.target.value)}
                    className="w-full px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
                  />
                </div>
                <div className="flex-1">
                  <label className="text-xs text-text-muted">{t("batchImport.paddingDigits")}</label>
                  <input
                    type="number"
                    value={padding}
                    onChange={(e) => setPadding(parseInt(e.target.value) || 0)}
                    placeholder="0"
                    className="w-full px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
                  />
                </div>
              </div>
            </div>
          )}

          {/* 保存目录 */}
          <div>
            <label className="text-sm text-text-secondary mb-1 block">{t("addTask.savePath")}</label>
            <input
              type="text"
              value={savePath}
              onChange={(e) => setSavePath(e.target.value)}
              placeholder={t("addTask.defaultDirPlaceholder")}
              className="w-full px-3 py-2 rounded border border-border bg-secondary text-text-primary text-sm focus:outline-none focus:ring-1 focus:ring-accent/50"
            />
          </div>

          {/* 选项 */}
          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 cursor-pointer text-sm">
              <input
                type="checkbox"
                checked={startImmediately}
                onChange={(e) => setStartImmediately(e.target.checked)}
                className="w-4 h-4 rounded border-border text-accent"
              />
              {t("addTask.startImmediatelyDownload")}
            </label>
            <label className="flex items-center gap-2 cursor-pointer text-sm">
              <input
                type="checkbox"
                checked={deduplicate}
                onChange={(e) => setDeduplicate(e.target.checked)}
                className="w-4 h-4 rounded border-border text-accent"
              />
              {t("batchImport.autoDeduplicate")}
            </label>
          </div>

          {/* 预览 */}
          {parseUrls().length > 0 && (
            <div className="text-xs text-text-muted">
              {t("batchImport.linkCount", { count: String(parseUrls().length) })}
            </div>
          )}

          {/* 结果 */}
          {result && (
            <div className="text-sm text-accent bg-accent/10 px-3 py-2 rounded">
              {result}
            </div>
          )}
        </div>

        {/* 底部 */}
        <div className="flex justify-end gap-2 px-5 py-3 border-t border-border">
          <button
            onClick={onClose}
            className="px-4 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary transition-colors"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleSubmit}
            disabled={loading}
            className="px-4 py-1.5 rounded-md text-sm text-white bg-accent hover:bg-accent-hover disabled:opacity-50 transition-colors"
          >
            {loading ? t("batchImport.importing") : t("batchImport.startImport")}
          </button>
        </div>
      </div>
    </div>
  );
}
