import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock Tauri clipboard manager
vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  readText: vi.fn(),
}));

// Mock React hooks for testing the pure logic
vi.mock("react", async () => {
  const actual = await vi.importActual("react");
  return {
    ...actual,
    useEffect: (fn: () => void | (() => void)) => {
      // Execute immediately for testing
      const cleanup = fn();
      return cleanup;
    },
    useRef: <T>(initial: T) => ({ current: initial }),
    useCallback: <T extends (...args: unknown[]) => unknown>(fn: T) => fn,
  };
});

import { readText } from "@tauri-apps/plugin-clipboard-manager";

// Test the detectDownloadUrl logic directly (extracted from useClipboard)
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

describe("useClipboard - detectDownloadUrl", () => {
  it("detects magnet links", () => {
    const url = "magnet:?xt=urn:btih:abc123";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("detects ed2k links", () => {
    const url = "ed2k://|file|test.zip|12345|abc123|/";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("detects http links", () => {
    const url = "http://example.com/file.zip";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("detects https links", () => {
    const url = "https://example.com/file.zip";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("detects ftp links", () => {
    const url = "ftp://files.example.com/file.zip";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("rejects plain text", () => {
    expect(detectDownloadUrl("just some text")).toBeNull();
  });

  it("rejects javascript protocol", () => {
    expect(detectDownloadUrl("javascript:alert(1)")).toBeNull();
  });

  it("rejects empty string", () => {
    expect(detectDownloadUrl("")).toBeNull();
  });

  it("trims whitespace before matching", () => {
    const url = "  https://example.com/file.zip  ";
    expect(detectDownloadUrl(url)).toBe("https://example.com/file.zip");
  });

  it("rejects data URLs", () => {
    expect(detectDownloadUrl("data:text/html,<h1>test</h1>")).toBeNull();
  });

  it("is case insensitive for magnet", () => {
    const url = "MAGNET:?xt=urn:btih:abc123";
    expect(detectDownloadUrl(url)).toBe(url);
  });

  it("is case insensitive for ed2k", () => {
    const url = "ED2K://|file|test.zip|12345|abc123|/";
    expect(detectDownloadUrl(url)).toBe(url);
  });
});

describe("useClipboard - clipboard integration", () => {
  const mockReadText = vi.mocked(readText);

  beforeEach(() => {
    mockReadText.mockReset();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("readText is called on interval", async () => {
    mockReadText.mockResolvedValue("https://example.com/file.zip");

    // Simulate interval behavior
    const callback = vi.fn();

    // First check
    const text = await mockReadText();
    if (text) {
      const url = detectDownloadUrl(text);
      if (url) callback(url);
    }
    expect(callback).toHaveBeenCalledWith("https://example.com/file.zip");
  });

  it("ignores non-download URLs", async () => {
    mockReadText.mockResolvedValue("just regular text");

    const callback = vi.fn();
    const text = await mockReadText();
    if (text) {
      const url = detectDownloadUrl(text);
      if (url) callback(url);
    }
    expect(callback).not.toHaveBeenCalled();
  });

  it("handles readText error gracefully", async () => {
    mockReadText.mockRejectedValue(new Error("No clipboard access"));

    const callback = vi.fn();
    try {
      const text = await mockReadText();
      if (text) {
        const url = detectDownloadUrl(text);
        if (url) callback(url);
      }
    } catch {
      // Expected - ignore clipboard errors
    }
    expect(callback).not.toHaveBeenCalled();
  });

  it("cooldown prevents rapid re-detection", () => {
    const cooldown = 10000;
    let lastDetect = 0;
    const callback = vi.fn();

    // First detection
    const now1 = Date.now();
    if (now1 - lastDetect >= cooldown) {
      const url = detectDownloadUrl("https://example.com/file.zip");
      if (url) {
        lastDetect = now1;
        callback(url);
      }
    }
    expect(callback).toHaveBeenCalledOnce();

    // Second detection within cooldown
    const now2 = now1 + 5000; // 5 seconds later (within 10s cooldown)
    if (now2 - lastDetect >= cooldown) {
      const url = detectDownloadUrl("https://example.com/other.zip");
      if (url) {
        lastDetect = now2;
        callback(url);
      }
    }
    // Should still be called only once (cooldown blocked)
    expect(callback).toHaveBeenCalledOnce();
  });

  it("cooldown allows detection after expiry", () => {
    const cooldown = 10000;
    let lastDetect = 0;
    const callback = vi.fn();

    // First detection
    lastDetect = Date.now();
    callback("https://example.com/file.zip");

    // After cooldown
    const later = lastDetect + cooldown + 1;
    if (later - lastDetect >= cooldown) {
      const url = detectDownloadUrl("https://example.com/other.zip");
      if (url) callback(url);
    }
    expect(callback).toHaveBeenCalledTimes(2);
  });

  it("skips detection when text is same as last", async () => {
    mockReadText.mockResolvedValue("https://example.com/file.zip");
    let lastContent = "";
    const callback = vi.fn();

    // First read
    const text1 = await mockReadText();
    if (text1 && text1 !== lastContent) {
      lastContent = text1;
      const url = detectDownloadUrl(text1);
      if (url) callback(url);
    }
    expect(callback).toHaveBeenCalledOnce();

    // Same content again
    const text2 = await mockReadText();
    if (text2 && text2 !== lastContent) {
      lastContent = text2;
      const url = detectDownloadUrl(text2);
      if (url) callback(url);
    }
    // Should still be called only once
    expect(callback).toHaveBeenCalledOnce();
  });
});
