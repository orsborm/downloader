// 插件管理组件
// 展示已安装插件列表，支持安装/卸载/启用/禁用

import { useState, useEffect, useCallback } from "react";
import { X, Package, Power, Trash2, Upload, RefreshCw } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { withErrorHandling, handleError, ErrorCode, extractErrorMessage } from "../lib/errors";
import type { PluginInfo } from "../lib/tauri-api";
import { useI18n } from "../hooks/useI18n";

interface PluginManagerProps {
  onClose: () => void;
}

export function PluginManager({ onClose }: PluginManagerProps) {
  const { t } = useI18n();
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [installPath, setInstallPath] = useState("");
  const [installing, setInstalling] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 加载插件列表
  const loadPlugins = useCallback(async () => {
    setLoading(true);
    try {
      const list = await invoke<PluginInfo[]>("list_plugins");
      setPlugins(list);
    } catch (e) {
      handleError(e, t("plugin.title"), ErrorCode.PLUGIN_LOAD_FAILED);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPlugins();
  }, [loadPlugins]);

  // 安装插件
  const handleInstall = useCallback(async () => {
    if (!installPath.trim()) return;
    setInstalling(true);
    setError(null);
    try {
      await invoke("install_plugin", { wasmPath: installPath.trim() });
      setInstallPath("");
      loadPlugins();
    } catch (e) {
      setError(extractErrorMessage(e));
    } finally {
      setInstalling(false);
    }
  }, [installPath, loadPlugins]);

  // 启用/禁用插件
  const handleToggle = useCallback(async (id: string, enable: boolean) => {
    const result = await withErrorHandling(
      () => invoke(enable ? "enable_plugin" : "disable_plugin", { id }),
      t("plugin.enablePlugin"),
      ErrorCode.PLUGIN_EXEC_FAILED
    );
    if (result !== undefined) {
      loadPlugins();
    }
  }, [loadPlugins]);

  // 卸载插件（需确认）
  const handleUninstall = useCallback(async (id: string) => {
    const { confirm } = await import("./ConfirmDialog");
    const ok = await confirm({
      title: t("plugin.uninstallPlugin"),
      message: t("plugin.uninstallConfirm"),
      variant: "danger",
    });
    if (!ok) return;
    const result = await withErrorHandling(
      () => invoke("uninstall_plugin", { id }),
      t("plugin.uninstall"),
      ErrorCode.PLUGIN_EXEC_FAILED
    );
    if (result !== undefined) {
      loadPlugins();
    }
  }, [loadPlugins, t]);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[80vh] flex flex-col animate-fade-in"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <h2 className="text-lg font-semibold text-text-primary flex items-center gap-2">
            <Package size={20} />
            {t("plugin.title")}
          </h2>
          <button onClick={onClose} aria-label="Close" className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted">
            <X size={18} />
          </button>
        </div>

        {/* 安装区域 */}
        <div className="px-5 py-3 border-b border-border space-y-2">
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={installPath}
              onChange={(e) => setInstallPath(e.target.value)}
              placeholder={t("plugin.inputPlaceholder")}
              className="flex-1 px-3 py-1.5 text-sm rounded-md border border-border bg-secondary text-text-primary placeholder:text-text-muted focus:outline-none focus:ring-1 focus:ring-accent"
              onKeyDown={(e) => e.key === "Enter" && handleInstall()}
            />
            <button
              onClick={handleInstall}
              disabled={installing || !installPath.trim()}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm bg-accent text-white hover:bg-accent-hover transition-colors disabled:opacity-50"
            >
              <Upload size={14} />
              {installing ? t("plugin.installing") : t("plugin.installPlugin")}
            </button>
            <button
              onClick={loadPlugins}
              className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary transition-colors"
            >
              <RefreshCw size={14} />
              {t("plugin.refresh")}
            </button>
          </div>
          {error && (
            <p className="text-xs text-error">{error}</p>
          )}
        </div>

        {/* 插件列表 */}
        <div className="flex-1 overflow-auto p-4">
          {loading ? (
            <div className="text-center text-text-muted py-12">{t("common.loading")}</div>
          ) : plugins.length === 0 ? (
            <div className="text-center text-text-muted py-12">
              <Package size={48} className="mx-auto mb-3 opacity-30" />
              <p>{t("plugin.empty")}</p>
              <p className="text-xs mt-1">{t("plugin.emptyHint")}</p>
              <p className="text-xs mt-1 text-accent/60">{t("plugin.emptyDesc")}</p>
            </div>
          ) : (
            <div className="space-y-2">
              {plugins.map((plugin) => (
                <div
                  key={plugin.id}
                  className="flex items-center gap-3 p-3 rounded-lg border border-border hover:bg-tertiary transition-colors"
                >
                  <Package size={20} className="text-accent flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-text-primary">{plugin.name}</span>
                      <span className="text-xs text-text-muted">v{plugin.version}</span>
                      <span className={`text-xs px-1.5 py-0.5 rounded ${
                        plugin.state === "enabled"
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : plugin.state === "disabled"
                          ? "bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400"
                          : "bg-gray-100 text-gray-700 dark:bg-gray-900/30 dark:text-gray-400"
                      }`}>
                        {plugin.state === "enabled" ? t("plugin.pluginEnabled") : plugin.state === "disabled" ? t("plugin.pluginDisabled") : t("plugin.pluginInstalled")}
                      </span>
                    </div>
                    <div className="text-xs text-text-secondary truncate" title={plugin.description || t("plugin.noDescription")}>
                      {plugin.description || t("plugin.noDescription")}
                    </div>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      onClick={() => handleToggle(plugin.id, plugin.state !== "enabled")}
                      title={plugin.state === "enabled" ? t("plugin.disablePlugin") : t("plugin.enablePlugin")}
                      className="p-1.5 rounded hover:bg-tertiary text-text-secondary"
                    >
                      <Power size={14} />
                    </button>
                    <button
                      onClick={() => handleUninstall(plugin.id)}
                      title={t("plugin.uninstall")}
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
