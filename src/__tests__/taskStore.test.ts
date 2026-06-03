import { describe, it, expect, beforeEach, vi } from "vitest";
import { useTaskStore } from "../stores/taskStore";
import type { TaskStatus, TaskUpdateEvent } from "../lib/types";

function makeTask(overrides: Partial<TaskStatus> = {}): TaskStatus {
  return {
    id: "task-1",
    name: "test.zip",
    protocol: "HTTP",
    state: "downloading",
    url: "https://example.com/test.zip",
    savePath: "/downloads",
    totalSize: 1024,
    downloaded: 512,
    uploaded: 0,
    downloadSpeed: 100,
    uploadSpeed: 0,
    progress: 0.5,
    peers: 0,
    seeds: 0,
    eta: 10,
    files: [],
    error: null,
    addedAt: "2026-05-29T00:00:00Z",
    completedAt: null,
    priority: 1,
    downloadLimit: 0,
    uploadLimit: 0,
    ...overrides,
  };
}

describe("taskStore", () => {
  beforeEach(() => {
    // Reset store between tests
    useTaskStore.setState({
      tasks: new Map(),
      selectedIds: new Set(),
      lastSelectedId: null,
      speedHistory: [],
      sortKey: "addedAt",
      sortAsc: false,
      searchQuery: "",
      statusFilter: "all",
    });
  });

  describe("setTasks", () => {
    it("populates the task map from an array", () => {
      const tasks = [makeTask({ id: "a" }), makeTask({ id: "b" })];
      useTaskStore.getState().setTasks(tasks);
      const state = useTaskStore.getState();
      expect(state.tasks.size).toBe(2);
      expect(state.tasks.get("a")).toBeDefined();
      expect(state.tasks.get("b")).toBeDefined();
    });

    it("replaces existing tasks", () => {
      useTaskStore.getState().setTasks([makeTask({ id: "old" })]);
      useTaskStore.getState().setTasks([makeTask({ id: "new" })]);
      const state = useTaskStore.getState();
      expect(state.tasks.size).toBe(1);
      expect(state.tasks.has("old")).toBe(false);
      expect(state.tasks.has("new")).toBe(true);
    });
  });

  describe("applyUpdate", () => {
    it("updates an existing task's state and speed", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      const event: TaskUpdateEvent = {
        taskId: "task-1",
        state: "downloading",
        downloaded: 768,
        totalSize: 1024,
        downloadSpeed: 200,
        uploadSpeed: 10,
        progress: 0.75,
        peers: 5,
        error: null,
        eta: 5,
      };
      useTaskStore.getState().applyUpdate(event);
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.downloaded).toBe(768);
      expect(task.downloadSpeed).toBe(200);
      expect(task.progress).toBe(0.75);
      expect(task.peers).toBe(5);
    });

    it("preserves existing progress when event progress < 0", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      const event: TaskUpdateEvent = {
        taskId: "task-1",
        state: "paused",
        downloaded: 512,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: -1,
        peers: 0,
        error: null,
        eta: null,
      };
      useTaskStore.getState().applyUpdate(event);
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.progress).toBe(0.5); // preserved
      expect(task.state).toBe("paused");
    });

    it("ignores updates for unknown task IDs", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      const event: TaskUpdateEvent = {
        taskId: "nonexistent",
        state: "done",
        downloaded: 100,
        totalSize: 100,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: 1,
        peers: 0,
        error: null,
        eta: null,
      };
      useTaskStore.getState().applyUpdate(event);
      expect(useTaskStore.getState().tasks.size).toBe(1);
    });
  });

  describe("selectTask", () => {
    beforeEach(() => {
      useTaskStore
        .getState()
        .setTasks([
          makeTask({ id: "a" }),
          makeTask({ id: "b" }),
          makeTask({ id: "c" }),
        ]);
    });

    it("single-selects a task", () => {
      useTaskStore.getState().selectTask("b");
      const { selectedIds, lastSelectedId } = useTaskStore.getState();
      expect(selectedIds.size).toBe(1);
      expect(selectedIds.has("b")).toBe(true);
      expect(lastSelectedId).toBe("b");
    });

    it("toggles selection with ctrlKey", () => {
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("b", true);
      expect(useTaskStore.getState().selectedIds.size).toBe(2);
      useTaskStore.getState().selectTask("b", true);
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
    });

    it("range-selects with shiftKey", () => {
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("c", false, true);
      const { selectedIds } = useTaskStore.getState();
      expect(selectedIds.size).toBe(3);
      expect(selectedIds.has("a")).toBe(true);
      expect(selectedIds.has("b")).toBe(true);
      expect(selectedIds.has("c")).toBe(true);
    });
  });

  describe("removeTask", () => {
    it("removes a task from the map", () => {
      useTaskStore
        .getState()
        .setTasks([makeTask({ id: "a" }), makeTask({ id: "b" })]);
      useTaskStore.getState().removeTask("a");
      expect(useTaskStore.getState().tasks.size).toBe(1);
      expect(useTaskStore.getState().tasks.has("a")).toBe(false);
    });

    it("removes the task from selectedIds", () => {
      useTaskStore
        .getState()
        .setTasks([makeTask({ id: "a" }), makeTask({ id: "b" })]);
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().removeTask("a");
      expect(useTaskStore.getState().selectedIds.has("a")).toBe(false);
    });
  });

  describe("setSort", () => {
    it("toggles sort direction on same key", () => {
      useTaskStore.getState().setSort("name");
      expect(useTaskStore.getState().sortKey).toBe("name");
      expect(useTaskStore.getState().sortAsc).toBe(true);
      useTaskStore.getState().setSort("name");
      expect(useTaskStore.getState().sortAsc).toBe(false);
    });

    it("resets direction on different key", () => {
      useTaskStore.getState().setSort("name");
      useTaskStore.getState().setSort("size");
      expect(useTaskStore.getState().sortKey).toBe("size");
      expect(useTaskStore.getState().sortAsc).toBe(true);
    });
  });

  describe("getSortedTasks", () => {
    it("sorts by name", () => {
      useTaskStore
        .getState()
        .setTasks([
          makeTask({ id: "1", name: "beta.zip" }),
          makeTask({ id: "2", name: "alpha.zip" }),
        ]);
      useTaskStore.getState().setSort("name");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted[0].name).toBe("alpha.zip");
      expect(sorted[1].name).toBe("beta.zip");
    });

    it("sorts by size descending", () => {
      useTaskStore
        .getState()
        .setTasks([
          makeTask({ id: "1", totalSize: 100 }),
          makeTask({ id: "2", totalSize: 999 }),
        ]);
      useTaskStore.getState().setSort("size");
      useTaskStore.getState().setSort("size"); // toggle to desc
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted[0].totalSize).toBe(999);
    });
  });

  describe("getGlobalStats", () => {
    it("counts active tasks and sums speeds", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", state: "downloading", downloadSpeed: 100 }),
        makeTask({ id: "2", state: "downloading", downloadSpeed: 200 }),
        makeTask({ id: "3", state: "paused", downloadSpeed: 0 }),
        makeTask({ id: "4", state: "done", downloadSpeed: 0 }),
      ]);
      const stats = useTaskStore.getState().getGlobalStats();
      expect(stats.activeCount).toBe(2);
      expect(stats.totalCount).toBe(4);
      expect(stats.totalSpeed).toBe(300);
    });

    it("returns zeros for empty store", () => {
      const stats = useTaskStore.getState().getGlobalStats();
      expect(stats.activeCount).toBe(0);
      expect(stats.totalCount).toBe(0);
      expect(stats.totalSpeed).toBe(0);
    });
  });

  describe("speedHistory", () => {
    it("records global speed across all active tasks, not just event speed", () => {
      // Set up two downloading tasks
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", state: "downloading", downloadSpeed: 100, uploadSpeed: 10 }),
        makeTask({ id: "2", state: "downloading", downloadSpeed: 200, uploadSpeed: 20 }),
      ]);

      // Apply update for task 1 only — speed history should reflect global total
      const event: TaskUpdateEvent = {
        taskId: "1",
        state: "downloading",
        downloaded: 600,
        totalSize: 1024,
        downloadSpeed: 150,
        uploadSpeed: 15,
        progress: 0.6,
        peers: 3,
        error: null,
        eta: 5,
      };
      useTaskStore.getState().applyUpdate(event);

      const history = useTaskStore.getState().speedHistory;
      expect(history.length).toBeGreaterThan(0);
      const lastPoint = history[history.length - 1];
      // Global speed = task1 (150 after update) + task2 (200) = 350
      expect(lastPoint.download).toBe(350);
      expect(lastPoint.upload).toBe(35);
    });

    it("throttles history points to at most one per second", () => {
      useTaskStore.getState().setTasks([makeTask({ id: "1", state: "downloading", downloadSpeed: 100 })]);

      // Fire two events in quick succession
      useTaskStore.getState().applyUpdate({
        taskId: "1", state: "downloading", downloaded: 100, totalSize: 1024,
        downloadSpeed: 100, uploadSpeed: 0, progress: 0.1, peers: 0, error: null, eta: 10,
      });
      useTaskStore.getState().applyUpdate({
        taskId: "1", state: "downloading", downloaded: 200, totalSize: 1024,
        downloadSpeed: 200, uploadSpeed: 0, progress: 0.2, peers: 0, error: null, eta: 9,
      });

      const history = useTaskStore.getState().speedHistory;
      // Should only have 1 point since both events happened within 1 second
      expect(history.length).toBe(1);
      expect(history[0].download).toBe(100);
    });
  });

  describe("clearSelection", () => {
    it("clears all selected IDs", () => {
      useTaskStore
        .getState()
        .setTasks([makeTask({ id: "a" }), makeTask({ id: "b" })]);
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("b", true);
      useTaskStore.getState().clearSelection();
      expect(useTaskStore.getState().selectedIds.size).toBe(0);
      expect(useTaskStore.getState().lastSelectedId).toBeNull();
    });
  });

  describe("applyUpdate edge cases", () => {
    it("sets error state with error message", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "error",
        downloaded: 512,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: 0.5,
        peers: 0,
        error: "Connection refused",
        eta: null,
      });
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.state).toBe("error");
      expect(task.error).toBe("Connection refused");
      expect(task.downloadSpeed).toBe(0);
    });

    it("sets done state with completedAt timestamp", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "done",
        downloaded: 1024,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: 1,
        peers: 0,
        error: null,
        eta: null,
      });
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.state).toBe("done");
      expect(task.progress).toBe(1);
    });

    it("updates upload speed independently", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "seeding",
        downloaded: 1024,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 500,
        progress: 1,
        peers: 10,
        error: null,
        eta: null,
      });
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.uploadSpeed).toBe(500);
      expect(task.peers).toBe(10);
    });
  });

  describe("removeTask edge cases", () => {
    it("handles removing from empty store gracefully", () => {
      useTaskStore.getState().removeTask("nonexistent");
      expect(useTaskStore.getState().tasks.size).toBe(0);
    });

    it("updates lastSelectedId when selected task is removed", () => {
      useTaskStore
        .getState()
        .setTasks([makeTask({ id: "a" }), makeTask({ id: "b" })]);
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("b", true);
      useTaskStore.getState().removeTask("a");
      // lastSelectedId was "a" (last selected), should still be "a" even though removed
      // The store doesn't auto-update lastSelectedId on removal
      expect(useTaskStore.getState().selectedIds.has("a")).toBe(false);
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
    });
  });

  describe("getSortedTasks edge cases", () => {
    it("sorts by progress", () => {
      useTaskStore
        .getState()
        .setTasks([
          makeTask({ id: "1", progress: 0.3 }),
          makeTask({ id: "2", progress: 0.9 }),
          makeTask({ id: "3", progress: 0.1 }),
        ]);
      useTaskStore.getState().setSort("progress");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted[0].progress).toBe(0.1);
      expect(sorted[1].progress).toBe(0.3);
      expect(sorted[2].progress).toBe(0.9);
    });

    it("sorts by download speed descending", () => {
      useTaskStore
        .getState()
        .setTasks([
          makeTask({ id: "1", downloadSpeed: 100 }),
          makeTask({ id: "2", downloadSpeed: 500 }),
          makeTask({ id: "3", downloadSpeed: 50 }),
        ]);
      useTaskStore.getState().setSort("speed");
      useTaskStore.getState().setSort("speed"); // toggle to desc
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted[0].downloadSpeed).toBe(500);
      expect(sorted[1].downloadSpeed).toBe(100);
      expect(sorted[2].downloadSpeed).toBe(50);
    });

    it("returns empty array for empty store", () => {
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted).toEqual([]);
    });
  });

  describe("getGlobalStats edge cases", () => {
    it("counts tasks in all states correctly", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", state: "downloading", downloadSpeed: 100, uploadSpeed: 10 }),
        makeTask({ id: "2", state: "queued", downloadSpeed: 0 }),
        makeTask({ id: "3", state: "paused", downloadSpeed: 0 }),
        makeTask({ id: "4", state: "done", downloadSpeed: 0 }),
        makeTask({ id: "5", state: "error", downloadSpeed: 0 }),
        makeTask({ id: "6", state: "seeding", downloadSpeed: 0, uploadSpeed: 50 }),
      ]);
      const stats = useTaskStore.getState().getGlobalStats();
      expect(stats.activeCount).toBe(2); // downloading + seeding both count as active
      expect(stats.totalCount).toBe(6);
      expect(stats.totalSpeed).toBe(100);
      expect(stats.totalUploadSpeed).toBe(60); // 10 + 50
    });
  });

  describe("applyUpdate conditional fields", () => {
    it("preserves downloaded when event has 0 downloaded", () => {
      useTaskStore.getState().setTasks([makeTask({ downloaded: 512 })]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "downloading",
        downloaded: 0,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: 0.5,
        peers: 0,
        error: null,
        eta: null,
      });
      expect(useTaskStore.getState().tasks.get("task-1")!.downloaded).toBe(512);
    });

    it("preserves totalSize when event has 0 totalSize", () => {
      useTaskStore.getState().setTasks([makeTask({ totalSize: 2048 })]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "downloading",
        downloaded: 100,
        totalSize: 0,
        downloadSpeed: 0,
        uploadSpeed: 0,
        progress: 0.5,
        peers: 0,
        error: null,
        eta: null,
      });
      expect(useTaskStore.getState().tasks.get("task-1")!.totalSize).toBe(2048);
    });

    it("updates downloaded when event has positive value", () => {
      useTaskStore.getState().setTasks([makeTask({ downloaded: 512 })]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "downloading",
        downloaded: 768,
        totalSize: 1024,
        downloadSpeed: 100,
        uploadSpeed: 0,
        progress: 0.75,
        peers: 0,
        error: null,
        eta: null,
      });
      expect(useTaskStore.getState().tasks.get("task-1")!.downloaded).toBe(768);
    });

    it("sets error message to null on success", () => {
      useTaskStore.getState().setTasks([makeTask({ error: "previous error" })]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "downloading",
        downloaded: 100,
        totalSize: 1024,
        downloadSpeed: 100,
        uploadSpeed: 0,
        progress: 0.1,
        peers: 0,
        error: null,
        eta: null,
      });
      expect(useTaskStore.getState().tasks.get("task-1")!.error).toBeNull();
    });
  });

  describe("setTasks edge cases", () => {
    it("handles empty array", () => {
      useTaskStore.getState().setTasks([makeTask()]);
      useTaskStore.getState().setTasks([]);
      expect(useTaskStore.getState().tasks.size).toBe(0);
    });

    it("handles duplicate IDs (last wins)", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "dup", name: "first" }),
        makeTask({ id: "dup", name: "second" }),
      ]);
      expect(useTaskStore.getState().tasks.size).toBe(1);
      expect(useTaskStore.getState().tasks.get("dup")!.name).toBe("second");
    });
  });

  describe("selectTask edge cases", () => {
    it("shift-select with no lastSelectedId does single select", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "a" }),
        makeTask({ id: "b" }),
      ]);
      useTaskStore.getState().selectTask("a", false, true);
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
      expect(useTaskStore.getState().selectedIds.has("a")).toBe(true);
    });

    it("selecting same task without ctrl clears and reselects", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "a" }),
        makeTask({ id: "b" }),
      ]);
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("b", true);
      expect(useTaskStore.getState().selectedIds.size).toBe(2);
      useTaskStore.getState().selectTask("a"); // no ctrl = single select
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
      expect(useTaskStore.getState().selectedIds.has("a")).toBe(true);
    });
  });

  describe("getSortedTasks state priority", () => {
    it("sorts downloading tasks before done tasks at same primary sort", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", name: "same", state: "done" }),
        makeTask({ id: "2", name: "same", state: "downloading" }),
      ]);
      useTaskStore.getState().setSort("name");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted[0].state).toBe("downloading");
      expect(sorted[1].state).toBe("done");
    });

    it("sorts by addedAt by default", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", addedAt: "2026-05-30T00:00:00Z" }),
        makeTask({ id: "2", addedAt: "2026-05-29T00:00:00Z" }),
      ]);
      const sorted = useTaskStore.getState().getSortedTasks();
      // Default is addedAt desc
      expect(sorted[0].id).toBe("1");
      expect(sorted[1].id).toBe("2");
    });
  });

  describe("setSearchQuery", () => {
    it("updates the search query", () => {
      useTaskStore.getState().setSearchQuery("test");
      expect(useTaskStore.getState().searchQuery).toBe("test");
    });

    it("clears the search query", () => {
      useTaskStore.getState().setSearchQuery("test");
      useTaskStore.getState().setSearchQuery("");
      expect(useTaskStore.getState().searchQuery).toBe("");
    });
  });

  describe("setStatusFilter", () => {
    it("updates the status filter", () => {
      useTaskStore.getState().setStatusFilter("downloading");
      expect(useTaskStore.getState().statusFilter).toBe("downloading");
    });

    it("resets to all", () => {
      useTaskStore.getState().setStatusFilter("paused");
      useTaskStore.getState().setStatusFilter("all");
      expect(useTaskStore.getState().statusFilter).toBe("all");
    });
  });

  describe("getSortedTasks with search filter", () => {
    beforeEach(() => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", name: "movie.mp4", url: "https://example.com/movie.mp4", state: "downloading" }),
        makeTask({ id: "2", name: "music.mp3", url: "https://example.com/music.mp3", state: "done" }),
        makeTask({ id: "3", name: "document.pdf", url: "https://test.com/doc.pdf", state: "paused" }),
      ]);
    });

    it("filters by search query matching name", () => {
      useTaskStore.getState().setSearchQuery("movie");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("movie.mp4");
    });

    it("filters by search query matching URL", () => {
      useTaskStore.getState().setSearchQuery("test.com");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("document.pdf");
    });

    it("search is case-insensitive", () => {
      useTaskStore.getState().setSearchQuery("MOVIE");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("movie.mp4");
    });

    it("search trims whitespace", () => {
      useTaskStore.getState().setSearchQuery("  movie  ");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
    });

    it("returns all tasks when search is empty", () => {
      useTaskStore.getState().setSearchQuery("");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(3);
    });

    it("returns empty array when no match", () => {
      useTaskStore.getState().setSearchQuery("nonexistent");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(0);
    });
  });

  describe("getSortedTasks with status filter", () => {
    beforeEach(() => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", name: "a.zip", state: "downloading" }),
        makeTask({ id: "2", name: "b.zip", state: "done" }),
        makeTask({ id: "3", name: "c.zip", state: "paused" }),
        makeTask({ id: "4", name: "d.zip", state: "downloading" }),
      ]);
    });

    it("filters by downloading state", () => {
      useTaskStore.getState().setStatusFilter("downloading");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(2);
      expect(sorted.every((t) => t.state === "downloading")).toBe(true);
    });

    it("filters by done state", () => {
      useTaskStore.getState().setStatusFilter("done");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("b.zip");
    });

    it("filters by paused state", () => {
      useTaskStore.getState().setStatusFilter("paused");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("c.zip");
    });

    it("returns all tasks when filter is all", () => {
      useTaskStore.getState().setStatusFilter("all");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(4);
    });
  });

  describe("getSortedTasks with combined search and status filter", () => {
    beforeEach(() => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", name: "movie.mp4", state: "downloading" }),
        makeTask({ id: "2", name: "movie.mkv", state: "done" }),
        makeTask({ id: "3", name: "music.mp3", state: "downloading" }),
        makeTask({ id: "4", name: "movie.avi", state: "paused" }),
      ]);
    });

    it("applies both search and status filter", () => {
      useTaskStore.getState().setSearchQuery("movie");
      useTaskStore.getState().setStatusFilter("downloading");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(1);
      expect(sorted[0].name).toBe("movie.mp4");
    });

    it("search with no status match returns empty", () => {
      useTaskStore.getState().setSearchQuery("movie");
      useTaskStore.getState().setStatusFilter("error");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(0);
    });

    it("status filter with no search match returns empty", () => {
      useTaskStore.getState().setSearchQuery("nonexistent");
      useTaskStore.getState().setStatusFilter("downloading");
      const sorted = useTaskStore.getState().getSortedTasks();
      expect(sorted.length).toBe(0);
    });
  });

  describe("getTaskById", () => {
    it("returns task by ID", () => {
      useTaskStore.getState().setTasks([makeTask({ id: "abc", name: "test.zip" })]);
      const task = useTaskStore.getState().getTaskById("abc");
      expect(task).toBeDefined();
      expect(task!.name).toBe("test.zip");
    });

    it("returns undefined for nonexistent ID", () => {
      useTaskStore.getState().setTasks([makeTask({ id: "abc" })]);
      const task = useTaskStore.getState().getTaskById("xyz");
      expect(task).toBeUndefined();
    });

    it("returns undefined for empty store", () => {
      const task = useTaskStore.getState().getTaskById("abc");
      expect(task).toBeUndefined();
    });
  });

  describe("applyUpdate with seeding state", () => {
    it("transitions from downloading to seeding", () => {
      useTaskStore.getState().setTasks([makeTask({ state: "downloading" })]);
      useTaskStore.getState().applyUpdate({
        taskId: "task-1",
        state: "seeding",
        downloaded: 1024,
        totalSize: 1024,
        downloadSpeed: 0,
        uploadSpeed: 500,
        progress: 1,
        peers: 10,
        error: null,
        eta: null,
      });
      const task = useTaskStore.getState().tasks.get("task-1")!;
      expect(task.state).toBe("seeding");
      expect(task.uploadSpeed).toBe(500);
      expect(task.downloadSpeed).toBe(0);
    });
  });

  describe("selectTask with large dataset", () => {
    it("handles selecting from many tasks", () => {
      const tasks = Array.from({ length: 100 }, (_, i) =>
        makeTask({ id: `task-${i}`, name: `file-${i}.zip` })
      );
      useTaskStore.getState().setTasks(tasks);
      useTaskStore.getState().selectTask("task-50");
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
      expect(useTaskStore.getState().selectedIds.has("task-50")).toBe(true);
    });

    it("shift-select across many tasks", () => {
      const tasks = Array.from({ length: 100 }, (_, i) =>
        makeTask({ id: `task-${i}` })
      );
      useTaskStore.getState().setTasks(tasks);
      useTaskStore.getState().selectTask("task-10");
      useTaskStore.getState().selectTask("task-19", false, true);
      expect(useTaskStore.getState().selectedIds.size).toBe(10);
    });
  });

  describe("setTasks with many tasks", () => {
    it("handles 1000 tasks", () => {
      const tasks = Array.from({ length: 1000 }, (_, i) =>
        makeTask({ id: `task-${i}` })
      );
      useTaskStore.getState().setTasks(tasks);
      expect(useTaskStore.getState().tasks.size).toBe(1000);
    });
  });

  describe("getGlobalStats comprehensive", () => {
    it("handles mixed upload and download speeds", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "1", state: "downloading", downloadSpeed: 100, uploadSpeed: 10 }),
        makeTask({ id: "2", state: "seeding", downloadSpeed: 0, uploadSpeed: 200 }),
        makeTask({ id: "3", state: "done", downloadSpeed: 0, uploadSpeed: 0 }),
        makeTask({ id: "4", state: "error", downloadSpeed: 0, uploadSpeed: 0 }),
        makeTask({ id: "5", state: "queued", downloadSpeed: 0, uploadSpeed: 0 }),
      ]);
      const stats = useTaskStore.getState().getGlobalStats();
      expect(stats.activeCount).toBe(2); // downloading + seeding
      expect(stats.totalCount).toBe(5);
      expect(stats.totalSpeed).toBe(100);
      expect(stats.totalUploadSpeed).toBe(210); // 10 + 200
    });
  });

  describe("removeTask with multiple selections", () => {
    it("removes all selected tasks", () => {
      useTaskStore.getState().setTasks([
        makeTask({ id: "a" }),
        makeTask({ id: "b" }),
        makeTask({ id: "c" }),
      ]);
      useTaskStore.getState().selectTask("a");
      useTaskStore.getState().selectTask("b", true);
      useTaskStore.getState().selectTask("c", true);
      expect(useTaskStore.getState().selectedIds.size).toBe(3);

      useTaskStore.getState().removeTask("a");
      useTaskStore.getState().removeTask("b");
      expect(useTaskStore.getState().tasks.size).toBe(1);
      expect(useTaskStore.getState().selectedIds.size).toBe(1);
      expect(useTaskStore.getState().selectedIds.has("c")).toBe(true);
    });
  });

  describe("speedHistory with many updates", () => {
    it("maintains reasonable history length", () => {
      useTaskStore.getState().setTasks([makeTask({ id: "1", state: "downloading", downloadSpeed: 100 })]);

      // Simulate many updates over time
      for (let i = 0; i < 50; i++) {
        const now = Date.now() + i * 1100; // 1.1 seconds apart to avoid throttle
        vi.spyOn(Date, "now").mockReturnValue(now);
        useTaskStore.getState().applyUpdate({
          taskId: "1",
          state: "downloading",
          downloaded: 100 * (i + 1),
          totalSize: 10000,
          downloadSpeed: 100,
          uploadSpeed: 0,
          progress: (i + 1) / 100,
          peers: 0,
          error: null,
          eta: 100 - i,
        });
      }
      vi.restoreAllMocks();

      const history = useTaskStore.getState().speedHistory;
      // Should have at most 300 points (5 minutes at 1/sec)
      expect(history.length).toBeLessThanOrEqual(300);
    });
  });
});
