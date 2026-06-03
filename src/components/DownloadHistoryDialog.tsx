// 下载历史对话框组件
// 显示已完成的下载记录，支持搜索过滤、统计摘要、流量图表、CSV 导出、清空历史

import { useState, useEffect, useCallback, useMemo } from "react";
import { X, Trash2, Clock, Download, Search, BarChart3, FileDown, List } from "lucide-react";
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from "recharts";
import { getDownloadHistory, clearDownloadHistory } from "../lib/tauri-api";
import {
  formatSize, formatSpeed, formatDateTime, formatDuration, protocolLabel,
  filterHistory, computeHistoryStats, aggregateByDay,
} from "../lib/format";
import type { DownloadHistory } from "../lib/types";
import { showToast } from "./Toast";
import { extractErrorMessage } from "../lib/errors";
import { useI18n } from "../hooks/useI18n";

interface DownloadHistoryDialogProps {
  onClose: () => void;
}

type ViewMode = "list" | "chart";

/** 统计摘要 */
function StatsBar({ items }: { items: DownloadHistory[] }) {
  const { t } = useI18n();
  const stats = useMemo(() => computeHistoryStats(items), [items]);

  return (
    <div className="flex items-center gap-4 px-4 py-2 bg-tertiary/50 border-b border-border text-xs">
      <div className="flex items-center gap-1 text-text-secondary">
        <BarChart3 size={14} className="text-accent" />
        <span>{t("downloadHistory.totalRecords", { count: String(stats.count) })}</span>
      </div>
      <div className="text-text-secondary">
        {t("downloadHistory.stats.totalSize")}: <strong className="text-text-primary">{formatSize(stats.totalSize)}</strong>
      </div>
      <div className="text-text-secondary">
        {t("downloadHistory.stats.avgSpeed")}: <strong className="text-text-primary">{formatSpeed(stats.avgSpeed)}</strong>
      </div>
      <div className="text-text-secondary">
        {t("downloadHistory.stats.totalDuration")}: <strong className="text-text-primary">{formatDuration(stats.totalDuration)}</strong>
      </div>
    </div>
  );
}

/** 流量图表 */
function TrafficChart({ items }: { items: DownloadHistory[] }) {
  const { t } = useI18n();
  const data = useMemo(() => aggregateByDay(items, 30), [items]);
  const hasData = data.some((d) => d.downloads > 0);

  if (!hasData) {
    return (
      <div className="flex items-center justify-center py-12 text-text-muted">
        {t("common.noData")}
      </div>
    );
  }

  return (
    <div className="px-4 py-3">
      <div className="text-xs text-text-muted mb-2">{t("downloadHistory.chartTitle")}</div>
      <ResponsiveContainer width="100%" height={200}>
        <BarChart data={data} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" />
          <XAxis
            dataKey="date"
            tickFormatter={(v: string) => v.slice(5)}
            tick={{ fontSize: 10, fill: "var(--text-muted)" }}
            interval="preserveStartEnd"
          />
          <YAxis
            tick={{ fontSize: 10, fill: "var(--text-muted)" }}
            allowDecimals={false}
            width={30}
          />
          <Tooltip
            contentStyle={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border)",
              borderRadius: 6,
              fontSize: 12,
            }}
            formatter={(value: number, name: string) => [
              name === "downloads" ? `${value}` : formatSize(value),
              name === "downloads" ? t("downloadHistory.stats.downloadCount") || "下载数" : t("downloadHistory.stats.totalSize"),
            ]}
            labelFormatter={(label: string) => label}
          />
          <Bar dataKey="downloads" fill="var(--accent)" radius={[2, 2, 0, 0]} name="downloads" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}

/** 导出为 CSV */
function exportCsv(items: DownloadHistory[]) {
  const header = "Name,Protocol,Size,Downloaded,Avg Speed,Duration(s),Completed At\n";
  const rows = items.map(i =>
    [
      `"${(i.name || "").replace(/"/g, '""')}"`,
      protocolLabel(i.protocol),
      i.totalSize || 0,
      i.downloaded || 0,
      i.averageSpeed || 0,
      i.duration || 0,
      `"${i.completedAt || ""}"`,
    ].join(",")
  ).join("\n");
  const csv = "﻿" + header + rows; // BOM for Excel 中文兼容
  const blob = new Blob([csv], { type: "text/csv;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `download_history_${new Date().toISOString().slice(0, 10)}.csv`;
  a.click();
  URL.revokeObjectURL(url);
}

export function DownloadHistoryDialog({ onClose }: DownloadHistoryDialogProps) {
  const { t } = useI18n();
  const [history, setHistory] = useState<DownloadHistory[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [viewMode, setViewMode] = useState<ViewMode>("list");

  const loadHistory = useCallback(async () => {
    try {
      const data = await getDownloadHistory(500);
      setHistory(data);
    } catch (e) {
      showToast(`${t("common.error")}: ${extractErrorMessage(e)}`, "error");
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    loadHistory();
  }, [loadHistory]);

  const handleClear = useCallback(async () => {
    const { confirm } = await import("./ConfirmDialog");
    const ok = await confirm({
      title: t("downloadHistory.clearAll"),
      message: t("downloadHistory.clearConfirm"),
      variant: "danger",
    });
    if (!ok) return;
    try {
      await clearDownloadHistory();
      setHistory([]);
    } catch (e) {
      showToast(`${t("common.error")}: ${extractErrorMessage(e)}`, "error");
    }
  }, [t]);

  const filtered = useMemo(() => filterHistory(history, search), [history, search]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div className="bg-secondary rounded-lg shadow-xl w-[90vw] max-w-[800px] max-h-[80vh] flex flex-col border border-border" onClick={(e) => e.stopPropagation()}>
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-2">
            <Clock size={18} className="text-accent" />
            <h2 className="text-base font-semibold text-text-primary">{t("downloadHistory.title")}</h2>
            <span className="text-xs text-text-muted">
              ({t("downloadHistory.totalRecords", { count: String(search ? filtered.length : history.length) })})
            </span>
          </div>
          <div className="flex items-center gap-2">
            {/* 视图切换 */}
            <div className="flex items-center border border-border rounded overflow-hidden">
              <button
                onClick={() => setViewMode("list")}
                className={`p-1.5 transition-colors ${viewMode === "list" ? "bg-accent text-white" : "text-text-secondary hover:bg-tertiary"}`}
                title={t("downloadHistory.listView")}
              >
                <List size={14} />
              </button>
              <button
                onClick={() => setViewMode("chart")}
                className={`p-1.5 transition-colors ${viewMode === "chart" ? "bg-accent text-white" : "text-text-secondary hover:bg-tertiary"}`}
                title={t("downloadHistory.chartView")}
              >
                <BarChart3 size={14} />
              </button>
            </div>
            {history.length > 0 && (
              <>
                <button
                  onClick={() => exportCsv(filtered)}
                  className="flex items-center gap-1 px-2 py-1 rounded text-xs
                             text-accent hover:bg-accent/10 transition-colors"
                  title={t("downloadHistory.exportCsv")}
                >
                  <FileDown size={14} />
                  {t("downloadHistory.exportCsv")}
                </button>
                <button
                  onClick={handleClear}
                  className="flex items-center gap-1 px-2 py-1 rounded text-xs
                             text-error hover:bg-error/10 transition-colors"
                >
                  <Trash2 size={14} />
                  {t("downloadHistory.clearAll")}
                </button>
              </>
            )}
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-tertiary transition-colors text-text-secondary"
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* 搜索栏 */}
        <div className="px-4 py-2 border-b border-border">
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-text-muted" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("downloadHistory.searchPlaceholder")}
              className="w-full pl-8 pr-3 py-1.5 text-sm bg-tertiary border border-border rounded
                         text-text-primary placeholder:text-text-muted
                         focus:outline-none focus:border-accent transition-colors"
            />
          </div>
        </div>

        {/* 统计摘要 */}
        {!loading && filtered.length > 0 && <StatsBar items={filtered} />}

        {/* 内容 */}
        <div className="flex-1 overflow-y-auto">
          {loading ? (
            <div className="flex items-center justify-center py-12 text-text-muted">
              {t("common.loading")}
            </div>
          ) : filtered.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 text-text-muted gap-2">
              <Download size={32} className="opacity-30" />
              <span>{search ? t("taskList.noMatch") : t("downloadHistory.empty")}</span>
            </div>
          ) : viewMode === "chart" ? (
            <TrafficChart items={filtered} />
          ) : (
            <table className="w-full text-sm">
              <thead className="sticky top-0 bg-tertiary text-text-secondary text-xs">
                <tr>
                  <th className="text-left px-4 py-2 font-medium">{t("downloadHistory.columns.name")}</th>
                  <th className="text-left px-4 py-2 font-medium w-16">{t("downloadHistory.columns.protocol")}</th>
                  <th className="text-right px-4 py-2 font-medium w-20">{t("downloadHistory.columns.size")}</th>
                  <th className="text-right px-4 py-2 font-medium w-24">{t("downloadHistory.columns.avgSpeed")}</th>
                  <th className="text-right px-4 py-2 font-medium w-20">{t("downloadHistory.columns.duration")}</th>
                  <th className="text-right px-4 py-2 font-medium w-36">{t("downloadHistory.columns.completedAt")}</th>
                </tr>
              </thead>
              <tbody>
                {filtered.map((item) => (
                  <tr
                    key={item.id}
                    className="border-b border-border/30 hover:bg-tertiary/50 transition-colors"
                  >
                    <td className="px-4 py-2 text-text-primary truncate max-w-[300px]" title={item.name}>
                      {item.name}
                    </td>
                    <td className="px-4 py-2">
                      <span className="inline-block px-1.5 py-0.5 rounded text-xs bg-accent/10 text-accent">
                        {protocolLabel(item.protocol)}
                      </span>
                    </td>
                    <td className="px-4 py-2 text-right text-text-secondary font-mono">
                      {formatSize(item.totalSize)}
                    </td>
                    <td className="px-4 py-2 text-right text-text-secondary font-mono">
                      {formatSpeed(item.averageSpeed)}
                    </td>
                    <td className="px-4 py-2 text-right text-text-secondary text-xs">
                      {formatDuration(item.duration)}
                    </td>
                    <td className="px-4 py-2 text-right text-text-muted text-xs">
                      {formatDateTime(item.completedAt)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </div>
  );
}
