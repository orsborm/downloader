// RSS 订阅管理组件
// 支持添加/删除/编辑订阅、过滤规则、OPML 导入/导出

import { useState, useCallback, useEffect } from "react";
import { X, Rss, Plus, Trash2, Download, Upload, RefreshCw } from "lucide-react";
import {
  addRssFeed,
  removeRssFeed,
  getRssFeeds,
  importOpml,
  exportOpml,
  type RssFeedInfo,
} from "../lib/tauri-api";
import { showToast } from "./Toast";
import { withErrorHandling, handleError, ErrorCode } from "../lib/errors";
import { useI18n } from "../hooks/useI18n";

interface RssManagerProps {
  onClose: () => void;
}

export function RssManager({ onClose }: RssManagerProps) {
  const { t } = useI18n();
  const [feeds, setFeeds] = useState<RssFeedInfo[]>([]);
  const [showAdd, setShowAdd] = useState(false);
  const [newName, setNewName] = useState("");
  const [newUrl, setNewUrl] = useState("");
  const [newInterval, setNewInterval] = useState(30);
  const [loading, setLoading] = useState(true);

  // 加载订阅列表
  const loadFeeds = useCallback(async () => {
    try {
      const list = await getRssFeeds();
      setFeeds(list);
    } catch (e) {
      handleError(e, t("rss.title"), ErrorCode.RSS_FETCH_FAILED);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadFeeds();
  }, [loadFeeds]);

  // 添加订阅
  const handleAdd = useCallback(async () => {
    if (!newUrl.trim()) return;
    const result = await withErrorHandling(
      () => addRssFeed(
        newName.trim() || newUrl.trim(),
        newUrl.trim(),
        newInterval
      ),
      t("rss.addFeed"),
      ErrorCode.RSS_FETCH_FAILED
    );
    if (result !== undefined) {
      setNewName("");
      setNewUrl("");
      setShowAdd(false);
      await loadFeeds();
    }
  }, [newName, newUrl, newInterval, loadFeeds]);

  // 删除订阅（需确认）
  const handleDelete = useCallback(
    async (id: string) => {
      const { confirm } = await import("./ConfirmDialog");
      const ok = await confirm({
        title: t("rss.deleteFeed"),
        message: t("rss.deleteConfirm"),
        variant: "danger",
      });
      if (!ok) return;
      const result = await withErrorHandling(
        () => removeRssFeed(id),
        t("rss.deleteFeed"),
        ErrorCode.RSS_FETCH_FAILED
      );
      if (result !== undefined) {
        await loadFeeds();
      }
    },
    [loadFeeds, t]
  );

  // 导入 OPML
  const handleImportOpml = useCallback(async () => {
    try {
      // 使用 Tauri 文件对话框读取 OPML 文件
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".opml,.xml";
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        const content = await file.text();
        const count = await withErrorHandling(
          () => importOpml(content),
          t("rss.importOpml"),
          ErrorCode.RSS_PARSE_FAILED
        );
        if (count !== undefined) {
          showToast(t("rss.importSuccess", { count: String(count) }), "success");
          await loadFeeds();
        }
      };
      input.click();
    } catch (e) {
      handleError(e, t("rss.importOpml"), ErrorCode.RSS_PARSE_FAILED);
    }
  }, [loadFeeds]);

  // 导出 OPML
  const handleExportOpml = useCallback(async () => {
    const content = await withErrorHandling(
      () => exportOpml(),
      t("rss.exportOpml"),
      ErrorCode.RSS_PARSE_FAILED
    );
    if (content) {
      const blob = new Blob([content], { type: "text/xml" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = "feeds.opml";
      a.click();
      URL.revokeObjectURL(url);
    }
  }, []);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <h2 className="text-lg font-semibold text-text-primary flex items-center gap-2">
            <Rss size={20} />
            {t("rss.title")}
          </h2>
          <button onClick={onClose} className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted">
            <X size={18} />
          </button>
        </div>

        {/* 工具栏 */}
        <div className="flex items-center gap-2 px-5 py-3 border-b border-border">
          <button
            onClick={() => setShowAdd(!showAdd)}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm bg-accent text-white hover:bg-accent-hover transition-colors"
          >
            <Plus size={14} />
            {t("rss.addFeed")}
          </button>
          <button
            onClick={handleImportOpml}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary"
          >
            <Upload size={14} />
            {t("rss.importOpml")}
          </button>
          <button
            onClick={handleExportOpml}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary"
          >
            <Download size={14} />
            {t("rss.exportOpml")}
          </button>
          <div className="flex-1" />
          <button
            onClick={loadFeeds}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary"
          >
            <RefreshCw size={14} />
            {t("rss.refresh")}
          </button>
        </div>

        {/* 添加订阅表单 */}
        {showAdd && (
          <div className="px-5 py-3 border-b border-border bg-tertiary">
            <div className="flex gap-2 mb-2">
              <input
                type="text"
                value={newName}
                onChange={(e) => setNewName(e.target.value)}
                placeholder={t("rss.feedNameOptional")}
                className="flex-1 px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
              />
              <input
                type="number"
                value={newInterval}
                onChange={(e) => setNewInterval(Math.max(1, parseInt(e.target.value) || 30))}
                placeholder={t("rss.checkInterval", { interval: "30" })}
                className="w-28 px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
              />
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                value={newUrl}
                onChange={(e) => setNewUrl(e.target.value)}
                placeholder={t("rss.feedUrlPlaceholder")}
                className="flex-1 px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
              />
              <button
                onClick={handleAdd}
                className="px-4 py-1.5 rounded bg-accent text-white text-sm hover:bg-accent-hover"
              >
                {t("rss.addFeed")}
              </button>
            </div>
          </div>
        )}

        {/* 订阅列表 */}
        <div className="flex-1 overflow-auto p-4">
          {loading ? (
            <div className="text-center text-text-muted py-12">{t("common.loading")}</div>
          ) : feeds.length === 0 ? (
            <div className="text-center text-text-muted py-12">
              <Rss size={48} className="mx-auto mb-3 opacity-30" />
              <p>{t("rss.empty")}</p>
              <p className="text-xs mt-1">{t("rss.feedHint")}</p>
            </div>
          ) : (
            <div className="space-y-2">
              {feeds.map((feed) => (
                <div
                  key={feed.id}
                  className="flex items-center gap-3 p-3 rounded-lg border border-border hover:bg-tertiary transition-colors"
                >
                  <Rss size={16} className={feed.enabled ? "text-accent" : "text-text-muted"} />
                  <div className="flex-1 min-w-0">
                    <div className="font-medium text-text-primary text-sm">{feed.name}</div>
                    <div className="text-xs text-text-muted truncate" title={feed.url}>{feed.url}</div>
                    <div className="text-xs text-text-muted">
                      {t("rss.checkInterval", { interval: feed.interval })}
                      {feed.lastUpdate && ` · ${t("rss.lastUpdate")}: ${feed.lastUpdate}`}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <span
                      className={`px-2 py-1 rounded text-xs ${
                        feed.enabled
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-gray-100 text-gray-500 dark:bg-gray-900/30 dark:text-gray-400"
                      }`}
                    >
                      {feed.enabled ? t("rss.enabled") : t("rss.disabled")}
                    </span>
                    <button
                      onClick={() => handleDelete(feed.id)}
                      className="p-1.5 rounded hover:bg-error/10 text-error"
                    >
                      <Trash2 size={14} />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
