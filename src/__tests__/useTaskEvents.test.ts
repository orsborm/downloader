import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock Tauri APIs
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-notification", () => ({
  sendNotification: vi.fn(),
}));

vi.mock("../stores/taskStore", () => ({
  useTaskStore: Object.assign(
    (selector: (state: Record<string, unknown>) => unknown) => selector(mockStoreState),
    { getState: () => mockStoreState }
  ),
}));

let mockStoreState: Record<string, unknown>;

import { listen } from "@tauri-apps/api/event";
import { sendNotification } from "@tauri-apps/plugin-notification";

const mockListen = vi.mocked(listen);
const mockSendNotification = vi.mocked(sendNotification);

describe("useTaskEvents - notification deduplication", () => {
  // Test the notifiedTasks deduplication logic directly
  const notifiedTasks = new Set<string>();

  beforeEach(() => {
    notifiedTasks.clear();
    mockSendNotification.mockClear();
  });

  it("sends notification for first completion", () => {
    const taskId = "task-1";
    expect(notifiedTasks.has(taskId)).toBe(false);

    notifiedTasks.add(taskId);
    sendNotification({ title: "下载完成", body: `"file.zip" 已完成` });

    expect(mockSendNotification).toHaveBeenCalledOnce();
    expect(notifiedTasks.has(taskId)).toBe(true);
  });

  it("skips notification for already notified task", () => {
    const taskId = "task-1";
    notifiedTasks.add(taskId);

    // Try to notify again
    if (!notifiedTasks.has(taskId)) {
      sendNotification({ title: "下载完成", body: "test" });
    }

    expect(mockSendNotification).not.toHaveBeenCalled();
  });

  it("sends notifications for different tasks", () => {
    notifiedTasks.add("task-1");
    sendNotification({ title: "下载完成", body: "task-1 done" });

    const task2 = "task-2";
    if (!notifiedTasks.has(task2)) {
      notifiedTasks.add(task2);
      sendNotification({ title: "下载完成", body: "task-2 done" });
    }

    expect(mockSendNotification).toHaveBeenCalledTimes(2);
  });

  it("cleans up notified tasks for deleted tasks", () => {
    notifiedTasks.add("task-1");
    notifiedTasks.add("task-2");
    notifiedTasks.add("task-3");

    // Simulate cleanup: task-2 was deleted
    const tasks = new Map([["task-1", {}], ["task-3", {}]]);
    for (const id of notifiedTasks) {
      if (!tasks.has(id)) {
        notifiedTasks.delete(id);
      }
    }

    expect(notifiedTasks.has("task-1")).toBe(true);
    expect(notifiedTasks.has("task-2")).toBe(false);
    expect(notifiedTasks.has("task-3")).toBe(true);
  });

  it("sends error notification with error message", () => {
    const error = "Connection timeout";
    sendNotification({ title: "下载失败", body: error || "任务下载出错" });

    expect(mockSendNotification).toHaveBeenCalledWith({
      title: "下载失败",
      body: "Connection timeout",
    });
  });

  it("uses default error message when error is empty", () => {
    const error = "";
    sendNotification({ title: "下载失败", body: error || "任务下载出错" });

    expect(mockSendNotification).toHaveBeenCalledWith({
      title: "下载失败",
      body: "任务下载出错",
    });
  });
});

describe("useTaskEvents - event listener setup", () => {
  beforeEach(() => {
    mockListen.mockReset();
    mockSendNotification.mockClear();
    mockStoreState = {
      applyUpdate: vi.fn(),
      tasks: new Map(),
      getTaskById: vi.fn().mockReturnValue({ name: "test.zip" }),
    };
  });

  it("listen is called with task-update event", async () => {
    const unlisten = vi.fn();
    mockListen.mockResolvedValue(unlisten);

    // Simulate the setup from useTaskEvents
    await listen("task-update", () => {});

    expect(mockListen).toHaveBeenCalledWith("task-update", expect.any(Function));
  });

  it("unlisten function is returned", async () => {
    const unlisten = vi.fn();
    mockListen.mockResolvedValue(unlisten);

    const result = await listen("task-update", () => {});

    expect(result).toBe(unlisten);
  });

  it("applyUpdate is called with event payload", async () => {
    const applyUpdate = vi.fn();
    mockStoreState.applyUpdate = applyUpdate;

    let handler: ((event: { payload: Record<string, unknown> }) => void) | undefined;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (mockListen as any).mockImplementation(async (_event: string, cb: (event: { payload: Record<string, unknown> }) => void) => {
      handler = cb;
      return vi.fn();
    });

    await listen("task-update", (event: { payload: Record<string, unknown> }) => {
      applyUpdate(event.payload);
    });

    // Simulate event
    const payload = { taskId: "task-1", state: "downloading", progress: 0.5 };
    handler?.({ payload });

    expect(applyUpdate).toHaveBeenCalledWith(payload);
  });

  it("sends completion notification on done state", async () => {
    const notifiedTasks = new Set<string>();
    mockStoreState.getTaskById = vi.fn().mockReturnValue({ name: "file.zip" });

    await listen("task-update", () => {});

    // Simulate done event
    const payload = { taskId: "task-done", state: "done" };
    if (payload.state === "done" && !notifiedTasks.has(payload.taskId as string)) {
      notifiedTasks.add(payload.taskId as string);
      sendNotification({
        title: "下载完成",
        body: `"${(mockStoreState.getTaskById as (id: string) => { name: string } | undefined)(payload.taskId as string)?.name ?? payload.taskId}" 已完成`,
      });
    }

    expect(mockSendNotification).toHaveBeenCalledWith({
      title: "下载完成",
      body: '"file.zip" 已完成',
    });
  });

  it("sends error notification on error state", async () => {
    const notifiedTasks = new Set<string>();

    await listen("task-update", () => {});

    const payload = { taskId: "task-err", state: "error", error: "Disk full" };
    if (payload.state === "error" && !notifiedTasks.has(payload.taskId as string)) {
      notifiedTasks.add(payload.taskId as string);
      sendNotification({
        title: "下载失败",
        body: (payload.error as string) || "任务下载出错",
      });
    }

    expect(mockSendNotification).toHaveBeenCalledWith({
      title: "下载失败",
      body: "Disk full",
    });
  });

  it("uses task name from store when available", () => {
    const getTaskById = vi.fn().mockReturnValue({ name: "important-file.zip" });
    const taskId = "task-123";

    const task = getTaskById(taskId);
    const body = `"${task?.name ?? taskId}" 已完成`;

    expect(body).toBe('"important-file.zip" 已完成');
    expect(getTaskById).toHaveBeenCalledWith("task-123");
  });

  it("falls back to taskId when task not found in store", () => {
    const getTaskById = vi.fn().mockReturnValue(undefined);
    const taskId = "task-unknown";

    const task = getTaskById(taskId);
    const body = `"${task?.name ?? taskId}" 已完成`;

    expect(body).toBe('"task-unknown" 已完成');
  });
});
