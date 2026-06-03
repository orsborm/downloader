// 任务状态管理 Store（Zustand）
// 管理所有下载任务的状态、选择、全局统计

import { create } from "zustand";
import type { TaskStatus, TaskUpdateEvent } from "../lib/types";

/** 列排序键 */
export type SortKey = "name" | "size" | "progress" | "speed" | "addedAt";

/** 状态过滤类型 */
export type StatusFilter = "all" | "downloading" | "paused" | "done" | "error" | "seeding" | "queued";

/** 状态优先级（数值越小优先级越高） */
const STATE_ORDER: Record<string, number> = {
  downloading: 0,
  seeding: 1,
  queued: 2,
  paused: 3,
  done: 4,
  error: 5,
};

/** 任务 Store 状态 */
interface TaskStoreState {
  /** 任务映射表（id → TaskStatus） */
  tasks: Map<string, TaskStatus>;
  /** 当前选中的任务 ID 集合 */
  selectedIds: Set<string>;
  /** 最后一次选中的 ID（用于 Shift 范围选择） */
  lastSelectedId: string | null;
  /** 速度历史（最近 1 小时，每秒一个点） */
  speedHistory: { time: number; download: number; upload: number }[];
  /** 排序键 */
  sortKey: SortKey;
  /** 排序方向 */
  sortAsc: boolean;
  /** 搜索关键词 */
  searchQuery: string;
  /** 状态过滤 */
  statusFilter: StatusFilter;
}

/** 任务 Store 操作 */
interface TaskStoreActions {
  /** 初始化任务列表（从后端加载） */
  setTasks: (tasks: TaskStatus[]) => void;
  /** 处理后端推送的状态更新 */
  applyUpdate: (event: TaskUpdateEvent) => void;
  /** 选择任务 */
  selectTask: (id: string, ctrlKey?: boolean, shiftKey?: boolean) => void;
  /** 清除选择 */
  clearSelection: () => void;
  /** 删除任务 */
  removeTask: (id: string) => void;
  /** 设置排序 */
  setSort: (key: SortKey) => void;
  /** 设置搜索关键词 */
  setSearchQuery: (query: string) => void;
  /** 设置状态过滤 */
  setStatusFilter: (filter: StatusFilter) => void;
  /** 获取排序后的任务列表 */
  getSortedTasks: () => TaskStatus[];
  /** 根据 ID 获取任务 */
  getTaskById: (id: string) => TaskStatus | undefined;
  /** 获取全局统计 */
  getGlobalStats: () => {
    totalSpeed: number;
    totalUploadSpeed: number;
    activeCount: number;
    totalCount: number;
  };
}

export const useTaskStore = create<TaskStoreState & TaskStoreActions>(
  (set, get) => ({
    tasks: new Map(),
    selectedIds: new Set(),
    lastSelectedId: null,
    speedHistory: [],
    sortKey: "addedAt",
    sortAsc: false,
    searchQuery: "",
    statusFilter: "all",

    setTasks: (tasks) => {
      const map = new Map<string, TaskStatus>();
      tasks.forEach((t) => map.set(t.id, t));
      set({ tasks: map });
    },

    applyUpdate: (event) => {
      set((state) => {
        const newTasks = new Map(state.tasks);
        const existing = newTasks.get(event.taskId);

        if (existing) {
          // 更新已有任务的状态
          // progress < 0 表示不更新进度（暂停/恢复时保留现有值）
          const shouldUpdateProgress = event.progress >= 0;
          newTasks.set(event.taskId, {
            ...existing,
            state: event.state,
            downloaded: event.downloaded > 0 ? event.downloaded : existing.downloaded,
            totalSize: event.totalSize > 0 ? event.totalSize : existing.totalSize,
            downloadSpeed: event.downloadSpeed,
            uploadSpeed: event.uploadSpeed,
            progress: shouldUpdateProgress ? event.progress : existing.progress,
            peers: event.peers,
            error: event.error,
            eta: event.eta,
          });
        }

        // 更新速度历史（节流：每秒最多一个点）
        // 使用全局速度总和而非单个事件的速度
        const now = Date.now();
        const lastPoint = state.speedHistory[state.speedHistory.length - 1];
        if (!lastPoint || now - lastPoint.time >= 1000) {
          // 仅在需要更新时才复制数组，避免每次渲染都创建新数组
          const history = [...state.speedHistory];
          let globalDownload = 0;
          let globalUpload = 0;
          newTasks.forEach((t) => {
            if (t.state === "downloading" || t.state === "seeding") {
              globalDownload += t.downloadSpeed;
              globalUpload += t.uploadSpeed;
            }
          });
          history.push({
            time: now,
            download: globalDownload,
            upload: globalUpload,
          });
          // 保留最近 1 小时（与 SpeedChart 最大时间范围一致）
          // 使用长度限制替代按时间查找，O(1) 摊销
          const MAX_HISTORY = 3600; // 1小时 × 每秒1条
          if (history.length > MAX_HISTORY) {
            history.splice(0, history.length - MAX_HISTORY);
          }
          return { tasks: newTasks, speedHistory: history };
        }

        return { tasks: newTasks };
      });
    },

    selectTask: (id, ctrlKey, shiftKey) => {
      set((state) => {
        const newSelected = new Set(state.selectedIds);

        if (shiftKey && state.lastSelectedId) {
          // Shift 范围选择（使用排序后的顺序，而非插入顺序）
          const sorted = get().getSortedTasks();
          const taskIds = sorted.map((t) => t.id);
          const startIdx = taskIds.indexOf(state.lastSelectedId);
          const endIdx = taskIds.indexOf(id);
          if (startIdx !== -1 && endIdx !== -1) {
            const [from, to] =
              startIdx < endIdx ? [startIdx, endIdx] : [endIdx, startIdx];
            for (let i = from; i <= to; i++) {
              newSelected.add(taskIds[i]);
            }
          }
        } else if (ctrlKey) {
          // Ctrl 切换选择
          if (newSelected.has(id)) {
            newSelected.delete(id);
          } else {
            newSelected.add(id);
          }
        } else {
          // 单选
          newSelected.clear();
          newSelected.add(id);
        }

        return { selectedIds: newSelected, lastSelectedId: id };
      });
    },

    clearSelection: () => set({ selectedIds: new Set(), lastSelectedId: null }),

    setSort: (key) => {
      set((state) => {
        if (state.sortKey === key) {
          return { sortAsc: !state.sortAsc };
        }
        return { sortKey: key, sortAsc: true };
      });
    },

    setSearchQuery: (query) => set({ searchQuery: query }),

    setStatusFilter: (filter) => set({ statusFilter: filter }),

    removeTask: (id) => {
      set((state) => {
        const newTasks = new Map(state.tasks);
        newTasks.delete(id);
        const newSelected = new Set(state.selectedIds);
        newSelected.delete(id);
        return {
          tasks: newTasks,
          selectedIds: newSelected,
          lastSelectedId: state.lastSelectedId === id ? null : state.lastSelectedId,
        };
      });
    },

    getTaskById: (id) => get().tasks.get(id),

    getSortedTasks: () => {
      const { tasks, sortKey, sortAsc, searchQuery, statusFilter } = get();
      let sorted = Array.from(tasks.values());

      // 应用状态过滤
      if (statusFilter !== "all") {
        sorted = sorted.filter((t) => t.state === statusFilter);
      }

      // 应用搜索过滤（匹配文件名或 URL）
      if (searchQuery.trim()) {
        const q = searchQuery.trim().toLowerCase();
        sorted = sorted.filter(
          (t) =>
            t.name.toLowerCase().includes(q) ||
            t.url.toLowerCase().includes(q)
        );
      }

      // 状态优先级排序（始终作为次要排序）

      sorted.sort((a, b) => {
        let cmp = 0;
        switch (sortKey) {
          case "name":
            cmp = a.name.localeCompare(b.name);
            break;
          case "size":
            cmp = a.totalSize - b.totalSize;
            break;
          case "progress":
            cmp = a.progress - b.progress;
            break;
          case "speed":
            cmp = a.downloadSpeed - b.downloadSpeed;
            break;
          case "addedAt":
          default:
            cmp = new Date(a.addedAt).getTime() - new Date(b.addedAt).getTime();
            break;
        }
        if (cmp !== 0) return sortAsc ? cmp : -cmp;

        // 次要排序：状态优先级
        const orderA = STATE_ORDER[a.state] ?? 99;
        const orderB = STATE_ORDER[b.state] ?? 99;
        return orderA - orderB;
      });

      return sorted;
    },

    getGlobalStats: () => {
      const { tasks } = get();
      let totalSpeed = 0;
      let totalUploadSpeed = 0;
      let activeCount = 0;

      tasks.forEach((t) => {
        if (t.state === "downloading" || t.state === "seeding") {
          totalSpeed += t.downloadSpeed;
          totalUploadSpeed += t.uploadSpeed;
          activeCount++;
        }
      });

      return {
        totalSpeed,
        totalUploadSpeed,
        activeCount,
        totalCount: tasks.size,
      };
    },
  })
);
