// 任务事件订阅 Hook
// 监听后端推送的任务状态更新事件，自动同步到 Zustand Store
// 任务完成/出错时显示系统通知

import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { useTaskStore } from "../stores/taskStore";
import { t } from "./useI18n";
import type { TaskUpdateEvent } from "../lib/types";

/** 通知去重：记录已通知的任务 ID */
const notifiedTasks = new Set<string>();

/**
 * 订阅后端任务状态更新事件
 * 组件挂载时开始监听，卸载时自动清理
 */
export function useTaskEvents() {
  const applyUpdate = useTaskStore((s) => s.applyUpdate);
  const tasks = useTaskStore((s) => s.tasks);
  const getTaskById = useTaskStore((s) => s.getTaskById);
  const unlistenRef = useRef<(() => void) | null>(null);

  // 清理已删除任务的通知记录，防止内存泄漏
  useEffect(() => {
    for (const id of notifiedTasks) {
      if (!tasks.has(id)) {
        notifiedTasks.delete(id);
      }
    }
  }, [tasks]);

  useEffect(() => {
    // 监听 "task-update" 事件
    const setup = async () => {
      const unlisten = await listen<TaskUpdateEvent>(
        "task-update",
        (event) => {
          const payload = event.payload;
          applyUpdate(payload);

          // 任务完成通知
          if (payload.state === "done" && !notifiedTasks.has(payload.taskId)) {
            notifiedTasks.add(payload.taskId);
            const task = getTaskById(payload.taskId);
            sendNotification({
              title: t("notification.taskComplete"),
              body: task?.name
                ? t("notification.taskCompleteBody", { name: task.name })
                : t("notification.taskComplete"),
            });
          }

          // 任务出错通知
          if (payload.state === "error" && !notifiedTasks.has(payload.taskId)) {
            notifiedTasks.add(payload.taskId);
            sendNotification({
              title: t("notification.taskError"),
              body: payload.error || t("notification.taskError"),
            });
          }
        }
      );
      unlistenRef.current = unlisten;
    };

    setup();

    return () => {
      // 清理监听器
      unlistenRef.current?.();
    };
  }, [applyUpdate]);
}
