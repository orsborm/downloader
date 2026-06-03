import { describe, it, expect } from "vitest";

// ==================== WebUI statusLabel 测试 ====================
// Mirrors src-webui/App.tsx statusLabel function

function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    active: "下载中",
    downloading: "下载中",
    paused: "已暂停",
    waiting: "等待中",
    completed: "已完成",
    error: "出错",
    seeding: "做种中",
  };
  return labels[status] || status;
}

describe("WebUI: statusLabel", () => {
  it("labels active status", () => {
    expect(statusLabel("active")).toBe("下载中");
  });

  it("labels downloading status", () => {
    expect(statusLabel("downloading")).toBe("下载中");
  });

  it("labels paused status", () => {
    expect(statusLabel("paused")).toBe("已暂停");
  });

  it("labels waiting status", () => {
    expect(statusLabel("waiting")).toBe("等待中");
  });

  it("labels completed status", () => {
    expect(statusLabel("completed")).toBe("已完成");
  });

  it("labels error status", () => {
    expect(statusLabel("error")).toBe("出错");
  });

  it("labels seeding status", () => {
    expect(statusLabel("seeding")).toBe("做种中");
  });

  it("returns raw status for unknown", () => {
    expect(statusLabel("unknown")).toBe("unknown");
  });

  it("returns empty string for empty input", () => {
    expect(statusLabel("")).toBe("");
  });
});

// ==================== WebUI StatusIcon 颜色映射测试 ====================
// Mirrors src-webui/App.tsx StatusIcon color mapping

function getStatusColor(status: string): string {
  const colors: Record<string, string> = {
    active: "bg-blue-500",
    downloading: "bg-blue-500",
    paused: "bg-yellow-500",
    waiting: "bg-slate-400",
    completed: "bg-green-500",
    error: "bg-red-500",
    seeding: "bg-purple-500",
  };
  return colors[status] || "bg-slate-300";
}

describe("WebUI: StatusIcon colors", () => {
  it("active is blue", () => {
    expect(getStatusColor("active")).toBe("bg-blue-500");
  });

  it("downloading is blue", () => {
    expect(getStatusColor("downloading")).toBe("bg-blue-500");
  });

  it("paused is yellow", () => {
    expect(getStatusColor("paused")).toBe("bg-yellow-500");
  });

  it("completed is green", () => {
    expect(getStatusColor("completed")).toBe("bg-green-500");
  });

  it("error is red", () => {
    expect(getStatusColor("error")).toBe("bg-red-500");
  });

  it("seeding is purple", () => {
    expect(getStatusColor("seeding")).toBe("bg-purple-500");
  });

  it("unknown falls back to slate", () => {
    expect(getStatusColor("unknown")).toBe("bg-slate-300");
  });
});

// ==================== WebUI progress bar color test ====================

function getProgressColor(
  _status: string,
  isError: boolean,
  isCompleted: boolean,
  isPaused: boolean
): string {
  if (isError) return "bg-red-500";
  if (isCompleted) return "bg-green-500";
  if (isPaused) return "bg-yellow-500";
  return "bg-blue-500";
}

describe("WebUI: progress bar colors", () => {
  it("red for error", () => {
    expect(getProgressColor("error", true, false, false)).toBe("bg-red-500");
  });

  it("green for completed", () => {
    expect(getProgressColor("completed", false, true, false)).toBe(
      "bg-green-500"
    );
  });

  it("yellow for paused", () => {
    expect(getProgressColor("paused", false, false, true)).toBe(
      "bg-yellow-500"
    );
  });

  it("blue for active", () => {
    expect(getProgressColor("active", false, false, false)).toBe(
      "bg-blue-500"
    );
  });
});

// ==================== WebUI task action visibility test ====================

function getAvailableActions(status: string): string[] {
  const actions: string[] = [];
  const isActive = status === "active" || status === "downloading";
  const isPaused = status === "paused";
  const isError = status === "error";

  if (isActive) actions.push("pause");
  if (isPaused || isError) actions.push("resume");
  actions.push("delete");
  return actions;
}

describe("WebUI: task action visibility", () => {
  it("active tasks show pause and delete", () => {
    expect(getAvailableActions("active")).toEqual(["pause", "delete"]);
  });

  it("downloading tasks show pause and delete", () => {
    expect(getAvailableActions("downloading")).toEqual(["pause", "delete"]);
  });

  it("paused tasks show resume and delete", () => {
    expect(getAvailableActions("paused")).toEqual(["resume", "delete"]);
  });

  it("error tasks show resume and delete", () => {
    expect(getAvailableActions("error")).toEqual(["resume", "delete"]);
  });

  it("completed tasks show only delete", () => {
    expect(getAvailableActions("completed")).toEqual(["delete"]);
  });

  it("seeding tasks show only delete", () => {
    expect(getAvailableActions("seeding")).toEqual(["delete"]);
  });
});

// ==================== WebUI tab configuration test ====================

interface TabInfo {
  key: string;
  label: string;
  count: number;
}

function buildTabs(globalStat: {
  numActive: number;
  numWaiting: number;
  numStopped: number;
}): TabInfo[] {
  return [
    { key: "active", label: "下载中", count: globalStat.numActive },
    { key: "waiting", label: "等待中", count: globalStat.numWaiting },
    { key: "stopped", label: "已完成", count: globalStat.numStopped },
  ];
}

describe("WebUI: tab configuration", () => {
  it("builds correct tabs from global stat", () => {
    const tabs = buildTabs({ numActive: 3, numWaiting: 5, numStopped: 12 });
    expect(tabs).toHaveLength(3);
    expect(tabs[0]).toEqual({ key: "active", label: "下载中", count: 3 });
    expect(tabs[1]).toEqual({ key: "waiting", label: "等待中", count: 5 });
    expect(tabs[2]).toEqual({ key: "stopped", label: "已完成", count: 12 });
  });

  it("handles zero counts", () => {
    const tabs = buildTabs({ numActive: 0, numWaiting: 0, numStopped: 0 });
    expect(tabs.every((t) => t.count === 0)).toBe(true);
  });

  it("handles large counts", () => {
    const tabs = buildTabs({
      numActive: 999,
      numWaiting: 1000,
      numStopped: 50000,
    });
    expect(tabs[0].count).toBe(999);
    expect(tabs[1].count).toBe(1000);
    expect(tabs[2].count).toBe(50000);
  });
});

// ==================== WebUI empty state messages test ====================

function getEmptyMessage(connected: boolean, activeTab: string): string {
  if (!connected) return "连接中...";
  switch (activeTab) {
    case "active":
      return "暂无下载中的任务";
    case "waiting":
      return "暂无等待中的任务";
    case "stopped":
      return "暂无已完成的任务";
    default:
      return "暂无任务";
  }
}

describe("WebUI: empty state messages", () => {
  it("shows connecting when not connected", () => {
    expect(getEmptyMessage(false, "active")).toBe("连接中...");
  });

  it("shows correct message for active tab", () => {
    expect(getEmptyMessage(true, "active")).toBe("暂无下载中的任务");
  });

  it("shows correct message for waiting tab", () => {
    expect(getEmptyMessage(true, "waiting")).toBe("暂无等待中的任务");
  });

  it("shows correct message for stopped tab", () => {
    expect(getEmptyMessage(true, "stopped")).toBe("暂无已完成的任务");
  });

  it("shows default message for unknown tab", () => {
    expect(getEmptyMessage(true, "unknown")).toBe("暂无任务");
  });
});

// ==================== WebUI task name display test ====================

function getDisplayName(name: string, taskId: string): string {
  return name || taskId.slice(0, 8);
}

describe("WebUI: task name display", () => {
  it("shows name when available", () => {
    expect(getDisplayName("movie.mp4", "abc123")).toBe("movie.mp4");
  });

  it("falls back to task ID prefix when name is empty", () => {
    expect(getDisplayName("", "abc123456789")).toBe("abc12345");
  });

  it("falls back to task ID prefix when name is empty string", () => {
    expect(getDisplayName("", "xyz")).toBe("xyz");
  });
});

// ==================== WebUI detail row test ====================

function formatDetailValue(label: string, value: string | number): string {
  if (label === "任务 ID" && typeof value === "string" && value.length > 12) {
    return value.slice(0, 12) + "...";
  }
  if (label === "剩余时间" && typeof value === "number") {
    return value > 0 ? `${value}s` : "-";
  }
  return String(value);
}

describe("WebUI: detail value formatting", () => {
  it("truncates long task ID", () => {
    const id = "abc1234567890123456789";
    expect(formatDetailValue("任务 ID", id)).toBe("abc123456789...");
  });

  it("keeps short task ID as-is", () => {
    expect(formatDetailValue("任务 ID", "abc123")).toBe("abc123");
  });

  it("formats positive ETA", () => {
    expect(formatDetailValue("剩余时间", 120)).toBe("120s");
  });

  it("shows dash for zero ETA", () => {
    expect(formatDetailValue("剩余时间", 0)).toBe("-");
  });

  it("shows dash for negative ETA", () => {
    expect(formatDetailValue("剩余时间", -1)).toBe("-");
  });

  it("converts number to string for other labels", () => {
    expect(formatDetailValue("连接数", 42)).toBe("42");
  });
});
