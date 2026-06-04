// 全局确认对话框组件
// 替代 window.confirm()，支持队列、i18n、一致视觉风格

import { useState, useEffect, useCallback, useRef } from "react";
import { AlertTriangle, X } from "lucide-react";
import { useI18n } from "../hooks/useI18n";

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "warning" | "info";
}

interface QueuedConfirm extends ConfirmOptions {
  id: number;
  resolve: (value: boolean) => void;
}

// 全局队列
let nextId = 0;
const queue: QueuedConfirm[] = [];
let notifyListener: (() => void) | null = null;

export function confirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    queue.push({ ...options, id: nextId++, resolve });
    notifyListener?.();
  });
}

export function ConfirmDialogContainer() {
  const { t } = useI18n();
  const [current, setCurrent] = useState<QueuedConfirm | null>(null);
  const processingRef = useRef(false);

  const processQueue = useCallback(() => {
    if (processingRef.current || queue.length === 0) return;
    processingRef.current = true;
    setCurrent(queue.shift()!);
  }, []);

  useEffect(() => {
    notifyListener = processQueue;
    processQueue();
    return () => {
      notifyListener = null;
    };
  }, [processQueue]);

  const handleConfirm = useCallback(() => {
    current?.resolve(true);
    processingRef.current = false;
    setCurrent(null);
    // 处理队列中的下一个
    setTimeout(processQueue, 0);
  }, [current, processQueue]);

  const handleCancel = useCallback(() => {
    current?.resolve(false);
    processingRef.current = false;
    setCurrent(null);
    setTimeout(processQueue, 0);
  }, [current, processQueue]);

  // Escape 关闭
  useEffect(() => {
    if (!current) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") handleCancel();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [current, handleCancel]);

  if (!current) return null;

  const variant = current.variant || "danger";
  const iconColor =
    variant === "danger"
      ? "text-error"
      : variant === "warning"
        ? "text-warning"
        : "text-accent";
  const btnColor =
    variant === "danger"
      ? "bg-error hover:bg-error/90"
      : variant === "warning"
        ? "bg-warning hover:bg-warning/90"
        : "bg-accent hover:bg-accent/90";

  return (
    <div
      className="fixed inset-0 z-[200] flex items-center justify-center bg-black/40 animate-fade-in"
      onClick={handleCancel}
    >
      <div
        className="bg-secondary rounded-lg shadow-xl border border-border w-[90vw] max-w-[400px] animate-slide-up"
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-4 py-3 border-b border-border">
          <div className="flex items-center gap-2">
            <AlertTriangle size={18} className={iconColor} />
            <span className="text-sm font-medium text-text-primary">
              {current.title}
            </span>
          </div>
          <button
            onClick={handleCancel}
            aria-label="Close"
            className="p-1 rounded-md hover:bg-tertiary hover:text-text-primary transition-colors text-text-muted"
          >
            <X size={16} />
          </button>
        </div>
        <div className="px-4 py-3">
          <p className="text-sm text-text-secondary">{current.message}</p>
        </div>
        <div className="flex justify-end gap-2 px-4 py-3 border-t border-border">
          <button
            onClick={handleCancel}
            autoFocus
            className="px-4 py-1.5 text-sm rounded-md border border-border text-text-primary hover:bg-tertiary transition-colors"
          >
            {current.cancelLabel || t("common.cancel")}
          </button>
          <button
            onClick={handleConfirm}
            className={`px-4 py-1.5 text-sm rounded-md text-white transition-colors ${btnColor}`}
          >
            {current.confirmLabel || t("common.confirm")}
          </button>
        </div>
      </div>
    </div>
  );
}
