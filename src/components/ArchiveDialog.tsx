// 自动解压设置对话框
// 管理解压密码、自动解压配置

import { useState, useCallback, useEffect } from "react";
import { X, Archive, Plus, Trash2, Key } from "lucide-react";
import { Toggle } from "./Toggle";
import { extractErrorMessage } from "../lib/errors";
import {
  getArchiveConfig,
  updateArchiveConfig,
  type ArchiveConfigInfo,
  type PasswordInfo,
} from "../lib/tauri-api";
import { showToast } from "./Toast";
import { useI18n } from "../hooks/useI18n";

interface ArchiveDialogProps {
  onClose: () => void;
}

export function ArchiveDialog({ onClose }: ArchiveDialogProps) {
  const [autoExtract, setAutoExtract] = useState(false);
  const [deleteAfter, setDeleteAfter] = useState(false);
  const [extractDir, setExtractDir] = useState("");
  const [passwords, setPasswords] = useState<PasswordInfo[]>([]);
  const [showAddPassword, setShowAddPassword] = useState(false);
  const [newName, setNewName] = useState("");
  const [newPassword, setNewPassword] = useState("");
  const [newPattern, setNewPattern] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  const { t } = useI18n();

  // 加载配置
  useEffect(() => {
    const load = async () => {
      try {
        const config = await getArchiveConfig();
        setAutoExtract(config.autoExtract);
        setDeleteAfter(config.deleteAfterExtract);
        setExtractDir(config.extractDir);
        setPasswords(config.passwords);
      } catch (e) {
        showToast(t("archive.loadFailed", { error: extractErrorMessage(e) }), "error");
      } finally {
        setLoading(false);
      }
    };
    load();
  }, []);

  // 添加密码
  const handleAddPassword = useCallback(() => {
    if (!newPassword.trim()) return;
    setPasswords((prev) => [
      ...prev,
      {
        id: Date.now().toString(),
        name: newName.trim() || t("archive.defaultName"),
        password: newPassword.trim(),
        pattern: newPattern.trim(),
        enabled: true,
      },
    ]);
    setNewName("");
    setNewPassword("");
    setNewPattern("");
    setShowAddPassword(false);
  }, [newName, newPassword, newPattern]);

  // 删除密码
  const handleDeletePassword = useCallback((id: string) => {
    setPasswords((prev) => prev.filter((p) => p.id !== id));
  }, []);

  // 切换密码启用状态
  const handleTogglePassword = useCallback((id: string) => {
    setPasswords((prev) =>
      prev.map((p) => (p.id === id ? { ...p, enabled: !p.enabled } : p))
    );
  }, []);

  // 保存设置
  const handleSave = useCallback(async () => {
    setSaving(true);
    try {
      const config: ArchiveConfigInfo = {
        autoExtract,
        deleteAfterExtract: deleteAfter,
        extractDir,
        passwords,
      };
      await updateArchiveConfig(config);
      onClose();
    } catch (e) {
      showToast(t("archive.saveFailed", { error: extractErrorMessage(e) }), "error");
    } finally {
      setSaving(false);
    }
  }, [autoExtract, deleteAfter, extractDir, passwords, onClose, t]);

  if (loading) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
        <div className="bg-primary rounded-lg shadow-xl p-8 animate-fade-in" role="dialog" aria-modal="true">
          <span className="text-text-muted">{t("common.loading")}</span>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div
        className="bg-primary rounded-lg shadow-xl w-full max-w-lg mx-4 animate-fade-in"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        {/* 标题栏 */}
        <div className="flex items-center justify-between px-5 py-3 border-b border-border">
          <h2 className="text-lg font-semibold text-text-primary flex items-center gap-2">
            <Archive size={20} />
            {t("archive.settingsTitle")}
          </h2>
          <button onClick={onClose} aria-label="Close" className="p-1 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted">
            <X size={18} />
          </button>
        </div>

        {/* 内容 */}
        <div className="px-5 py-4 space-y-4">
          {/* 基本设置 */}
          <div className="space-y-3">
            <label className="flex items-center justify-between">
              <span className="text-sm text-text-primary">{t("archive.autoExtract")}</span>
              <Toggle checked={autoExtract} onChange={setAutoExtract} />
            </label>

            <label className="flex items-center justify-between">
              <span className="text-sm text-text-primary">{t("archive.deleteAfter")}</span>
              <Toggle checked={deleteAfter} onChange={setDeleteAfter} />
            </label>

            <div>
              <label className="text-sm text-text-secondary mb-1 block">{t("archive.extractDir")}</label>
              <input
                type="text"
                value={extractDir}
                onChange={(e) => setExtractDir(e.target.value)}
                placeholder={t("archive.extractDirPlaceholder")}
                className="w-full px-3 py-2 rounded border border-border bg-secondary text-text-primary text-sm focus:outline-none focus:ring-1 focus:ring-accent/50"
              />
            </div>
          </div>

          {/* 支持格式 */}
          <div className="bg-tertiary rounded-lg p-3">
            <div className="text-xs text-text-muted mb-2">{t("archive.supportedFormats")}</div>
            <div className="flex flex-wrap gap-1.5">
              {[".zip", ".tar", ".tar.gz", ".tar.bz2", ".tar.xz"].map((fmt) => (
                <span
                  key={fmt}
                  className="px-2 py-0.5 rounded bg-secondary text-xs text-text-secondary"
                >
                  {fmt}
                </span>
              ))}
            </div>
          </div>

          {/* 密码管理 */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <span className="text-sm font-medium text-text-primary flex items-center gap-1.5">
                <Key size={14} />
                {t("archive.passwords")}
              </span>
              <button
                onClick={() => setShowAddPassword(!showAddPassword)}
                className="text-xs text-accent hover:underline flex items-center gap-1"
              >
                <Plus size={12} />
                {t("archive.addPassword")}
              </button>
            </div>

            {/* 添加密码表单 */}
            {showAddPassword && (
              <div className="mb-3 p-3 bg-tertiary rounded-lg space-y-2">
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder={t("archive.namePlaceholder")}
                  className="w-full px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
                />
                <input
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder={t("archive.passwordPlaceholder")}
                  className="w-full px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
                />
                <input
                  type="text"
                  value={newPattern}
                  onChange={(e) => setNewPattern(e.target.value)}
                  placeholder={t("archive.patternPlaceholder")}
                  className="w-full px-3 py-1.5 rounded border border-border bg-primary text-text-primary text-sm"
                />
                <div className="flex justify-end">
                  <button
                    onClick={handleAddPassword}
                    className="px-3 py-1.5 rounded bg-accent text-white text-sm hover:bg-accent-hover"
                  >
                    {t("archive.add")}
                  </button>
                </div>
              </div>
            )}

            {/* 密码列表 */}
            {passwords.length === 0 ? (
              <div className="text-xs text-text-muted text-center py-4 bg-tertiary rounded">
                {t("archive.noPasswords")}
              </div>
            ) : (
              <div className="space-y-1">
                {passwords.map((pwd) => (
                  <div
                    key={pwd.id}
                    className="flex items-center gap-2 px-3 py-2 rounded bg-tertiary text-sm"
                  >
                    <Key size={12} className="text-text-muted flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <span className="text-text-primary">{pwd.name}</span>
                      {pwd.pattern && (
                        <span className="text-xs text-text-muted ml-2">({pwd.pattern})</span>
                      )}
                    </div>
                    <span className="text-text-muted text-xs">{"*".repeat(pwd.password.length)}</span>
                    <button
                      onClick={() => handleTogglePassword(pwd.id)}
                      className={`text-xs px-1.5 py-0.5 rounded ${
                        pwd.enabled
                          ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                          : "bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400"
                      }`}
                    >
                      {pwd.enabled ? t("archive.enabled") : t("archive.disabled")}
                    </button>
                    <button
                      onClick={() => handleDeletePassword(pwd.id)}
                      className="p-1 rounded hover:bg-error/10 text-error"
                    >
                      <Trash2 size={12} />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
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
            onClick={handleSave}
            disabled={saving}
            className="px-4 py-1.5 rounded-md text-sm text-white bg-accent hover:bg-accent-hover transition-colors disabled:opacity-50"
          >
            {saving ? t("archive.saving") : t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
