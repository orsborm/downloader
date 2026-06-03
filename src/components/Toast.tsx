// 轻量级 Toast 通知组件
// 支持 success/error/info 类型，自动消失
// 位置跟随设置页"通知位置"配置

import { useState, useEffect, useCallback, useRef } from "react";
import { CheckCircle, XCircle, Info, X } from "lucide-react";

export type ToastType = "success" | "error" | "info";

export interface ToastMessage {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

// 全局 toast 管理（使用 globalThis 避免 HMR 时重置）
const g = globalThis as unknown as { __toastListeners?: ((toast: ToastMessage) => void)[] };
const toastListeners: ((toast: ToastMessage) => void)[] = g.__toastListeners ??= [];

export function showToast(message: string, type: ToastType = "info", duration = 3000) {
  const toast: ToastMessage = {
    id: crypto.randomUUID(),
    type,
    message,
    duration,
  };
  toastListeners.forEach((listener) => listener(toast));
}

/** 通知位置映射（从 config.toml 的 notification.position 读取） */
function getPositionClasses(): string {
  try {
    // 从 localStorage 读取配置（Tauri 应用通过 tauri-api 保存时会同步到 localStorage）
    const stored = localStorage.getItem("downloader-notification-position") || "bottom-right";
    switch (stored) {
      case "top-left": return "top-16 left-4";
      case "top-right": return "top-16 right-4";
      case "bottom-left": return "bottom-16 left-4";
      case "bottom-right":
      default: return "bottom-16 right-4";
    }
  } catch {
    return "bottom-16 right-4";
  }
}

export function ToastContainer() {
  const [toasts, setToasts] = useState<ToastMessage[]>([]);
  const [position, setPosition] = useState(getPositionClasses);
  const timersRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
    const timer = timersRef.current.get(id);
    if (timer) {
      clearTimeout(timer);
      timersRef.current.delete(id);
    }
  }, []);

  // 监听通知位置变化
  useEffect(() => {
    const checkPosition = () => setPosition(getPositionClasses());
    // 监听 storage 事件（跨标签页同步）
    window.addEventListener("storage", checkPosition);
    // 监听自定义事件（同标签页内 settings 即时更新）
    window.addEventListener("toast-position-changed", checkPosition);
    return () => {
      window.removeEventListener("storage", checkPosition);
      window.removeEventListener("toast-position-changed", checkPosition);
    };
  }, []);

  useEffect(() => {
    const listener = (toast: ToastMessage) => {
      setToasts((prev) => [...prev.slice(-4), toast]); // max 5 toasts
      const timer = setTimeout(() => removeToast(toast.id), toast.duration || 3000);
      timersRef.current.set(toast.id, timer);
    };

    toastListeners.push(listener);
    return () => {
      const idx = toastListeners.indexOf(listener);
      if (idx >= 0) toastListeners.splice(idx, 1);
    };
  }, [removeToast]);

  // 组件卸载时清理所有定时器
  useEffect(() => {
    return () => {
      timersRef.current.forEach((timer) => clearTimeout(timer));
      timersRef.current.clear();
    };
  }, []);

  return (
    <div className={`fixed ${position} z-[100] flex flex-col gap-2 pointer-events-none`} aria-live="polite">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className={`pointer-events-auto flex items-center gap-2 px-4 py-2.5 rounded-lg shadow-lg
            border text-sm max-w-sm animate-slide-in-right
            ${toast.type === "success" ? "bg-success/10 border-success/30 text-success" : ""}
            ${toast.type === "error" ? "bg-error/10 border-error/30 text-error" : ""}
            ${toast.type === "info" ? "bg-accent/10 border-accent/30 text-accent" : ""}
          `}
        >
          {toast.type === "success" && <CheckCircle size={16} />}
          {toast.type === "error" && <XCircle size={16} />}
          {toast.type === "info" && <Info size={16} />}
          <span className="flex-1 text-text-primary">{toast.message}</span>
          <button
            onClick={() => removeToast(toast.id)}
            className="p-0.5 rounded hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted"
          >
            <X size={14} />
          </button>
        </div>
      ))}
    </div>
  );
}
