// 底部状态栏组件
// 显示：全局下载/上传速度、活跃任务数、DHT 状态、版本号

import { useState, useEffect, useMemo } from "react";
import { Download, Upload, Activity, Wifi, Globe } from "lucide-react";
import { useTaskStore } from "../stores/taskStore";
import { formatSpeed } from "../lib/format";
import { getBtStatus, getAppInfo, getApiStatus, getKadStatus } from "../lib/tauri-api";
import { useI18n } from "../hooks/useI18n";

export function StatusBar() {
  const { t } = useI18n();
  const tasks = useTaskStore((s) => s.tasks);
  // 使用 useMemo 计算统计，避免每次渲染都调用 getGlobalStats
  const stats = useMemo(() => {
    let totalSpeed = 0;
    let totalUploadSpeed = 0;
    let activeCount = 0;
    let totalDownloaded = 0;
    let totalSize = 0;
    tasks.forEach((t) => {
      if (t.state === "downloading" || t.state === "seeding") {
        totalSpeed += t.downloadSpeed;
        totalUploadSpeed += t.uploadSpeed;
        activeCount++;
      }
      // 计算全局进度（含已完成任务，已完成 = 100%）
      if (t.totalSize > 0) {
        totalDownloaded += t.downloaded;
        totalSize += t.totalSize;
      }
    });
    const globalProgress = totalSize > 0 ? Math.min(totalDownloaded / totalSize, 1) : 0;
    return { totalSpeed, totalUploadSpeed, activeCount, totalCount: tasks.size, globalProgress, hasProgress: totalSize > 0 };
  }, [tasks]);
  const [dhtNodes, setDhtNodes] = useState(0);
  const [dhtConnected, setDhtConnected] = useState(false);
  const [kadNodes, setKadNodes] = useState(0);
  const [kadRunning, setKadRunning] = useState(false);
  const [kadBootstrapDone, setKadBootstrapDone] = useState(false);
  const [version, setVersion] = useState("v1.0.0");
  const [apiPort, setApiPort] = useState(0);
  const [wsConnections, setWsConnections] = useState(0);

  // 获取应用版本和 API 状态
  useEffect(() => {
    getAppInfo()
      .then((info) => setVersion(`v${info.version}`))
      .catch(() => {});
    getApiStatus()
      .then((s) => { setApiPort(s.port); setWsConnections(s.wsConnections); })
      .catch(() => {});
  }, []);

  useEffect(() => {
    let mounted = true;

    const fetchBtStatus = async () => {
      try {
        const btStatus = await getBtStatus();
        if (mounted) {
          setDhtNodes(btStatus.dhtNodes);
          setDhtConnected(btStatus.dhtConnected);
        }
      } catch {
        // BT 引擎可能未初始化，忽略错误
      }
    };

    fetchBtStatus();
    const timer = setInterval(fetchBtStatus, 5000);

    // KAD 状态（与 BT 同步轮询）
    const fetchKadStatus = async () => {
      try {
        const kad = await getKadStatus();
        if (mounted && kad) {
          setKadRunning(kad.running);
          setKadNodes(kad.nodeCount);
          setKadBootstrapDone(kad.bootstrapDone);
        }
      } catch {
        // ed2k 引擎可能未初始化
      }
    };
    fetchKadStatus();
    const kadTimer = setInterval(fetchKadStatus, 10000);

    return () => {
      mounted = false;
      clearInterval(timer);
      clearInterval(kadTimer);
    };
  }, []);

  return (
    <div className="flex items-center gap-2 sm:gap-4 px-2 sm:px-4 py-1.5 bg-secondary border-t border-border text-xs text-text-secondary overflow-x-auto">
      {/* 全局下载速度 */}
      <div className="flex items-center gap-1 shrink-0">
        <Download size={12} className="text-accent" />
        <span className="font-mono">{formatSpeed(stats.totalSpeed)}</span>
      </div>

      {/* 全局上传速度 */}
      <div className="flex items-center gap-1 shrink-0">
        <Upload size={12} className="text-success" />
        <span className="font-mono">{formatSpeed(stats.totalUploadSpeed)}</span>
      </div>

      {/* 分隔线 */}
      <div className="w-px h-3 bg-border shrink-0" />

      {/* 活跃任务数 */}
      <div className="flex items-center gap-1 shrink-0">
        <Activity size={12} className="text-accent" />
        <span>{t("statusBar.activeTasks")}: {stats.activeCount}</span>
      </div>

      {/* 任务总数 */}
      <span className="shrink-0">{t("statusBar.totalTasks")}: {stats.totalCount}</span>

      {/* 全局下载进度条 */}
      {stats.hasProgress && (
        <div className="flex items-center gap-1.5 shrink-0">
          <div className="global-progress w-24">
            <div
              className="global-progress-fill"
              style={{ width: `${(stats.globalProgress * 100).toFixed(0)}%` }}
            />
          </div>
          <span className="text-[10px] font-mono text-text-muted">
            {(stats.globalProgress * 100).toFixed(0)}%
          </span>
        </div>
      )}

      {/* 占位 */}
      <div className="flex-1 min-w-0" />

      {/* API 状态 */}
      {apiPort > 0 && (
        <div className="hidden sm:flex items-center gap-1 shrink-0">
          <Globe size={12} className="text-accent" />
          <span>API: :{apiPort}{wsConnections > 0 ? ` (${wsConnections} ws)` : ""}</span>
        </div>
      )}

      {/* DHT 状态 */}
      <div className="hidden md:flex items-center gap-1 shrink-0">
        <Wifi
          size={12}
          className={dhtConnected ? "text-success" : "text-text-muted"}
        />
        <span>
          DHT: {dhtConnected ? `${dhtNodes} ${t("statusBar.dhtNodes")}` : t("statusBar.dhtDisconnected")}
        </span>
      </div>

      {/* KAD 状态 */}
      {kadRunning && (
        <div className="hidden md:flex items-center gap-1 shrink-0">
          <Wifi
            size={12}
            className={kadBootstrapDone && kadNodes > 0 ? "text-success" : "text-warning"}
          />
          <span>
            KAD: {kadBootstrapDone ? `${kadNodes} ${t("statusBar.kadNodes")}` : t("statusBar.kadBootstrap")}
          </span>
        </div>
      )}

      {/* 版本号 */}
      <span className="hidden lg:inline text-text-muted shrink-0">{version}</span>
    </div>
  );
}
