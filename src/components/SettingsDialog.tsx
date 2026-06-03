// 设置中心对话框组件
// 左侧分类导航 + 右侧表单
// 分类：常规、下载、连接、BT、HTTP、通知、高级

import { useState, useEffect, useCallback, useRef } from "react";
import { X, Settings, Download, Wifi, Radio, Bell, Sliders, Globe } from "lucide-react";
import { getSettings, updateSettings } from "../lib/tauri-api";
import type { AppConfig } from "../lib/types";
import { handleError, withErrorHandling, ErrorCode } from "../lib/errors";
import { useI18n, LANGUAGE_NAMES, type Language } from "../hooks/useI18n";
import { Toggle } from "./Toggle";

interface SettingsDialogProps {
  onClose: () => void;
}

type SettingsTab = "general" | "download" | "connection" | "bt" | "http" | "notification" | "advanced";

/** 配置字段更新函数类型 */
type UpdateFieldFn = <K extends keyof AppConfig>(
  section: K,
  field: keyof AppConfig[K],
  value: AppConfig[K][keyof AppConfig[K]]
) => void;

/** 设置分类定义 */
const settingTabs: { key: SettingsTab; labelKey: string; icon: React.ReactNode }[] = [
  { key: "general", labelKey: "settings.sections.general", icon: <Settings size={16} /> },
  { key: "download", labelKey: "settings.sections.download", icon: <Download size={16} /> },
  { key: "connection", labelKey: "settings.sections.connection", icon: <Wifi size={16} /> },
  { key: "bt", labelKey: "settings.sections.bittorrent", icon: <Radio size={16} /> },
  { key: "http", labelKey: "settings.sections.http", icon: <Globe size={16} /> },
  { key: "notification", labelKey: "settings.sections.notification", icon: <Bell size={16} /> },
  { key: "advanced", labelKey: "settings.sections.advanced", icon: <Sliders size={16} /> },
];

export function SettingsDialog({ onClose }: SettingsDialogProps) {
  const { t } = useI18n();
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);

  // 加载配置
  useEffect(() => {
    const load = async () => {
      try {
        const cfg = await getSettings();
        setConfig(cfg);
      } catch (e) {
        handleError(e, t("app.loadSettings"), ErrorCode.CONFIG_LOAD_FAILED);
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  // 核心保存逻辑（应用 = 保存但不关闭）
  const savingRef = useRef(false);
  const doSave = useCallback(async (): Promise<boolean> => {
    if (!config || savingRef.current) return false;
    savingRef.current = true;
    setSaving(true);
    const result = await withErrorHandling(
      () => updateSettings(config),
      t("app.saveSettings"),
      ErrorCode.CONFIG_SAVE_FAILED
    );
    setSaving(false);
    savingRef.current = false;
    return result !== undefined;
  }, [config, t]);

  // 保存配置并关闭
  const handleSave = useCallback(async () => {
    const ok = await doSave();
    if (ok) onClose();
  }, [doSave, onClose]);

  // 应用配置（保存但不关闭）
  const handleApply = useCallback(async () => {
    await doSave();
  }, [doSave]);

  // 键盘快捷键：Ctrl+S 保存
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "s") {
        e.preventDefault();
        handleSave();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleSave]);

  // 更新配置字段
  const updateField = useCallback(
    <K extends keyof AppConfig>(
      section: K,
      field: keyof AppConfig[K],
      value: AppConfig[K][keyof AppConfig[K]]
    ) => {
      setConfig((prev) => {
        if (!prev) return prev;
        return {
          ...prev,
          [section]: {
            ...prev[section],
            [field]: value,
          },
        };
      });
    },
    []
  );

  if (loading) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
        <div className="bg-primary rounded-lg p-8 text-text-secondary">{t("common.loading")}</div>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
        <div className="bg-primary rounded-lg shadow-xl p-8 max-w-sm mx-4 animate-fade-in" onClick={(e) => e.stopPropagation()}>
          <p className="text-error mb-4">{t("common.error")}</p>
          <button onClick={onClose} className="px-4 py-1.5 rounded-md text-sm text-white bg-accent hover:bg-accent-hover transition-colors">
            {t("common.close")}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      {/* 固定高度 600px，不随内容变化 */}
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-4xl mx-4 h-[min(600px,90vh)] flex flex-col animate-fade-in"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border flex-shrink-0">
          <h2 className="text-lg font-semibold text-text-primary">{t("settings.title")}</h2>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted"
          >
            <X size={18} />
          </button>
        </div>

        {/* 内容区域（固定高度，内部滚动） */}
        <div className="flex flex-1 min-h-0">
          {/* 左侧导航 */}
          <div className="w-44 border-r border-border py-2 bg-secondary flex-shrink-0">
            {settingTabs.map((tab) => (
              <button
                key={tab.key}
                onClick={() => setActiveTab(tab.key)}
                className={`flex items-center gap-2 w-full px-4 py-2.5 text-sm transition-colors
                  ${
                    activeTab === tab.key
                      ? "text-accent bg-accent/10 border-r-2 border-accent"
                      : "text-text-secondary hover:text-text-primary hover:bg-tertiary"
                  }`}
              >
                {tab.icon}
                <span>{t(tab.labelKey)}</span>
              </button>
            ))}
          </div>

          {/* 右侧表单（固定高度，独立滚动） */}
          <div className="flex-1 overflow-y-auto p-5">
            {activeTab === "general" && <GeneralSettings config={config} updateField={updateField} />}
            {activeTab === "download" && <DownloadSettings config={config} updateField={updateField} />}
            {activeTab === "connection" && <ConnectionSettings config={config} updateField={updateField} />}
            {activeTab === "bt" && <BtSettings config={config} updateField={updateField} />}
            {activeTab === "http" && <HttpSettings config={config} updateField={updateField} />}
            {activeTab === "notification" && <NotificationSettings config={config} updateField={updateField} />}
            {activeTab === "advanced" && <AdvancedSettings config={config} updateField={updateField} />}
          </div>
        </div>

        {/* 底部按钮 */}
        <div className="flex justify-end gap-2 px-5 py-3 border-t border-border flex-shrink-0">
          <button
            onClick={onClose}
            className="px-4 py-1.5 rounded-md text-sm text-text-secondary hover:bg-tertiary transition-colors"
          >
            {t("common.cancel")}
          </button>
          <button
            onClick={handleApply}
            disabled={saving}
            className="px-4 py-1.5 min-w-[72px] rounded-md text-sm text-text-primary border border-border
                       hover:bg-tertiary disabled:opacity-50 transition-colors"
          >
            {saving ? `${t("common.apply")}...` : t("common.apply")}
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-1.5 min-w-[72px] rounded-md text-sm text-white bg-accent
                       hover:bg-accent-hover disabled:opacity-50 transition-colors"
          >
            {saving ? `${t("common.save")}...` : t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}

// ==================== 通用组件 ====================

/** 通用设置行组件 */
function SettingRow({
  label,
  description,
  children,
}: {
  label: string;
  description?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between gap-4 py-2.5 border-b border-border/30 last:border-0">
      <div className="min-w-0">
        <div className="text-sm text-text-primary break-words">{label}</div>
        {description && (
          <div className="text-xs text-text-muted mt-0.5 break-words">{description}</div>
        )}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

/** 通用输入框（带 placeholder 默认值提示） */
function SettingInput({
  value,
  onChange,
  type = "text",
  placeholder,
  className = "",
  min,
  max,
  suffix,
}: {
  value: string | number;
  onChange: (value: string) => void;
  type?: string;
  placeholder?: string;
  className?: string;
  min?: number;
  max?: number;
  suffix?: string;
}) {
  return (
    <div className="flex items-center gap-1">
      <input
        type={type}
        value={value}
        min={min}
        max={max}
        onChange={(e) => {
          const v = e.target.value;
          if (type === "number" && min !== undefined) {
            const n = parseInt(v);
            if (!isNaN(n) && n < min) return;
          }
          if (type === "number" && max !== undefined) {
            const n = parseInt(v);
            if (!isNaN(n) && n > max) return;
          }
          onChange(v);
        }}
        placeholder={placeholder}
        className={`px-2.5 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm
          focus:outline-none focus:ring-1 focus:ring-accent/50 ${className}`}
      />
      {suffix && <span className="text-xs text-text-muted">{suffix}</span>}
    </div>
  );
}

// ==================== 各分类设置表单 ====================

function GeneralSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t, language, setLanguage } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.general")}</h3>
      <SettingRow label={t("settings.general.language")}>
        <select
          value={language}
          onChange={(e) => {
            const newLang = e.target.value as Language;
            setLanguage(newLang);
            updateField("general", "language", newLang === "zh" ? "zh-CN" : "en");
          }}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          {Object.entries(LANGUAGE_NAMES).map(([code, name]) => (
            <option key={code} value={code}>{name}</option>
          ))}
        </select>
      </SettingRow>
      <SettingRow label={t("settings.general.theme")}>
        <select
          value={config.general.theme}
          onChange={(e) => {
            const theme = e.target.value;
            updateField("general", "theme", theme);
            // 立即应用主题（不等保存）
            const root = document.documentElement;
            if (theme === "system") {
              const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
              root.setAttribute("data-theme", isDark ? "dark" : "light");
            } else {
              root.setAttribute("data-theme", theme);
            }
            localStorage.setItem("theme", theme);
          }}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="light">{t("settings.general.themeLight")}</option>
          <option value="dark">{t("settings.general.themeDark")}</option>
          <option value="system">{t("settings.general.themeSystem")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.general.fontSize")} description={t("settings.general.fontSizeDesc")}>
        <SettingInput
          value={config.general.fontSize}
          onChange={(v) => {
            const size = Math.max(10, Math.min(32, parseInt(v) || 14));
            updateField("general", "fontSize", size);
            // 立即应用字体大小
            document.documentElement.style.fontSize = `${size}px`;
          }}
          type="number"
          min={10}
          max={32}
          placeholder="14"
          suffix="px"
          className="w-16"
        />
      </SettingRow>
      {/* 合并：关闭/最小化时缩小到托盘 */}
      <SettingRow label={t("settings.general.closeToTray")}>
        <Toggle
          checked={config.general.closeToTray}
          onChange={(v) => {
            updateField("general", "closeToTray", v);
            updateField("general", "minimizeToTray", v);
          }}
        />
      </SettingRow>
      <SettingRow label={t("settings.general.autoStart")} description={t("settings.general.autoStartDesc")}>
        <Toggle checked={config.general.autoStart} onChange={(v) => updateField("general", "autoStart", v)} />
      </SettingRow>
    </div>
  );
}

function DownloadSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.download")}</h3>
      <SettingRow label={t("settings.download.defaultDir")}>
        <SettingInput
          value={config.download.defaultDir}
          onChange={(v) => updateField("download", "defaultDir", v)}
          placeholder="C:\\Users\\...\\Downloads"
          className="w-64"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.completeDir")} description={t("settings.download.completeDirDesc")}>
        <SettingInput
          value={config.download.completeDir}
          onChange={(v) => updateField("download", "completeDir", v)}
          placeholder={t("common.noData")}
          className="w-64"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.tempDir")} description={t("settings.download.tempDirDesc")}>
        <SettingInput
          value={config.download.tempDir}
          onChange={(v) => updateField("download", "tempDir", v)}
          placeholder="./data/temp"
          className="w-64"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.maxConcurrent")} description={t("settings.download.maxConcurrentDesc")}>
        <SettingInput
          value={config.download.maxConcurrentTasks}
          onChange={(v) => updateField("download", "maxConcurrentTasks", Math.max(1, parseInt(v) || 3))}
          type="number"
          min={1}
          max={32}
          placeholder="3"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.maxConnections")}>
        <SettingInput
          value={config.download.maxConnectionsPerTask}
          onChange={(v) => updateField("download", "maxConnectionsPerTask", Math.max(1, parseInt(v) || 64))}
          type="number"
          min={1}
          max={256}
          placeholder="64"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.maxGlobalConnections")}>
        <SettingInput
          value={config.download.maxGlobalConnections}
          onChange={(v) => updateField("download", "maxGlobalConnections", Math.max(1, parseInt(v) || 200))}
          type="number"
          min={1}
          max={1000}
          placeholder="200"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.retryCount")}>
        <SettingInput
          value={config.download.autoRetryCount}
          onChange={(v) => updateField("download", "autoRetryCount", Math.max(0, parseInt(v) || 3))}
          type="number"
          min={0}
          max={99}
          placeholder="3"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.retryInterval")}>
        <SettingInput
          value={config.download.autoRetryInterval}
          onChange={(v) => updateField("download", "autoRetryInterval", Math.max(1, parseInt(v) || 5))}
          type="number"
          min={1}
          max={3600}
          placeholder="5"
          suffix={t("format.seconds")}
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.maxDownloadSpeed")} description={t("settings.download.maxDownloadSpeedDesc")}>
        <SettingInput
          value={config.download.maxDownloadSpeed}
          onChange={(v) => updateField("download", "maxDownloadSpeed", Math.max(0, parseInt(v) || 0))}
          type="number"
          min={0}
          placeholder="0"
          suffix="B/s"
          className="w-24"
        />
      </SettingRow>
      <SettingRow label={t("settings.download.maxUploadSpeed")} description={t("settings.download.maxUploadSpeedDesc")}>
        <SettingInput
          value={config.download.maxUploadSpeed}
          onChange={(v) => updateField("download", "maxUploadSpeed", Math.max(0, parseInt(v) || 0))}
          type="number"
          min={0}
          placeholder="0"
          suffix="B/s"
          className="w-24"
        />
      </SettingRow>
    </div>
  );
}

function ConnectionSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.connection")}</h3>
      <SettingRow label={t("settings.connection.btPort")}>
        <SettingInput
          value={config.connection.btPort}
          onChange={(v) => updateField("connection", "btPort", Math.min(65535, Math.max(1024, parseInt(v) || 6881)))}
          type="number"
          min={1024}
          max={65535}
          placeholder="6881"
          className="w-20"
        />
      </SettingRow>
      <SettingRow label={t("settings.connection.ed2kPort")} description={t("settings.connection.ed2kPortDesc")}>
        <SettingInput
          value={config.connection.ed2kPort}
          onChange={(v) => updateField("connection", "ed2kPort", Math.min(65535, Math.max(1024, parseInt(v) || 4661)))}
          type="number"
          min={1024}
          max={65535}
          placeholder="4661"
          className="w-20"
        />
      </SettingRow>
      <div className="py-2.5 border-b border-border/30">
        <div className="text-sm text-text-primary mb-1">{t("settings.connection.ed2kServers")}</div>
        <div className="text-xs text-text-muted mb-2">{t("settings.connection.ed2kServersDesc")}</div>
        <textarea
          value={(config.connection.ed2kServers || []).join("\n")}
          onChange={(e) => {
            const servers = e.target.value.split("\n").map(s => s.trim()).filter(Boolean);
            updateField("connection", "ed2kServers", servers);
          }}
          placeholder={t("settings.connection.ed2kServersPlaceholder")}
          rows={3}
          className="px-2.5 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm
                     focus:outline-none focus:ring-1 focus:ring-accent/50 w-full resize-y font-mono"
        />
      </div>
      <SettingRow label={t("settings.connection.httpPort")} description={t("settings.connection.httpPortDesc")}>
        <SettingInput
          value={config.connection.httpPort}
          onChange={(v) => updateField("connection", "httpPort", parseInt(v) || 0)}
          type="number"
          placeholder="0"
          className="w-20"
        />
      </SettingRow>
      <SettingRow label={t("settings.connection.apiToken")} description={t("settings.connection.apiTokenDesc")}>
        <SettingInput
          value={config.connection.apiToken}
          onChange={(v) => updateField("connection", "apiToken", v)}
          placeholder={t("settings.connection.apiTokenPlaceholder")}
          className="w-48"
        />
      </SettingRow>
      {/* 端口映射：合并 UPnP 和 NAT-PMP 为一个开关 */}
      <SettingRow label={t("settings.connection.upnp")} description={t("settings.connection.natPmpDesc")}>
        <Toggle checked={config.connection.upnp} onChange={(v) => {
          updateField("connection", "upnp", v);
          updateField("connection", "natPmp", v);
        }} />
      </SettingRow>
      <SettingRow label={t("settings.connection.proxyType")}>
        <select
          value={config.connection.proxyType}
          onChange={(e) => updateField("connection", "proxyType", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="none">{t("settings.connection.proxyNoneLabel")}</option>
          <option value="http">{t("settings.connection.proxyHttpLabel")}</option>
          <option value="socks5">{t("settings.connection.proxySocks5Label")}</option>
        </select>
      </SettingRow>
      {config.connection.proxyType !== "none" && (
        <>
          <SettingRow label={t("settings.connection.proxyAddress")}>
            <SettingInput
              value={config.connection.proxyHost}
              onChange={(v) => updateField("connection", "proxyHost", v)}
              placeholder="127.0.0.1"
              className="w-40"
            />
          </SettingRow>
          <SettingRow label={t("settings.connection.proxyPort")}>
            <SettingInput
              value={config.connection.proxyPort}
              onChange={(v) => updateField("connection", "proxyPort", parseInt(v) || 0)}
              type="number"
              placeholder="1080"
              className="w-20"
            />
          </SettingRow>
          <SettingRow label={t("settings.connection.proxyUsername")}>
            <SettingInput
              value={config.connection.proxyUsername}
              onChange={(v) => updateField("connection", "proxyUsername", v)}
              placeholder={t("settings.connection.proxyUsername")}
              className="w-40"
            />
          </SettingRow>
          <SettingRow label={t("settings.connection.proxyPassword")}>
            <SettingInput
              value={config.connection.proxyPassword}
              onChange={(v) => updateField("connection", "proxyPassword", v)}
              type="password"
              placeholder="••••••"
              className="w-40"
            />
          </SettingRow>
        </>
      )}
      {/* 超时设置：合并连接超时和读取超时 */}
      <SettingRow label={t("settings.connection.timeout")}>
        <div className="flex items-center gap-3">
          <div className="flex items-center gap-1">
            <span className="text-xs text-text-muted w-16">{t("settings.connection.connectTimeout")}</span>
            <SettingInput
              value={config.connection.connectionTimeout}
              onChange={(v) => updateField("connection", "connectionTimeout", parseInt(v) || 30)}
              type="number"
              placeholder="30"
              suffix={t("format.seconds")}
              className="w-16"
            />
          </div>
          <div className="flex items-center gap-1">
            <span className="text-xs text-text-muted w-16">{t("settings.connection.readTimeout")}</span>
            <SettingInput
              value={config.connection.readTimeout}
              onChange={(v) => updateField("connection", "readTimeout", parseInt(v) || 60)}
              type="number"
              placeholder="60"
              suffix={t("format.seconds")}
              className="w-16"
            />
          </div>
        </div>
      </SettingRow>
    </div>
  );
}

function BtSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.bittorrent")}</h3>
      <SettingRow label={t("settings.bittorrent.enableDht")} description={t("settings.bittorrent.dhtDesc")}>
        <Toggle checked={config.bt.dht} onChange={(v) => updateField("bt", "dht", v)} />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.enablePex")} description={t("settings.bittorrent.pexDesc")}>
        <Toggle checked={config.bt.pex} onChange={(v) => updateField("bt", "pex", v)} />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.enableLsd")} description={t("settings.bittorrent.lsdDesc")}>
        <Toggle checked={config.bt.lsd} onChange={(v) => updateField("bt", "lsd", v)} />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.encryption")}>
        <select
          value={config.bt.encryption}
          onChange={(e) => updateField("bt", "encryption", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="disabled">{t("settings.bittorrent.encryptionDisabled")}</option>
          <option value="enabled">{t("settings.bittorrent.encryptionEnabled")}</option>
          <option value="forced">{t("settings.bittorrent.encryptionForced")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.seedRatio")} description={t("settings.bittorrent.seedRatioDesc")}>
        <SettingInput
          value={config.bt.seedRatioLimit}
          onChange={(v) => updateField("bt", "seedRatioLimit", parseFloat(v) || 2.0)}
          type="number"
          placeholder="2.0"
          className="w-20"
        />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.seedTime")} description={t("settings.bittorrent.seedTimeDesc")}>
        <SettingInput
          value={config.bt.seedTimeLimit}
          onChange={(v) => updateField("bt", "seedTimeLimit", parseInt(v) || 1440)}
          type="number"
          placeholder="1440"
          suffix={t("format.minutes")}
          className="w-20"
        />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.trackersFile")} description={t("settings.bittorrent.trackersFileDesc")}>
        <SettingInput
          value={config.bt.trackersFile}
          onChange={(v) => updateField("bt", "trackersFile", v)}
          placeholder="./resources/trackers_best.txt"
          className="w-64"
        />
      </SettingRow>
      <SettingRow label={t("settings.bittorrent.stopSeeding")} description={t("settings.bittorrent.stopSeedingDesc")}>
        <Toggle checked={config.bt.stopSeeding} onChange={(v) => updateField("bt", "stopSeeding", v)} />
      </SettingRow>
    </div>
  );
}

function NotificationSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.notification")}</h3>
      <SettingRow label={t("settings.notification.onComplete")}>
        <Toggle checked={config.notification.taskComplete} onChange={(v) => updateField("notification", "taskComplete", v)} />
      </SettingRow>
      <SettingRow label={t("settings.notification.onError")}>
        <Toggle checked={config.notification.taskError} onChange={(v) => updateField("notification", "taskError", v)} />
      </SettingRow>
      <SettingRow label={t("settings.notification.sound")}>
        <Toggle checked={config.notification.sound} onChange={(v) => updateField("notification", "sound", v)} />
      </SettingRow>
      <SettingRow label={t("settings.notification.position")}>
        <select
          value={config.notification.position}
          onChange={(e) => {
            const pos = e.target.value;
            updateField("notification", "position", pos);
            // 同步到 localStorage 并通知 Toast 组件即时更新
            try { localStorage.setItem("downloader-notification-position", pos); } catch {}
            window.dispatchEvent(new Event("toast-position-changed"));
          }}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="bottom-right">{t("settings.notification.positionBottomRight")}</option>
          <option value="bottom-left">{t("settings.notification.positionBottomLeft")}</option>
          <option value="top-right">{t("settings.notification.positionTopRight")}</option>
          <option value="top-left">{t("settings.notification.positionTopLeft")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.notification.duration")}>
        <SettingInput
          value={config.notification.duration}
          onChange={(v) => updateField("notification", "duration", parseInt(v) || 5)}
          type="number"
          placeholder="5"
          suffix={t("format.seconds")}
          className="w-16"
        />
      </SettingRow>
    </div>
  );
}

function HttpSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.http")}</h3>
      <SettingRow label={t("settings.http.userAgent")} description={t("settings.http.userAgentDesc")}>
        <SettingInput
          value={config.http.userAgent}
          onChange={(v) => updateField("http", "userAgent", v)}
          placeholder="Mozilla/5.0 ..."
          className="w-80"
        />
      </SettingRow>
      <SettingRow label={t("settings.http.cookiePolicy")}>
        <select
          value={config.http.cookiePolicy}
          onChange={(e) => updateField("http", "cookiePolicy", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="auto">{t("settings.http.cookiePolicyAuto")}</option>
          <option value="manual">{t("settings.http.cookiePolicyManual")}</option>
          <option value="disabled">{t("settings.http.cookiePolicyDisabled")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.http.refererPolicy")}>
        <select
          value={config.http.refererPolicy}
          onChange={(e) => updateField("http", "refererPolicy", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="strict">{t("settings.http.refererPolicyStrict")}</option>
          <option value="unsafe">{t("settings.http.refererPolicyUnsafe")}</option>
          <option value="none">{t("settings.http.refererPolicyNone")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.http.maxRedirects")}>
        <SettingInput
          value={config.http.maxRedirects}
          onChange={(v) => updateField("http", "maxRedirects", Math.max(0, parseInt(v) || 10))}
          type="number"
          min={0}
          max={50}
          placeholder="10"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.http.verifySsl")} description={t("settings.http.verifySslDesc")}>
        <Toggle checked={config.http.verifySsl} onChange={(v) => updateField("http", "verifySsl", v)} />
      </SettingRow>
    </div>
  );
}

function AdvancedSettings({ config, updateField }: { config: AppConfig; updateField: UpdateFieldFn }) {
  const { t } = useI18n();

  return (
    <div>
      <h3 className="text-sm font-semibold text-text-primary mb-3">{t("settings.sections.advanced")}</h3>
      <SettingRow label={t("settings.advanced.logLevel")}>
        <select
          value={config.advanced.logLevel}
          onChange={(e) => updateField("advanced", "logLevel", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="debug">{t("settings.advanced.logLevelDebug")}</option>
          <option value="info">{t("settings.advanced.logLevelInfo")}</option>
          <option value="warn">{t("settings.advanced.logLevelWarn")}</option>
          <option value="error">{t("settings.advanced.logLevelError")}</option>
        </select>
      </SettingRow>
      <SettingRow label={t("settings.advanced.logMaxSize")}>
        <SettingInput
          value={config.advanced.logMaxSize}
          onChange={(v) => updateField("advanced", "logMaxSize", parseInt(v) || 10)}
          type="number"
          placeholder="10"
          suffix="MB"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.advanced.logRetainDays")}>
        <SettingInput
          value={config.advanced.logRetainDays}
          onChange={(v) => updateField("advanced", "logRetainDays", parseInt(v) || 7)}
          type="number"
          placeholder="7"
          suffix={t("format.days")}
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.advanced.dbBackupInterval")} description={t("settings.advanced.dbBackupIntervalDesc")}>
        <SettingInput
          value={config.advanced.dbBackupInterval}
          onChange={(v) => updateField("advanced", "dbBackupInterval", parseInt(v) || 24)}
          type="number"
          min={0}
          placeholder="24"
          suffix={t("format.hours")}
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.advanced.tempCleanupInterval")} description={t("settings.advanced.tempCleanupIntervalDesc")}>
        <SettingInput
          value={config.advanced.tempCleanupInterval}
          onChange={(v) => updateField("advanced", "tempCleanupInterval", parseInt(v) || 1)}
          type="number"
          min={0}
          placeholder="1"
          suffix={t("format.hours")}
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.advanced.memoryLimit")} description={t("settings.advanced.memoryLimitDesc")}>
        <SettingInput
          value={config.advanced.memoryLimit}
          onChange={(v) => updateField("advanced", "memoryLimit", parseInt(v) || 0)}
          type="number"
          min={0}
          placeholder="0"
          suffix="MB"
          className="w-16"
        />
      </SettingRow>
      <SettingRow label={t("settings.advanced.postAction")} description={t("settings.advanced.postActionDesc")}>
        <select
          value={config.advanced.postDownloadAction}
          onChange={(e) => updateField("advanced", "postDownloadAction", e.target.value)}
          className="px-3 py-1.5 rounded border border-border bg-secondary text-text-primary text-sm"
        >
          <option value="none">{t("settings.advanced.postActionNone")}</option>
          <option value="shutdown">{t("settings.advanced.postActionShutdown")}</option>
          <option value="hibernate">{t("settings.advanced.postActionHibernate")}</option>
          <option value="sleep">{t("settings.advanced.postActionSleep")}</option>
          <option value="run_command">{t("settings.advanced.postActionCommand")}</option>
        </select>
      </SettingRow>
      {config.advanced.postDownloadAction === "run_command" && (
        <SettingRow label={t("settings.advanced.postActionCommand")}>
          <SettingInput
            value={config.advanced.postDownloadCommand}
            onChange={(v) => updateField("advanced", "postDownloadCommand", v)}
            placeholder={t("settings.advanced.postCommandPlaceholder")}
            className="w-80"
          />
        </SettingRow>
      )}
    </div>
  );
}
