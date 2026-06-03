import { describe, it, expect } from "vitest";

// Extract detectDownloadUrl logic for testing (mirrors useClipboard.ts)
function detectDownloadUrl(text: string): string | null {
  const trimmed = text.trim();
  const patterns = [
    /^magnet:\?xt=urn:/i,
    /^ed2k:\/\//i,
    /^https?:\/\/.+/i,
    /^ftp:\/\//i,
  ];
  for (const pattern of patterns) {
    if (pattern.test(trimmed)) {
      return trimmed;
    }
  }
  return null;
}

describe("detectDownloadUrl", () => {
  it("detects magnet links", () => {
    expect(detectDownloadUrl("magnet:?xt=urn:btih:abc123&dn=test")).toBe(
      "magnet:?xt=urn:btih:abc123&dn=test"
    );
  });

  it("detects ed2k links", () => {
    expect(detectDownloadUrl("ed2k://|file|test.zip|12345|abcdef|/")).toBe(
      "ed2k://|file|test.zip|12345|abcdef|/"
    );
  });

  it("detects HTTP links", () => {
    expect(detectDownloadUrl("http://example.com/file.zip")).toBe(
      "http://example.com/file.zip"
    );
  });

  it("detects HTTPS links", () => {
    expect(detectDownloadUrl("https://example.com/file.zip")).toBe(
      "https://example.com/file.zip"
    );
  });

  it("detects FTP links", () => {
    expect(detectDownloadUrl("ftp://example.com/file.zip")).toBe(
      "ftp://example.com/file.zip"
    );
  });

  it("returns null for non-URL text", () => {
    expect(detectDownloadUrl("hello world")).toBeNull();
  });

  it("returns null for empty string", () => {
    expect(detectDownloadUrl("")).toBeNull();
  });

  it("trims whitespace before detection", () => {
    expect(detectDownloadUrl("  https://example.com/file.zip  ")).toBe(
      "https://example.com/file.zip"
    );
  });

  it("handles case-insensitive magnet", () => {
    expect(detectDownloadUrl("MAGNET:?xt=urn:btih:abc123")).toBe(
      "MAGNET:?xt=urn:btih:abc123"
    );
  });

  it("handles case-insensitive ed2k", () => {
    expect(detectDownloadUrl("ED2K://|file|test.zip|12345|abcdef|/")).toBe(
      "ED2K://|file|test.zip|12345|abcdef|/"
    );
  });

  it("returns null for javascript protocol", () => {
    expect(detectDownloadUrl("javascript:alert(1)")).toBeNull();
  });

  it("returns null for data URI", () => {
    expect(detectDownloadUrl("data:text/html,<h1>test</h1>")).toBeNull();
  });

  it("detects URLs with complex query strings", () => {
    const url =
      "https://example.com/download?token=abc&expires=1234567890";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("detects magnet with multiple params", () => {
    const magnet =
      "magnet:?xt=urn:btih:abc123&dn=test.zip&tr=udp://tracker.example.com:80";
    expect(detectDownloadUrl(magnet)).toBe(magnet);
  });
});

// Test task state transition logic (mirrors taskManager behavior)
describe("task state transitions", () => {
  type TaskState =
    | "queued"
    | "downloading"
    | "paused"
    | "seeding"
    | "done"
    | "error";

  const validTransitions: Record<TaskState, TaskState[]> = {
    queued: ["downloading", "error"],
    downloading: ["paused", "done", "error", "seeding"],
    paused: ["downloading", "error"],
    seeding: ["paused", "done", "error"],
    done: [],
    error: ["queued", "downloading"],
  };

  function canTransition(from: TaskState, to: TaskState): boolean {
    return validTransitions[from]?.includes(to) ?? false;
  }

  it("allows queued -> downloading", () => {
    expect(canTransition("queued", "downloading")).toBe(true);
  });

  it("allows downloading -> paused", () => {
    expect(canTransition("downloading", "paused")).toBe(true);
  });

  it("allows downloading -> done", () => {
    expect(canTransition("downloading", "done")).toBe(true);
  });

  it("allows downloading -> error", () => {
    expect(canTransition("downloading", "error")).toBe(true);
  });

  it("allows paused -> downloading (resume)", () => {
    expect(canTransition("paused", "downloading")).toBe(true);
  });

  it("allows error -> downloading (retry)", () => {
    expect(canTransition("error", "downloading")).toBe(true);
  });

  it("disallows done -> downloading", () => {
    expect(canTransition("done", "downloading")).toBe(false);
  });

  it("disallows done -> paused", () => {
    expect(canTransition("done", "paused")).toBe(false);
  });

  it("disallows queued -> done", () => {
    expect(canTransition("queued", "done")).toBe(false);
  });

  it("disallows seeding -> queued", () => {
    expect(canTransition("seeding", "queued")).toBe(false);
  });

  it("allows downloading -> seeding (BT upload)", () => {
    expect(canTransition("downloading", "seeding")).toBe(true);
  });

  it("allows seeding -> done (seed ratio reached)", () => {
    expect(canTransition("seeding", "done")).toBe(true);
  });
});

// Test progress calculation logic
describe("progress calculation", () => {
  function calcProgress(downloaded: number, total: number): number {
    if (total <= 0) return 0;
    return Math.min(downloaded / total, 1);
  }

  it("returns 0 for zero total", () => {
    expect(calcProgress(0, 0)).toBe(0);
  });

  it("returns 0 for zero downloaded", () => {
    expect(calcProgress(0, 1024)).toBe(0);
  });

  it("returns 1 for complete download", () => {
    expect(calcProgress(1024, 1024)).toBe(1);
  });

  it("returns correct fraction", () => {
    expect(calcProgress(512, 1024)).toBeCloseTo(0.5);
  });

  it("caps at 1 even if downloaded exceeds total", () => {
    expect(calcProgress(2048, 1024)).toBe(1);
  });

  it("handles negative total gracefully", () => {
    expect(calcProgress(100, -100)).toBe(0);
  });
});

// Test ETA calculation logic
describe("ETA calculation", () => {
  function calcEta(
    downloaded: number,
    total: number,
    speed: number
  ): number | null {
    if (speed <= 0 || total <= 0 || downloaded >= total) return null;
    return (total - downloaded) / speed;
  }

  it("returns null for zero speed", () => {
    expect(calcEta(0, 1024, 0)).toBeNull();
  });

  it("returns null for zero total", () => {
    expect(calcEta(0, 0, 100)).toBeNull();
  });

  it("returns null when download is complete", () => {
    expect(calcEta(1024, 1024, 100)).toBeNull();
  });

  it("calculates correct ETA", () => {
    expect(calcEta(0, 1024, 100)).toBeCloseTo(10.24);
  });

  it("handles large files", () => {
    const oneGB = 1024 * 1024 * 1024;
    const tenMB = 10 * 1024 * 1024;
    expect(calcEta(0, oneGB, tenMB)).toBeCloseTo(102.4);
  });
});

// Test priority ordering logic
describe("priority ordering", () => {
  interface Task {
    id: string;
    priority: number;
    addedAt: string;
  }

  function sortByPriority(tasks: Task[]): Task[] {
    return [...tasks].sort((a, b) => {
      if (a.priority !== b.priority) return b.priority - a.priority;
      return new Date(b.addedAt).getTime() - new Date(a.addedAt).getTime();
    });
  }

  it("higher priority comes first", () => {
    const tasks: Task[] = [
      { id: "a", priority: 1, addedAt: "2026-05-29T00:00:00Z" },
      { id: "b", priority: 2, addedAt: "2026-05-29T00:00:00Z" },
    ];
    const sorted = sortByPriority(tasks);
    expect(sorted[0].id).toBe("b");
  });

  it("newer task comes first at same priority", () => {
    const tasks: Task[] = [
      { id: "a", priority: 1, addedAt: "2026-05-28T00:00:00Z" },
      { id: "b", priority: 1, addedAt: "2026-05-29T00:00:00Z" },
    ];
    const sorted = sortByPriority(tasks);
    expect(sorted[0].id).toBe("b");
  });

  it("handles empty array", () => {
    expect(sortByPriority([])).toEqual([]);
  });

  it("handles single item", () => {
    const tasks: Task[] = [
      { id: "a", priority: 1, addedAt: "2026-05-29T00:00:00Z" },
    ];
    expect(sortByPriority(tasks)).toHaveLength(1);
  });
});
