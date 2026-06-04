// 任务调度对话框
// 管理定时任务和带宽计划

import { useState, useEffect, useCallback } from "react";
import { X, Clock, Plus, Trash2, Edit2 } from "lucide-react";
import { showToast } from "./Toast";
import { extractErrorMessage } from "../lib/errors";
import { useI18n } from "../hooks/useI18n";
import {
  ScheduleRule,
  ScheduleRuleType,
  BandwidthSchedule,
  getScheduleRules,
  addScheduleRule,
  removeScheduleRule,
  updateScheduleRule,
  setScheduleRuleEnabled,
  getBandwidthSchedules,
  addBandwidthSchedule,
  removeBandwidthSchedule,
  updateBandwidthSchedule,
} from "../lib/tauri-api";


interface ScheduleDialogProps {
  open: boolean;
  onClose: () => void;
}

export { ScheduleDialog };

function ScheduleDialog({ open, onClose }: ScheduleDialogProps) {
  const [activeTab, setActiveTab] = useState<"rules" | "bandwidth">("rules");
  const [rules, setRules] = useState<ScheduleRule[]>([]);
  const [bandwidthSchedules, setBandwidthSchedules] = useState<BandwidthSchedule[]>([]);
  const [loading, setLoading] = useState(false);

  // 规则编辑状态
  const [editingRule, setEditingRule] = useState<ScheduleRule | null>(null);
  const [showRuleForm, setShowRuleForm] = useState(false);
  const [ruleForm, setRuleForm] = useState({
    name: "",
    ruleType: "StartTask" as ScheduleRuleType,
    cronExpression: "0 0 * * *",
    taskId: "",
    downloadSpeed: 0,
    uploadSpeed: 0,
  });

  // 带宽计划编辑状态
  const [editingBandwidth, setEditingBandwidth] = useState<number | null>(null);
  const [showBandwidthForm, setShowBandwidthForm] = useState(false);
  const [bandwidthForm, setBandwidthForm] = useState({
    startTime: "22:00",
    endTime: "06:00",
    downloadSpeed: 0,
    uploadSpeed: 0,
    weekdays: [] as number[],
  });

  const { t } = useI18n();

  // 常用 cron 表达式预设（使用 i18n 翻译标签）
  const CRON_PRESETS = [
    { label: t("schedule.cronPresets.daily0"), value: "0 0 * * *" },
    { label: t("schedule.cronPresets.daily8"), value: "0 8 * * *" },
    { label: t("schedule.cronPresets.daily22"), value: "0 22 * * *" },
    { label: t("schedule.cronPresets.weekday9"), value: "0 9 * * 1-5" },
    { label: t("schedule.cronPresets.weekday18"), value: "0 18 * * 1-5" },
    { label: t("schedule.cronPresets.hourly"), value: "0 * * * *" },
    { label: t("schedule.cronPresets.every6h"), value: "0 */6 * * *" },
    { label: t("schedule.cronPresets.weeklyMonday"), value: "0 0 * * 1" },
  ];

  // 星期几选项（使用 i18n 翻译标签）
  const WEEKDAYS = [
    { value: 1, label: t("schedule.days.mon") },
    { value: 2, label: t("schedule.days.tue") },
    { value: 3, label: t("schedule.days.wed") },
    { value: 4, label: t("schedule.days.thu") },
    { value: 5, label: t("schedule.days.fri") },
    { value: 6, label: t("schedule.days.sat") },
    { value: 7, label: t("schedule.days.sun") },
  ];

  // 加载数据
  const loadData = useCallback(async () => {
    setLoading(true);
    try {
      const [rulesData, bandwidthData] = await Promise.all([
        getScheduleRules(),
        getBandwidthSchedules(),
      ]);
      setRules(rulesData);
      setBandwidthSchedules(bandwidthData);
    } catch (error) {
      showToast(t("schedule.loadFailed", { error: extractErrorMessage(error) }), "error");
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    if (open) {
      loadData();
    }
  }, [open, loadData]);

  // 验证 cron 表达式格式（5 个字段：分 时 日 月 周）
  const isValidCron = (expr: string): boolean => {
    const parts = expr.trim().split(/\s+/);
    return parts.length === 5;
  };

  // 规则操作
  const handleAddRule = async () => {
    // 验证 cron 表达式
    if (!isValidCron(ruleForm.cronExpression)) {
      showToast(t("schedule.rule.addFailed", { error: "Invalid cron expression (expected 5 fields)" }), "error");
      return;
    }
    try {
      const rule: ScheduleRule = {
        id: `rule-${crypto.randomUUID()}`,
        name: ruleForm.name || t("schedule.rule.defaultName", { count: rules.length + 1 }),
        ruleType: ruleForm.ruleType,
        cronExpression: ruleForm.cronExpression,
        taskId: ruleForm.taskId || undefined,
        params: {
          downloadSpeed: ruleForm.downloadSpeed || undefined,
          uploadSpeed: ruleForm.uploadSpeed || undefined,
        },
        enabled: true,
        createdAt: new Date().toISOString(),
      };

      await addScheduleRule(rule);
      showToast(t("schedule.rule.added"), "success");
      setShowRuleForm(false);
      resetRuleForm();
      loadData();
    } catch (error) {
      showToast(t("schedule.rule.addFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const handleUpdateRule = async () => {
    if (!editingRule) return;

    // 验证 cron 表达式
    if (!isValidCron(ruleForm.cronExpression)) {
      showToast(t("schedule.rule.updateFailed", { error: "Invalid cron expression (expected 5 fields)" }), "error");
      return;
    }

    try {
      const rule: ScheduleRule = {
        ...editingRule,
        name: ruleForm.name || editingRule.name,
        ruleType: ruleForm.ruleType,
        cronExpression: ruleForm.cronExpression,
        taskId: ruleForm.taskId || undefined,
        params: {
          downloadSpeed: ruleForm.downloadSpeed || undefined,
          uploadSpeed: ruleForm.uploadSpeed || undefined,
        },
      };

      await updateScheduleRule(rule);
      showToast(t("schedule.rule.updated"), "success");
      setEditingRule(null);
      setShowRuleForm(false);
      resetRuleForm();
      loadData();
    } catch (error) {
      showToast(t("schedule.rule.updateFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const handleDeleteRule = async (ruleId: string) => {
    const { confirm } = await import("./ConfirmDialog");
    const ok = await confirm({
      title: t("schedule.rule.deleteTitle"),
      message: t("schedule.rule.deleteConfirm"),
      variant: "danger",
    });
    if (!ok) return;

    try {
      await removeScheduleRule(ruleId);
      showToast(t("schedule.rule.deleted"), "success");
      loadData();
    } catch (error) {
      showToast(t("schedule.rule.deleteFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const handleToggleRule = async (ruleId: string, enabled: boolean) => {
    try {
      await setScheduleRuleEnabled(ruleId, enabled);
      showToast(t("schedule.rule.toggled", { status: enabled ? t("schedule.rule.enabled") : t("schedule.rule.disabled") }), "success");
      loadData();
    } catch (error) {
      showToast(t("schedule.rule.toggleFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const startEditRule = (rule: ScheduleRule) => {
    setEditingRule(rule);
    setRuleForm({
      name: rule.name,
      ruleType: rule.ruleType,
      cronExpression: rule.cronExpression,
      taskId: rule.taskId || "",
      downloadSpeed: rule.params.downloadSpeed || 0,
      uploadSpeed: rule.params.uploadSpeed || 0,
    });
    setShowRuleForm(true);
  };

  const resetRuleForm = () => {
    setRuleForm({
      name: "",
      ruleType: "StartTask",
      cronExpression: "0 0 * * *",
      taskId: "",
      downloadSpeed: 0,
      uploadSpeed: 0,
    });
  };

  // 带宽计划操作
  const handleAddBandwidth = async () => {
    try {
      const schedule: BandwidthSchedule = {
        startTime: bandwidthForm.startTime,
        endTime: bandwidthForm.endTime,
        downloadSpeed: bandwidthForm.downloadSpeed,
        uploadSpeed: bandwidthForm.uploadSpeed,
        weekdays: bandwidthForm.weekdays.length > 0 ? bandwidthForm.weekdays : undefined,
      };

      await addBandwidthSchedule(schedule);
      showToast(t("schedule.bandwidth.added"), "success");
      setShowBandwidthForm(false);
      resetBandwidthForm();
      loadData();
    } catch (error) {
      showToast(t("schedule.bandwidth.addFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const handleUpdateBandwidth = async () => {
    if (editingBandwidth === null) return;

    try {
      const schedule: BandwidthSchedule = {
        startTime: bandwidthForm.startTime,
        endTime: bandwidthForm.endTime,
        downloadSpeed: bandwidthForm.downloadSpeed,
        uploadSpeed: bandwidthForm.uploadSpeed,
        weekdays: bandwidthForm.weekdays.length > 0 ? bandwidthForm.weekdays : undefined,
      };

      await updateBandwidthSchedule(editingBandwidth, schedule);
      showToast(t("schedule.bandwidth.updated"), "success");
      setEditingBandwidth(null);
      setShowBandwidthForm(false);
      resetBandwidthForm();
      loadData();
    } catch (error) {
      showToast(t("schedule.bandwidth.updateFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const handleDeleteBandwidth = async (index: number) => {
    const { confirm } = await import("./ConfirmDialog");
    const ok = await confirm({
      title: t("schedule.bandwidth.deleteTitle"),
      message: t("schedule.bandwidth.deleteConfirm"),
      variant: "danger",
    });
    if (!ok) return;

    try {
      await removeBandwidthSchedule(index);
      showToast(t("schedule.bandwidth.deleted"), "success");
      loadData();
    } catch (error) {
      showToast(t("schedule.bandwidth.deleteFailed", { error: extractErrorMessage(error) }), "error");
    }
  };

  const startEditBandwidth = (index: number, schedule: BandwidthSchedule) => {
    setEditingBandwidth(index);
    setBandwidthForm({
      startTime: schedule.startTime,
      endTime: schedule.endTime,
      downloadSpeed: schedule.downloadSpeed,
      uploadSpeed: schedule.uploadSpeed,
      weekdays: schedule.weekdays || [],
    });
    setShowBandwidthForm(true);
  };

  const resetBandwidthForm = () => {
    setBandwidthForm({
      startTime: "22:00",
      endTime: "06:00",
      downloadSpeed: 0,
      uploadSpeed: 0,
      weekdays: [],
    });
  };

  const toggleWeekday = (day: number) => {
    setBandwidthForm(prev => ({
      ...prev,
      weekdays: prev.weekdays.includes(day)
        ? prev.weekdays.filter(d => d !== day)
        : [...prev.weekdays, day].sort(),
    }));
  };

  // 格式化速度显示
  const formatSpeed = (bytesPerSec: number): string => {
    if (bytesPerSec === 0) return t("schedule.unlimited");
    if (bytesPerSec < 1024) return `${bytesPerSec} B/s`;
    if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
    return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
  };

  // 获取规则类型标签
  const getRuleTypeLabel = (type: ScheduleRuleType): string => {
    switch (type) {
      case "StartTask": return t("schedule.ruleTypes.startTask");
      case "PauseTask": return t("schedule.ruleTypes.pauseTask");
      case "BandwidthPlan": return t("schedule.ruleTypes.bandwidthPlan");
      case "SeedPlan": return t("schedule.ruleTypes.seedPlan");
      default: return type;
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div className="bg-primary rounded-lg shadow-xl w-[90vw] max-w-[700px] max-h-[80vh] flex flex-col" role="dialog" aria-modal="true" onClick={(e) => e.stopPropagation()}>
        {/* 头部 */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-2">
            <Clock className="w-5 h-5 text-accent" />
            <h2 className="text-lg font-semibold text-text-primary">{t("schedule.title")}</h2>
          </div>
          <button
            onClick={onClose}
            aria-label="Close"
            className="p-1 rounded hover:bg-tertiary text-text-muted hover:text-text-primary"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* 标签页 */}
        <div className="flex border-b border-border">
          <button
            className={`px-4 py-2 text-sm font-medium ${
              activeTab === "rules"
                ? "text-accent border-b-2 border-accent"
                : "text-text-secondary hover:text-text-primary"
            }`}
            onClick={() => setActiveTab("rules")}
          >
            {t("schedule.tabRules")}
          </button>
          <button
            className={`px-4 py-2 text-sm font-medium ${
              activeTab === "bandwidth"
                ? "text-accent border-b-2 border-accent"
                : "text-text-secondary hover:text-text-primary"
            }`}
            onClick={() => setActiveTab("bandwidth")}
          >
            {t("schedule.tabBandwidth")}
          </button>
        </div>

        {/* 内容区域 */}
        <div className="flex-1 overflow-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center h-32">
              <div className="text-text-muted">{t("common.loading")}</div>
            </div>
          ) : (
            <>
              {/* 调度规则标签页 */}
              {activeTab === "rules" && (
                <div>
                  {/* 添加规则按钮 */}
                  <div className="mb-4">
                    <button
                      onClick={() => {
                        resetRuleForm();
                        setEditingRule(null);
                        setShowRuleForm(true);
                      }}
                      className="flex items-center gap-2 px-3 py-2 bg-accent text-white rounded hover:bg-accent-hover"
                    >
                      <Plus className="w-4 h-4" />
                      {t("schedule.rule.add")}
                    </button>
                  </div>

                  {/* 规则表单 */}
                  {showRuleForm && (
                    <div className="mb-4 p-4 bg-secondary rounded-lg border border-border">
                      <h3 className="text-sm font-medium text-text-primary mb-3">
                        {editingRule ? t("schedule.rule.edit") : t("schedule.rule.add")}
                      </h3>
                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.rule.name")}</label>
                          <input
                            type="text"
                            value={ruleForm.name}
                            onChange={e => setRuleForm(prev => ({ ...prev, name: e.target.value }))}
                            placeholder={t("schedule.rule.namePlaceholder")}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.rule.type")}</label>
                          <select
                            value={ruleForm.ruleType}
                            onChange={e => setRuleForm(prev => ({ ...prev, ruleType: e.target.value as ScheduleRuleType }))}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          >
                            <option value="StartTask">{t("schedule.ruleTypes.startTask")}</option>
                            <option value="PauseTask">{t("schedule.ruleTypes.pauseTask")}</option>
                            <option value="BandwidthPlan">{t("schedule.ruleTypes.bandwidthPlan")}</option>
                            <option value="SeedPlan">{t("schedule.ruleTypes.seedPlan")}</option>
                          </select>
                        </div>
                        <div className="col-span-2">
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.rule.cronLabel")}</label>
                          <div className="flex gap-2">
                            <input
                              type="text"
                              value={ruleForm.cronExpression}
                              onChange={e => setRuleForm(prev => ({ ...prev, cronExpression: e.target.value }))}
                              placeholder="* * * * *"
                              className="flex-1 px-2 py-1 text-sm font-mono bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                            />
                            <select
                              onChange={e => {
                                if (e.target.value) {
                                  setRuleForm(prev => ({ ...prev, cronExpression: e.target.value }));
                                }
                              }}
                              className="px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                            >
                              <option value="">{t("schedule.rule.preset")}</option>
                              {CRON_PRESETS.map(preset => (
                                <option key={preset.value} value={preset.value}>
                                  {preset.label}
                                </option>
                              ))}
                            </select>
                          </div>
                          <p className="text-xs text-text-muted mt-1">
                            {t("schedule.rule.cronHint")}
                          </p>
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.rule.downloadSpeed")}</label>
                          <input
                            type="number"
                            value={ruleForm.downloadSpeed}
                            onChange={e => setRuleForm(prev => ({ ...prev, downloadSpeed: Number(e.target.value) }))}
                            placeholder={t("schedule.rule.speedPlaceholder")}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.rule.uploadSpeed")}</label>
                          <input
                            type="number"
                            value={ruleForm.uploadSpeed}
                            onChange={e => setRuleForm(prev => ({ ...prev, uploadSpeed: Number(e.target.value) }))}
                            placeholder={t("schedule.rule.speedPlaceholder")}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                      </div>
                      <div className="flex justify-end gap-2 mt-3">
                        <button
                          onClick={() => {
                            setShowRuleForm(false);
                            setEditingRule(null);
                          }}
                          className="px-3 py-1 text-sm text-text-secondary hover:text-text-primary"
                        >
                          {t("common.cancel")}
                        </button>
                        <button
                          onClick={editingRule ? handleUpdateRule : handleAddRule}
                          className="px-3 py-1 text-sm bg-accent text-white rounded hover:bg-accent-hover"
                        >
                          {editingRule ? t("common.edit") : t("archive.add")}
                        </button>
                      </div>
                    </div>
                  )}

                  {/* 规则列表 */}
                  {rules.length === 0 ? (
                    <div className="text-center text-text-muted py-8">
                      <Clock className="w-12 h-12 mx-auto mb-2 opacity-50" />
                      <p>{t("schedule.rule.empty")}</p>
                      <p className="text-sm">{t("schedule.rule.emptyHint")}</p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {rules.map(rule => (
                        <div
                          key={rule.id}
                          className="flex items-center justify-between p-3 bg-secondary rounded-lg border border-border"
                        >
                          <div className="flex items-center gap-3">
                            <input
                              type="checkbox"
                              checked={rule.enabled}
                              onChange={() => handleToggleRule(rule.id, !rule.enabled)}
                              className="w-4 h-4 accent-accent"
                            />
                            <div>
                              <div className="text-sm font-medium text-text-primary">
                                {rule.name}
                              </div>
                              <div className="text-xs text-text-muted">
                                <span className="inline-block px-1.5 py-0.5 bg-tertiary rounded mr-2">
                                  {getRuleTypeLabel(rule.ruleType)}
                                </span>
                                <span className="font-mono">{rule.cronExpression}</span>
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <button
                              onClick={() => startEditRule(rule)}
                              className="p-1 rounded hover:bg-tertiary text-text-muted hover:text-text-primary"
                            >
                              <Edit2 className="w-4 h-4" />
                            </button>
                            <button
                              onClick={() => handleDeleteRule(rule.id)}
                              className="p-1 rounded hover:bg-tertiary text-text-muted hover:text-error"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}

              {/* 带宽计划标签页 */}
              {activeTab === "bandwidth" && (
                <div>
                  {/* 添加带宽计划按钮 */}
                  <div className="mb-4">
                    <button
                      onClick={() => {
                        resetBandwidthForm();
                        setEditingBandwidth(null);
                        setShowBandwidthForm(true);
                      }}
                      className="flex items-center gap-2 px-3 py-2 bg-accent text-white rounded hover:bg-accent-hover"
                    >
                      <Plus className="w-4 h-4" />
                      {t("schedule.bandwidth.add")}
                    </button>
                  </div>

                  {/* 带宽计划表单 */}
                  {showBandwidthForm && (
                    <div className="mb-4 p-4 bg-secondary rounded-lg border border-border">
                      <h3 className="text-sm font-medium text-text-primary mb-3">
                        {editingBandwidth !== null ? t("schedule.bandwidth.edit") : t("schedule.bandwidth.add")}
                      </h3>
                      <div className="grid grid-cols-2 gap-3">
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.bandwidth.startTime")}</label>
                          <input
                            type="time"
                            value={bandwidthForm.startTime}
                            onChange={e => setBandwidthForm(prev => ({ ...prev, startTime: e.target.value }))}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.bandwidth.endTime")}</label>
                          <input
                            type="time"
                            value={bandwidthForm.endTime}
                            onChange={e => setBandwidthForm(prev => ({ ...prev, endTime: e.target.value }))}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.bandwidth.downloadSpeed")}</label>
                          <input
                            type="number"
                            value={bandwidthForm.downloadSpeed}
                            onChange={e => setBandwidthForm(prev => ({ ...prev, downloadSpeed: Number(e.target.value) }))}
                            placeholder={t("schedule.bandwidth.speedPlaceholder")}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div>
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.bandwidth.uploadSpeed")}</label>
                          <input
                            type="number"
                            value={bandwidthForm.uploadSpeed}
                            onChange={e => setBandwidthForm(prev => ({ ...prev, uploadSpeed: Number(e.target.value) }))}
                            placeholder={t("schedule.bandwidth.speedPlaceholder")}
                            className="w-full px-2 py-1 text-sm bg-tertiary border border-border rounded focus:outline-none focus:ring-1 focus:ring-accent/50"
                          />
                        </div>
                        <div className="col-span-2">
                          <label className="block text-xs text-text-muted mb-1">{t("schedule.bandwidth.weekdays")}</label>
                          <div className="flex gap-1">
                            {WEEKDAYS.map(day => (
                              <button
                                key={day.value}
                                onClick={() => toggleWeekday(day.value)}
                                className={`px-2 py-1 text-xs rounded ${
                                  bandwidthForm.weekdays.includes(day.value)
                                    ? "bg-accent text-white"
                                    : "bg-tertiary text-text-secondary hover:bg-secondary"
                                }`}
                              >
                                {day.label}
                              </button>
                            ))}
                          </div>
                          <p className="text-xs text-text-muted mt-1">
                            {t("schedule.bandwidth.weekdaysHint")}
                          </p>
                        </div>
                      </div>
                      <div className="flex justify-end gap-2 mt-3">
                        <button
                          onClick={() => {
                            setShowBandwidthForm(false);
                            setEditingBandwidth(null);
                          }}
                          className="px-3 py-1 text-sm text-text-secondary hover:text-text-primary"
                        >
                          {t("common.cancel")}
                        </button>
                        <button
                          onClick={editingBandwidth !== null ? handleUpdateBandwidth : handleAddBandwidth}
                          className="px-3 py-1 text-sm bg-accent text-white rounded hover:bg-accent-hover"
                        >
                          {editingBandwidth !== null ? t("common.edit") : t("archive.add")}
                        </button>
                      </div>
                    </div>
                  )}

                  {/* 带宽计划列表 */}
                  {bandwidthSchedules.length === 0 ? (
                    <div className="text-center text-text-muted py-8">
                      <Clock className="w-12 h-12 mx-auto mb-2 opacity-50" />
                      <p>{t("schedule.bandwidth.empty")}</p>
                      <p className="text-sm">{t("schedule.bandwidth.emptyHint")}</p>
                    </div>
                  ) : (
                    <div className="space-y-2">
                      {bandwidthSchedules.map((schedule, index) => (
                        <div
                          key={index}
                          className="flex items-center justify-between p-3 bg-secondary rounded-lg border border-border"
                        >
                          <div>
                            <div className="text-sm font-medium text-text-primary">
                              {schedule.startTime} - {schedule.endTime}
                            </div>
                            <div className="text-xs text-text-muted">
                              {t("schedule.bandwidth.downloadLabel")}: {formatSpeed(schedule.downloadSpeed)} | {t("schedule.bandwidth.uploadLabel")}: {formatSpeed(schedule.uploadSpeed)}
                              {schedule.weekdays && schedule.weekdays.length > 0 && (
                                <span className="ml-2">
                                  ({schedule.weekdays.map(d => WEEKDAYS.find(w => w.value === d)?.label).join(", ")})
                                </span>
                              )}
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <button
                              onClick={() => startEditBandwidth(index, schedule)}
                              className="p-1 rounded hover:bg-tertiary text-text-muted hover:text-text-primary"
                            >
                              <Edit2 className="w-4 h-4" />
                            </button>
                            <button
                              onClick={() => handleDeleteBandwidth(index)}
                              className="p-1 rounded hover:bg-tertiary text-text-muted hover:text-error"
                            >
                              <Trash2 className="w-4 h-4" />
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              )}
            </>
          )}
        </div>

        {/* 底部 */}
        <div className="flex justify-end p-4 border-t border-border">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-text-secondary hover:text-text-primary"
          >
            {t("common.close")}
          </button>
        </div>
      </div>
    </div>
  );
}
