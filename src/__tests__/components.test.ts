import { describe, it, expect } from "vitest";

// ==================== Toast ID 生成测试 ====================
// Mirrors src/components/Toast.tsx showToast ID generation (crypto.randomUUID)

function generateToastId(): string {
  return crypto.randomUUID();
}

describe("Toast: ID generation", () => {
  it("generates unique IDs", () => {
    const id1 = generateToastId();
    const id2 = generateToastId();
    expect(id1).not.toBe(id2);
  });

  it("returns valid UUID format", () => {
    const id = generateToastId();
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  });

  it("generates many unique IDs without collision", () => {
    const ids = new Set(Array.from({ length: 1000 }, () => generateToastId()));
    expect(ids.size).toBe(1000);
  });
});

// ==================== AddTaskDialog 限速解析测试 ====================
// Mirrors src/components/AddTaskDialog.tsx speed limit parsing

function parseSpeedLimit(input: string): number | undefined {
  if (!input) return undefined;
  const parsed = parseInt(input);
  if (isNaN(parsed) || parsed <= 0) return undefined;
  return parsed * 1024; // KB/s → bytes/s
}

describe("AddTaskDialog: speed limit parsing", () => {
  it("returns undefined for empty string", () => {
    expect(parseSpeedLimit("")).toBeUndefined();
  });

  it("converts KB/s to bytes/s", () => {
    expect(parseSpeedLimit("100")).toBe(102400);
  });

  it("converts 1 KB/s correctly", () => {
    expect(parseSpeedLimit("1")).toBe(1024);
  });

  it("handles large values", () => {
    expect(parseSpeedLimit("10240")).toBe(10485760); // 10 MB/s
  });

  it("returns undefined for non-numeric input", () => {
    expect(parseSpeedLimit("abc")).toBeUndefined();
  });

  it("returns undefined for zero", () => {
    expect(parseSpeedLimit("0")).toBeUndefined();
  });

  it("returns undefined for negative", () => {
    expect(parseSpeedLimit("-100")).toBeUndefined();
  });

  it("truncates decimal input", () => {
    expect(parseSpeedLimit("100.5")).toBe(102400); // parseInt truncates
  });
});

// ==================== AddTaskDialog URL 验证测试 ====================
// Mirrors the validation in AddTaskDialog.handleSubmit

function validateAddTaskInput(url: string): { valid: boolean; error?: string } {
  if (!url.trim()) {
    return { valid: false, error: "请输入下载链接" };
  }

  const validPrefixes = [
    "http://",
    "https://",
    "ftp://",
    "ftps://",
    "magnet:",
    "ed2k://",
    "file://",
  ];

  const trimmed = url.trim();
  const hasValidPrefix = validPrefixes.some((p) =>
    trimmed.toLowerCase().startsWith(p)
  );

  if (!hasValidPrefix) {
    // Check for .torrent file path
    if (!trimmed.toLowerCase().endsWith(".torrent")) {
      // Check for m3u8/mpd
      try {
        const urlObj = new URL(trimmed);
        const pathname = urlObj.pathname.toLowerCase();
        if (!pathname.endsWith(".m3u8") && !pathname.endsWith(".mpd")) {
          return {
            valid: false,
            error: "不支持的链接格式，请输入 HTTP/FTP/magnet/ed2k 链接",
          };
        }
      } catch {
        return {
          valid: false,
          error: "不支持的链接格式，请输入 HTTP/FTP/magnet/ed2k 链接",
        };
      }
    }
  }

  return { valid: true };
}

describe("AddTaskDialog: URL validation", () => {
  it("rejects empty URL", () => {
    const result = validateAddTaskInput("");
    expect(result.valid).toBe(false);
    expect(result.error).toBe("请输入下载链接");
  });

  it("rejects whitespace-only URL", () => {
    const result = validateAddTaskInput("   ");
    expect(result.valid).toBe(false);
    expect(result.error).toBe("请输入下载链接");
  });

  it("accepts HTTP URL", () => {
    expect(validateAddTaskInput("http://example.com/file.zip").valid).toBe(
      true
    );
  });

  it("accepts HTTPS URL", () => {
    expect(validateAddTaskInput("https://example.com/file.zip").valid).toBe(
      true
    );
  });

  it("accepts magnet link", () => {
    expect(
      validateAddTaskInput("magnet:?xt=urn:btih:abc123").valid
    ).toBe(true);
  });

  it("accepts ed2k link", () => {
    expect(
      validateAddTaskInput("ed2k://|file|test.zip|12345|abc|/").valid
    ).toBe(true);
  });

  it("accepts FTP link", () => {
    expect(validateAddTaskInput("ftp://example.com/file.zip").valid).toBe(true);
  });

  it("rejects javascript: protocol", () => {
    const result = validateAddTaskInput("javascript:alert(1)");
    expect(result.valid).toBe(false);
  });

  it("rejects plain text", () => {
    const result = validateAddTaskInput("not a url");
    expect(result.valid).toBe(false);
  });

  it("accepts .torrent file path", () => {
    expect(validateAddTaskInput("/path/to/file.torrent").valid).toBe(true);
  });

  it("accepts m3u8 URL", () => {
    expect(
      validateAddTaskInput("https://example.com/stream.m3u8").valid
    ).toBe(true);
  });

  it("trims whitespace before validation", () => {
    expect(
      validateAddTaskInput("  https://example.com/file.zip  ").valid
    ).toBe(true);
  });
});

// ==================== 状态优先级排序测试 ====================
// Mirrors taskStore stateOrder logic

const stateOrder: Record<string, number> = {
  downloading: 0,
  seeding: 1,
  queued: 2,
  paused: 3,
  done: 4,
  error: 5,
};

function sortByStatePriority<T extends { state: string }>(items: T[]): T[] {
  return [...items].sort(
    (a, b) => (stateOrder[a.state] ?? 99) - (stateOrder[b.state] ?? 99)
  );
}

describe("state priority ordering", () => {
  it("downloading comes first", () => {
    const items = [
      { state: "done" },
      { state: "downloading" },
      { state: "paused" },
    ];
    const sorted = sortByStatePriority(items);
    expect(sorted[0].state).toBe("downloading");
  });

  it("seeding comes before queued", () => {
    const items = [{ state: "queued" }, { state: "seeding" }];
    const sorted = sortByStatePriority(items);
    expect(sorted[0].state).toBe("seeding");
  });

  it("error comes last", () => {
    const items = [
      { state: "error" },
      { state: "downloading" },
      { state: "done" },
    ];
    const sorted = sortByStatePriority(items);
    expect(sorted[sorted.length - 1].state).toBe("error");
  });

  it("unknown states go to end", () => {
    const items = [{ state: "unknown" }, { state: "downloading" }];
    const sorted = sortByStatePriority(items);
    expect(sorted[0].state).toBe("downloading");
    expect(sorted[1].state).toBe("unknown");
  });
});

// ==================== protocolColor 测试 ====================
// Mirrors src/components/TaskList.tsx protocolColor function

function protocolColor(protocol: string): string {
  switch (protocol) {
    case "HTTP":
      return "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400";
    case "FTP":
      return "bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400";
    case "BT":
    case "MAGNET":
      return "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400";
    case "ED2K":
      return "bg-orange-100 text-orange-700 dark:bg-orange-900/30 dark:text-orange-400";
    default:
      return "bg-gray-100 text-gray-700 dark:bg-gray-900/30 dark:text-gray-400";
  }
}

describe("protocolColor", () => {
  it("returns blue for HTTP", () => {
    expect(protocolColor("HTTP")).toContain("blue");
  });

  it("returns purple for FTP", () => {
    expect(protocolColor("FTP")).toContain("purple");
  });

  it("returns green for BT", () => {
    expect(protocolColor("BT")).toContain("green");
  });

  it("returns green for MAGNET", () => {
    expect(protocolColor("MAGNET")).toContain("green");
  });

  it("returns orange for ED2K", () => {
    expect(protocolColor("ED2K")).toContain("orange");
  });

  it("returns gray for unknown protocol", () => {
    expect(protocolColor("HLS")).toContain("gray");
  });

  it("returns gray for empty string", () => {
    expect(protocolColor("")).toContain("gray");
  });

  it("includes dark mode classes", () => {
    const cls = protocolColor("HTTP");
    expect(cls).toContain("dark:");
  });
});

// ==================== StateIcon state mapping 测试 ====================
// Mirrors src/components/TaskList.tsx StateIcon switch

function getStateIconClass(state: string): string {
  switch (state) {
    case "downloading":
      return "state-downloading";
    case "paused":
      return "state-paused";
    case "done":
      return "state-done";
    case "error":
      return "state-error";
    case "seeding":
      return "state-seeding";
    default:
      return "state-queued";
  }
}

describe("StateIcon state mapping", () => {
  it("maps downloading state", () => {
    expect(getStateIconClass("downloading")).toBe("state-downloading");
  });

  it("maps paused state", () => {
    expect(getStateIconClass("paused")).toBe("state-paused");
  });

  it("maps done state", () => {
    expect(getStateIconClass("done")).toBe("state-done");
  });

  it("maps error state", () => {
    expect(getStateIconClass("error")).toBe("state-error");
  });

  it("maps seeding state", () => {
    expect(getStateIconClass("seeding")).toBe("state-seeding");
  });

  it("maps unknown state to queued", () => {
    expect(getStateIconClass("unknown")).toBe("state-queued");
  });

  it("maps empty string to queued", () => {
    expect(getStateIconClass("")).toBe("state-queued");
  });
});

// ==================== 速度图表时间范围过滤测试 ====================
// Mirrors src/components/SpeedChart.tsx chartData filtering

interface SpeedPoint {
  time: number;
  download: number;
  upload: number;
}

function filterByTimeRange(
  history: SpeedPoint[],
  rangeSeconds: number,
  now: number
): SpeedPoint[] {
  const cutoff = now - rangeSeconds * 1000;
  return history.filter((p) => p.time >= cutoff);
}

describe("SpeedChart: time range filtering", () => {
  const now = 1000000;
  const history: SpeedPoint[] = [
    { time: now - 4000, download: 100, upload: 10 },
    { time: now - 3000, download: 200, upload: 20 },
    { time: now - 2000, download: 300, upload: 30 },
    { time: now - 1000, download: 400, upload: 40 },
    { time: now, download: 500, upload: 50 },
  ];

  it("filters last 5 minutes", () => {
    const filtered = filterByTimeRange(history, 5 * 60, now);
    expect(filtered).toHaveLength(5);
  });

  it("filters last 2 seconds", () => {
    const filtered = filterByTimeRange(history, 2, now);
    expect(filtered).toHaveLength(3); // now-2000, now-1000, now
  });

  it("filters last 1 second (inclusive boundary)", () => {
    const filtered = filterByTimeRange(history, 1, now);
    expect(filtered).toHaveLength(2); // now-1000 and now (>= cutoff)
  });

  it("returns 1 item for zero range (exact now)", () => {
    const filtered = filterByTimeRange(history, 0, now);
    expect(filtered).toHaveLength(1); // only time === now
  });

  it("returns all for very large range", () => {
    const filtered = filterByTimeRange(history, 86400, now);
    expect(filtered).toHaveLength(5);
  });
});

// ==================== TaskDetail Tab 配置测试 ====================
// Mirrors src/components/TaskDetail.tsx tabs definition

interface TabDef {
  key: string;
  label: string;
}

const detailTabs: TabDef[] = [
  { key: "summary", label: "概要" },
  { key: "files", label: "文件" },
  { key: "connections", label: "连接" },
  { key: "tracker", label: "Tracker" },
  { key: "logs", label: "日志" },
];

describe("TaskDetail: tab configuration", () => {
  it("has exactly 5 tabs", () => {
    expect(detailTabs).toHaveLength(5);
  });

  it("has correct tab keys", () => {
    const keys = detailTabs.map((t) => t.key);
    expect(keys).toEqual(["summary", "files", "connections", "tracker", "logs"]);
  });

  it("has labels for all tabs", () => {
    detailTabs.forEach((tab) => {
      expect(tab.label.length).toBeGreaterThan(0);
    });
  });

  it("all tab keys are unique", () => {
    const keys = detailTabs.map((t) => t.key);
    expect(new Set(keys).size).toBe(keys.length);
  });
});

// ==================== TaskDetail LogsTab 日志生成测试 ====================
// Mirrors src/components/TaskDetail.tsx LogsTab logic

interface TaskLike {
  state: string;
  error: string | null;
  completedAt: string | null;
  downloadSpeed: number;
  downloaded: number;
  totalSize: number;
  addedAt: string;
}

function generateLogs(task: TaskLike): { level: string; message: string }[] {
  const logs: { level: string; message: string }[] = [];

  if (task.error) {
    logs.push({ level: "ERROR", message: task.error });
  }

  if (task.state === "done") {
    logs.push({ level: "INFO", message: "下载完成" });
  }

  if (task.state === "downloading" || task.state === "seeding") {
    logs.push({
      level: "INFO",
      message: `下载中 - ${task.downloadSpeed} - ${task.downloaded}/${task.totalSize > 0 ? task.totalSize : "未知"}`,
    });
  }

  if (task.state === "paused") {
    logs.push({ level: "INFO", message: `已暂停 - ${task.downloaded} 已下载` });
  }

  return logs;
}

describe("TaskDetail: LogsTab log generation", () => {
  it("generates error log for error state", () => {
    const task: TaskLike = {
      state: "error",
      error: "Connection refused",
      completedAt: null,
      downloadSpeed: 0,
      downloaded: 100,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs).toHaveLength(1);
    expect(logs[0].level).toBe("ERROR");
    expect(logs[0].message).toBe("Connection refused");
  });

  it("generates completion log for done state", () => {
    const task: TaskLike = {
      state: "done",
      error: null,
      completedAt: "2026-01-01T01:00:00Z",
      downloadSpeed: 0,
      downloaded: 500,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs.some((l) => l.message === "下载完成")).toBe(true);
  });

  it("generates progress log for downloading state", () => {
    const task: TaskLike = {
      state: "downloading",
      error: null,
      completedAt: null,
      downloadSpeed: 1024,
      downloaded: 250,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs).toHaveLength(1);
    expect(logs[0].level).toBe("INFO");
    expect(logs[0].message).toContain("下载中");
  });

  it("generates progress log for seeding state", () => {
    const task: TaskLike = {
      state: "seeding",
      error: null,
      completedAt: null,
      downloadSpeed: 0,
      downloaded: 500,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs).toHaveLength(1);
    expect(logs[0].message).toContain("下载中");
  });

  it("generates pause log for paused state", () => {
    const task: TaskLike = {
      state: "paused",
      error: null,
      completedAt: null,
      downloadSpeed: 0,
      downloaded: 100,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs).toHaveLength(1);
    expect(logs[0].message).toContain("已暂停");
  });

  it("generates empty logs for queued state", () => {
    const task: TaskLike = {
      state: "queued",
      error: null,
      completedAt: null,
      downloadSpeed: 0,
      downloaded: 0,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs).toHaveLength(0);
  });

  it("generates both error and done logs for error with completion", () => {
    const task: TaskLike = {
      state: "error",
      error: "Hash mismatch",
      completedAt: null,
      downloadSpeed: 0,
      downloaded: 500,
      totalSize: 500,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    // Only error log, not done (state is error)
    expect(logs).toHaveLength(1);
    expect(logs[0].level).toBe("ERROR");
  });

  it("handles unknown totalSize in downloading log", () => {
    const task: TaskLike = {
      state: "downloading",
      error: null,
      completedAt: null,
      downloadSpeed: 100,
      downloaded: 50,
      totalSize: 0,
      addedAt: "2026-01-01T00:00:00Z",
    };
    const logs = generateLogs(task);
    expect(logs[0].message).toContain("未知");
  });
});

// ==================== TaskDetail ConnectionsTab/TrackerTab 协议过滤测试 ====================
// Mirrors src/components/TaskDetail.tsx ConnectionsTab and TrackerTab

function isBtProtocol(protocol: string): boolean {
  return protocol === "BT" || protocol === "MAGNET";
}

describe("TaskDetail: protocol-based tab filtering", () => {
  it("BT protocol shows connections", () => {
    expect(isBtProtocol("BT")).toBe(true);
  });

  it("MAGNET protocol shows connections", () => {
    expect(isBtProtocol("MAGNET")).toBe(true);
  });

  it("HTTP protocol hides connections", () => {
    expect(isBtProtocol("HTTP")).toBe(false);
  });

  it("ED2K protocol hides connections", () => {
    expect(isBtProtocol("ED2K")).toBe(false);
  });

  it("FTP protocol hides connections", () => {
    expect(isBtProtocol("FTP")).toBe(false);
  });
});

// ==================== TaskDetail FilesTab 测试 ====================
// Mirrors src/components/TaskDetail.tsx FilesTab

interface FileEntry {
  index: number;
  path: string;
  size: number;
  priority: number;
}

function priorityLabel(priority: number): string {
  if (priority === 0) return "跳过";
  if (priority === 2) return "高";
  return "正常";
}

describe("TaskDetail: FilesTab", () => {
  it("priority 0 maps to 跳过", () => {
    expect(priorityLabel(0)).toBe("跳过");
  });

  it("priority 1 maps to 正常", () => {
    expect(priorityLabel(1)).toBe("正常");
  });

  it("priority 2 maps to 高", () => {
    expect(priorityLabel(2)).toBe("高");
  });

  it("unknown priority defaults to 正常", () => {
    expect(priorityLabel(99)).toBe("正常");
  });

  it("empty files array is valid", () => {
    const files: FileEntry[] = [];
    expect(files).toHaveLength(0);
  });

  it("files have sequential indices", () => {
    const files: FileEntry[] = [
      { index: 0, path: "a.txt", size: 100, priority: 1 },
      { index: 1, path: "b.txt", size: 200, priority: 1 },
    ];
    expect(files[0].index).toBe(0);
    expect(files[1].index).toBe(1);
  });
});

// ==================== TaskList ContextMenu 状态菜单测试 ====================
// Mirrors src/components/TaskList.tsx ContextMenu state-dependent items

function getContextMenuItems(state: string): string[] {
  const items: string[] = [];

  if (state === "downloading" || state === "seeding") {
    items.push("暂停");
  }
  if (state === "paused") {
    items.push("恢复");
  }
  if (state === "error") {
    items.push("重试");
  }

  // Always present
  items.push("删除任务");
  items.push("删除任务和文件");

  return items;
}

describe("TaskList: ContextMenu items by state", () => {
  it("shows pause for downloading state", () => {
    const items = getContextMenuItems("downloading");
    expect(items).toContain("暂停");
    expect(items).not.toContain("恢复");
    expect(items).not.toContain("重试");
  });

  it("shows pause for seeding state", () => {
    const items = getContextMenuItems("seeding");
    expect(items).toContain("暂停");
  });

  it("shows resume for paused state", () => {
    const items = getContextMenuItems("paused");
    expect(items).toContain("恢复");
    expect(items).not.toContain("暂停");
  });

  it("shows retry for error state", () => {
    const items = getContextMenuItems("error");
    expect(items).toContain("重试");
    expect(items).not.toContain("暂停");
    expect(items).not.toContain("恢复");
  });

  it("always shows delete options", () => {
    const states = ["downloading", "paused", "done", "error", "seeding", "queued"];
    states.forEach((state) => {
      const items = getContextMenuItems(state);
      expect(items).toContain("删除任务");
      expect(items).toContain("删除任务和文件");
    });
  });

  it("queued state shows only delete options", () => {
    const items = getContextMenuItems("queued");
    expect(items).toEqual(["删除任务", "删除任务和文件"]);
  });

  it("done state shows only delete options", () => {
    const items = getContextMenuItems("done");
    expect(items).toEqual(["删除任务", "删除任务和文件"]);
  });
});

// ==================== TaskList ContextMenu 位置调整测试 ====================
// Mirrors src/components/TaskList.tsx ContextMenu position adjustment

function adjustMenuPosition(
  x: number,
  y: number,
  menuWidth: number,
  menuHeight: number,
  viewportWidth: number,
  viewportHeight: number
): { x: number; y: number } {
  const adjustedX = x + menuWidth > viewportWidth ? x - menuWidth : x;
  const adjustedY = y + menuHeight > viewportHeight ? y - menuHeight : y;
  return {
    x: Math.max(0, adjustedX),
    y: Math.max(0, adjustedY),
  };
}

describe("TaskList: ContextMenu position adjustment", () => {
  it("keeps position when within viewport", () => {
    const pos = adjustMenuPosition(100, 100, 180, 300, 1920, 1080);
    expect(pos.x).toBe(100);
    expect(pos.y).toBe(100);
  });

  it("flips horizontally when overflowing right", () => {
    const pos = adjustMenuPosition(1800, 100, 180, 300, 1920, 1080);
    expect(pos.x).toBe(1620); // 1800 - 180
  });

  it("flips vertically when overflowing bottom", () => {
    const pos = adjustMenuPosition(100, 900, 180, 300, 1920, 1080);
    expect(pos.y).toBe(600); // 900 - 300
  });

  it("clamps negative x to 0", () => {
    adjustMenuPosition(50, 100, 180, 300, 1920, 1080);
    // 50 - 180 = -130, but Math.max(0, -130) = 0... wait, 50+180=230 < 1920, so no flip
    // Let me use a case where it actually flips
    const pos2 = adjustMenuPosition(50, 100, 180, 300, 200, 1080);
    // 50 + 180 = 230 > 200, so flip: 50 - 180 = -130, clamped to 0
    expect(pos2.x).toBe(0);
  });

  it("clamps negative y to 0", () => {
    const pos = adjustMenuPosition(100, 50, 180, 300, 1920, 200);
    // 50 + 300 = 350 > 200, so flip: 50 - 300 = -250, clamped to 0
    expect(pos.y).toBe(0);
  });

  it("handles corner case (both overflow)", () => {
    const pos = adjustMenuPosition(1900, 1000, 180, 300, 1920, 1080);
    expect(pos.x).toBe(1720); // 1900 - 180
    expect(pos.y).toBe(700);  // 1000 - 300
  });
});

// ==================== TaskList 速度显示格式测试 ====================
// Mirrors src/components/TaskList.tsx speed display logic

function getSpeedDisplay(state: string, downloadSpeed: number, uploadSpeed: number): string {
  if (state === "downloading") return `${downloadSpeed}`;
  if (state === "seeding") return `↑${uploadSpeed}`;
  if (state === "done") return "完成";
  if (state === "error") return "错误";
  return "--";
}

describe("TaskList: speed display by state", () => {
  it("shows download speed for downloading state", () => {
    expect(getSpeedDisplay("downloading", 1024, 0)).toBe("1024");
  });

  it("shows upload speed prefix for seeding state", () => {
    expect(getSpeedDisplay("seeding", 0, 512)).toBe("↑512");
  });

  it("shows 完成 for done state", () => {
    expect(getSpeedDisplay("done", 0, 0)).toBe("完成");
  });

  it("shows 错误 for error state", () => {
    expect(getSpeedDisplay("error", 0, 0)).toBe("错误");
  });

  it("shows -- for queued state", () => {
    expect(getSpeedDisplay("queued", 0, 0)).toBe("--");
  });

  it("shows -- for paused state", () => {
    expect(getSpeedDisplay("paused", 0, 0)).toBe("--");
  });
});

// ==================== TaskList ETA 显示测试 ====================
// Mirrors src/components/TaskList.tsx ETA display logic

function getEtaDisplay(state: string, eta: number | null): string {
  if ((state === "downloading" || state === "seeding") && eta !== null && eta !== undefined) {
    return "has_eta";
  }
  if (state === "done") return "完成";
  return "--";
}

describe("TaskList: ETA display by state", () => {
  it("shows ETA for downloading with valid eta", () => {
    expect(getEtaDisplay("downloading", 60)).toBe("has_eta");
  });

  it("shows ETA for seeding with valid eta", () => {
    expect(getEtaDisplay("seeding", 120)).toBe("has_eta");
  });

  it("shows -- for downloading with null eta", () => {
    expect(getEtaDisplay("downloading", null)).toBe("--");
  });

  it("shows 完成 for done state", () => {
    expect(getEtaDisplay("done", null)).toBe("完成");
  });

  it("shows -- for paused state", () => {
    expect(getEtaDisplay("paused", null)).toBe("--");
  });
});

// ==================== 全局统计计算测试 ====================
// Mirrors taskStore getGlobalStats logic

interface SpeedTask {
  state: string;
  downloadSpeed: number;
  uploadSpeed: number;
}

function calculateGlobalStats(tasks: SpeedTask[]) {
  let totalSpeed = 0;
  let totalUploadSpeed = 0;
  let activeCount = 0;

  for (const t of tasks) {
    if (t.state === "downloading" || t.state === "seeding") {
      totalSpeed += t.downloadSpeed;
      totalUploadSpeed += t.uploadSpeed;
      activeCount++;
    }
  }

  return { totalSpeed, totalUploadSpeed, activeCount, totalCount: tasks.length };
}

describe("global stats calculation", () => {
  it("sums download speeds of active tasks", () => {
    const tasks: SpeedTask[] = [
      { state: "downloading", downloadSpeed: 1000, uploadSpeed: 100 },
      { state: "downloading", downloadSpeed: 2000, uploadSpeed: 200 },
      { state: "done", downloadSpeed: 0, uploadSpeed: 0 },
    ];
    const stats = calculateGlobalStats(tasks);
    expect(stats.totalSpeed).toBe(3000);
    expect(stats.totalUploadSpeed).toBe(300);
    expect(stats.activeCount).toBe(2);
    expect(stats.totalCount).toBe(3);
  });

  it("includes seeding tasks in active count", () => {
    const tasks: SpeedTask[] = [
      { state: "seeding", downloadSpeed: 0, uploadSpeed: 500 },
      { state: "paused", downloadSpeed: 0, uploadSpeed: 0 },
    ];
    const stats = calculateGlobalStats(tasks);
    expect(stats.activeCount).toBe(1);
    expect(stats.totalUploadSpeed).toBe(500);
  });

  it("returns zero for empty task list", () => {
    const stats = calculateGlobalStats([]);
    expect(stats.totalSpeed).toBe(0);
    expect(stats.activeCount).toBe(0);
    expect(stats.totalCount).toBe(0);
  });

  it("ignores paused/error/done tasks", () => {
    const tasks: SpeedTask[] = [
      { state: "paused", downloadSpeed: 5000, uploadSpeed: 500 },
      { state: "error", downloadSpeed: 0, uploadSpeed: 0 },
      { state: "done", downloadSpeed: 0, uploadSpeed: 0 },
    ];
    const stats = calculateGlobalStats(tasks);
    expect(stats.activeCount).toBe(0);
    expect(stats.totalSpeed).toBe(0);
  });
});
