// 速度图表组件
// Canvas 渲染，60fps 实时更新，替代 Recharts SVG 方案
// 支持 5 分钟 / 30 分钟 / 1 小时时间轴切换

import { useRef, useEffect, useState, useCallback } from "react";
import { useTaskStore } from "../stores/taskStore";
import { formatSpeed } from "../lib/format";
import { useI18n } from "../hooks/useI18n";

/** 时间范围秒数 */
const TIME_RANGE_SECONDS = [5 * 60, 30 * 60, 60 * 60] as const;

/** 图表内边距 */
const PADDING = { top: 12, right: 12, bottom: 28, left: 64 };

/** 从 CSS 变量读取颜色值 */
function getCssVar(name: string, fallback: string): string {
  const val = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return val || fallback;
}

/** 颜色定义（从 CSS 变量动态读取） */
function getColors() {
  const dark = isDark();
  return {
    grid: getCssVar("--border", dark ? "#334155" : "#e2e8f0"),
    download: getCssVar("--accent", "#3b82f6"),
    upload: getCssVar("--success", "#22c55e"),
    text: getCssVar("--text-muted", "#94a3b8"),
    bg: getCssVar("--bg-primary", dark ? "#1e293b" : "#f8fafc"),
    tooltipBg: getCssVar("--bg-secondary", dark ? "#1e293b" : "#ffffff"),
    tooltipBorder: getCssVar("--border", "#e2e8f0"),
  };
}

function isDark(): boolean {
  const theme = document.documentElement.getAttribute("data-theme");
  if (theme === "dark") return true;
  if (theme === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/** 格式化时间轴标签（导出供测试） */
export function formatTimeLabel(ts: number): string {
  const d = new Date(ts);
  const h = d.getHours().toString().padStart(2, "0");
  const m = d.getMinutes().toString().padStart(2, "0");
  const s = d.getSeconds().toString().padStart(2, "0");
  return `${h}:${m}:${s}`;
}

/** 计算合适的 Y 轴刻度（导出供测试） */
export function computeYTicks(maxVal: number): number[] {
  if (maxVal <= 0) return [0];
  const steps = [1024, 2048, 5120, 10240, 25600, 51200, 102400, 256000, 512000, 1048576, 2621440, 5242880, 10485760, 26214400, 52428800, 104857600];
  const step = steps.find((s) => maxVal / s <= 6) ?? steps[steps.length - 1];
  const ticks: number[] = [];
  for (let v = 0; v <= maxVal + step; v += step) {
    ticks.push(v);
  }
  return ticks;
}

export function SpeedChart() {
  const { t } = useI18n();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);
  const rafRef = useRef<number>(0);
  const [rangeIdx, setRangeIdx] = useState(0);
  const hoverXRef = useRef<number | null>(null);
  const rangeSeconds = TIME_RANGE_SECONDS[rangeIdx];
  // 颜色缓存（仅主题变化时刷新）
  const colorsRef = useRef<ReturnType<typeof getColors>>(getColors());
  const lastThemeRef = useRef<boolean>(isDark());
  // Canvas 尺寸缓存（避免每帧重新分配缓冲区）
  const canvasSizeRef = useRef<{ w: number; h: number; dpr: number }>({ w: 0, h: 0, dpr: 0 });

  const TIME_RANGES = [
    { label: t("speedChart.5min"), seconds: TIME_RANGE_SECONDS[0] },
    { label: t("speedChart.30min"), seconds: TIME_RANGE_SECONDS[1] },
    { label: t("speedChart.1hour"), seconds: TIME_RANGE_SECONDS[2] },
  ];

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    const container = containerRef.current;
    if (!canvas || !container) return;

    const rect = container.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    const w = rect.width;
    const h = rect.height;

    // 仅在尺寸变化时重新分配缓冲区
    const prev = canvasSizeRef.current;
    if (w !== prev.w || h !== prev.h || dpr !== prev.dpr) {
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
      canvasSizeRef.current = { w, h, dpr };
    }

    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    // 重置变换矩阵后再缩放（防止每帧复合缩放导致图表变形）
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.scale(dpr, dpr);

    // 颜色仅在主题变化时刷新（使用 isDark() 检测实际暗色状态，支持系统主题切换）
    const currentDark = isDark();
    if (currentDark !== lastThemeRef.current) {
      colorsRef.current = getColors();
      lastThemeRef.current = currentDark;
    }
    const colors = colorsRef.current;
    const bgColor = colors.bg;
    const gridColor = colors.grid;
    const textColor = colors.text;

    // 背景
    ctx.fillStyle = bgColor;
    ctx.fillRect(0, 0, w, h);

    const chartW = w - PADDING.left - PADDING.right;
    const chartH = h - PADDING.top - PADDING.bottom;
    if (chartW <= 0 || chartH <= 0) return;

    // 获取数据
    const speedHistory = useTaskStore.getState().speedHistory;
    const cutoff = Date.now() - rangeSeconds * 1000;
    const data = speedHistory.filter((p) => p.time >= cutoff);
    if (data.length < 2) {
      ctx.fillStyle = textColor;
      ctx.font = "13px system-ui, sans-serif";
      ctx.textAlign = "center";
      ctx.fillText(t("speedChart.waiting"), w / 2, h / 2);
      return;
    }

    const timeMin = data[0].time;
    const timeMax = data[data.length - 1].time;
    const timeSpan = Math.max(timeMax - timeMin, 1);

    const maxSpeed = Math.max(
      ...data.map((p) => Math.max(p.download, p.upload)),
      1024
    );
    const yTicks = computeYTicks(maxSpeed);
    const yMax = yTicks[yTicks.length - 1] || maxSpeed;

    // 坐标映射
    const toX = (t: number) => PADDING.left + ((t - timeMin) / timeSpan) * chartW;
    const toY = (v: number) => PADDING.top + chartH - (v / yMax) * chartH;

    // 网格线 + Y 轴标签
    ctx.strokeStyle = gridColor;
    ctx.lineWidth = 0.5;
    ctx.fillStyle = textColor;
    ctx.font = "10px system-ui, sans-serif";
    ctx.textAlign = "right";
    for (const tick of yTicks) {
      const y = toY(tick);
      ctx.beginPath();
      ctx.moveTo(PADDING.left, y);
      ctx.lineTo(w - PADDING.right, y);
      ctx.stroke();
      ctx.fillText(formatSpeed(tick), PADDING.left - 6, y + 3);
    }

    // X 轴标签
    ctx.textAlign = "center";
    const xLabelCount = Math.min(data.length, Math.floor(chartW / 80));
    const xStep = Math.max(1, Math.floor(data.length / xLabelCount));
    for (let i = 0; i < data.length; i += xStep) {
      const x = toX(data[i].time);
      ctx.fillText(formatTimeLabel(data[i].time), x, h - 4);
    }

    // 绘制线条
    function drawLine(c: CanvasRenderingContext2D, values: { x: number; y: number }[], color: string) {
      if (values.length < 2) return;
      c.beginPath();
      c.strokeStyle = color;
      c.lineWidth = 1.5;
      c.lineJoin = "round";
      c.moveTo(values[0].x, values[0].y);
      for (let i = 1; i < values.length; i++) {
        c.lineTo(values[i].x, values[i].y);
      }
      c.stroke();
    }

    const dlPoints = data.map((p) => ({ x: toX(p.time), y: toY(p.download) }));
    const ulPoints = data.map((p) => ({ x: toX(p.time), y: toY(p.upload) }));

    drawLine(ctx, dlPoints, colors.download);
    drawLine(ctx, ulPoints, colors.upload);

    // 十字线 + tooltip
    const hoverX = hoverXRef.current;
    if (hoverX !== null && hoverX >= PADDING.left && hoverX <= w - PADDING.right) {
      // 找最近的数据点
      const timeAtCursor = timeMin + ((hoverX - PADDING.left) / chartW) * timeSpan;
      let closest = data[0];
      let minDist = Infinity;
      for (const p of data) {
        const d = Math.abs(p.time - timeAtCursor);
        if (d < minDist) {
          minDist = d;
          closest = p;
        }
      }

      const cx = toX(closest.time);

      // 垂直线
      ctx.strokeStyle = textColor;
      ctx.lineWidth = 0.5;
      ctx.setLineDash([4, 4]);
      ctx.beginPath();
      ctx.moveTo(cx, PADDING.top);
      ctx.lineTo(cx, PADDING.top + chartH);
      ctx.stroke();
      ctx.setLineDash([]);

      // 数据点
      ctx.fillStyle = colors.download;
      ctx.beginPath();
      ctx.arc(cx, toY(closest.download), 3, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = colors.upload;
      ctx.beginPath();
      ctx.arc(cx, toY(closest.upload), 3, 0, Math.PI * 2);
      ctx.fill();

      // 更新 DOM tooltip（使用 DOM API 替代 innerHTML 防止 XSS）
      const tooltip = tooltipRef.current;
      if (tooltip) {
        tooltip.style.display = "block";
        // 防止 tooltip 溢出右边界（使用 CSS 像素，非设备像素）
        const tooltipLeft = cx + 12;
        const cssWidth = canvas.width / (window.devicePixelRatio || 1);
        const maxLeft = cssWidth - PADDING.right - 120; // 120px ≈ tooltip 宽度
        tooltip.style.left = `${Math.min(tooltipLeft, maxLeft)}px`;
        tooltip.style.top = `${PADDING.top + 8}px`;
        // 清空旧内容
        while (tooltip.firstChild) tooltip.removeChild(tooltip.firstChild);
        // 时间标签
        const timeDiv = document.createElement("div");
        timeDiv.style.cssText = `font-size:10px;color:${textColor};margin-bottom:4px`;
        timeDiv.textContent = formatTimeLabel(closest.time);
        tooltip.appendChild(timeDiv);
        // 下载速度
        const dlDiv = document.createElement("div");
        dlDiv.style.cssText = `color:${colors.download};font-size:12px`;
        dlDiv.textContent = `${t("speedChart.download")}: ${formatSpeed(closest.download)}`;
        tooltip.appendChild(dlDiv);
        // 上传速度
        const ulDiv = document.createElement("div");
        ulDiv.style.cssText = `color:${colors.upload};font-size:12px`;
        ulDiv.textContent = `${t("speedChart.upload")}: ${formatSpeed(closest.upload)}`;
        tooltip.appendChild(ulDiv);
      }
    } else {
      const tooltip = tooltipRef.current;
      if (tooltip) tooltip.style.display = "none";
    }
  }, [rangeSeconds, t]);

  // 动画循环：无数据时停止 rAF 节省 CPU
  useEffect(() => {
    let running = true;
    const loop = () => {
      if (!running) return;
      draw();
      // 检查是否有数据，无数据时降低刷新频率
      const speedHistory = useTaskStore.getState().speedHistory;
      const cutoff = Date.now() - rangeSeconds * 1000;
      const hasData = speedHistory.some((p) => p.time >= cutoff);
      if (hasData) {
        rafRef.current = requestAnimationFrame(loop);
      } else {
        // 无数据时每秒检查一次
        rafRef.current = requestAnimationFrame(() => {
          setTimeout(() => {
            if (running) rafRef.current = requestAnimationFrame(loop);
          }, 1000);
        });
      }
    };
    rafRef.current = requestAnimationFrame(loop);
    return () => {
      running = false;
      cancelAnimationFrame(rafRef.current);
    };
  }, [draw, rangeSeconds]);

  // 鼠标移动（仅更新 ref，由 rAF 循环读取并更新 DOM，不触发 React 重渲染）
  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLDivElement>) => {
      const rect = e.currentTarget.getBoundingClientRect();
      hoverXRef.current = e.clientX - rect.left;
    },
    []
  );

  const handleMouseLeave = useCallback(() => {
    hoverXRef.current = null;
  }, []);

  return (
    <div className="h-full flex flex-col bg-secondary">
      {/* 标题栏 */}
      <div className="flex items-center justify-between px-4 py-1.5 border-b border-border">
        <span className="text-sm font-medium text-text-primary">{t("speedChart.title")}</span>
        <div className="flex gap-1">
          {TIME_RANGES.map((r, i) => (
            <button
              key={r.label}
              onClick={() => setRangeIdx(i)}
              className={`px-2 py-0.5 rounded text-xs transition-colors
                ${
                  i === rangeIdx
                    ? "bg-accent text-white"
                    : "text-text-secondary hover:bg-tertiary"
                }`}
            >
              {r.label}
            </button>
          ))}
        </div>
      </div>

      {/* Canvas 图表 */}
      <div
        ref={containerRef}
        className="flex-1 relative"
        onMouseMove={handleMouseMove}
        onMouseLeave={handleMouseLeave}
      >
        <canvas ref={canvasRef} className="absolute inset-0" />
        <div
          ref={tooltipRef}
          className="absolute pointer-events-none hidden z-10 px-2.5 py-1.5 rounded-md text-xs shadow-sm"
          style={{
            backgroundColor: "var(--bg-primary)",
            border: "1px solid var(--border)",
          }}
        />
      </div>
    </div>
  );
}
