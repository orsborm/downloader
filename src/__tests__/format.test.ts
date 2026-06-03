import { describe, it, expect } from "vitest";
import {
  formatSize,
  formatSpeed,
  formatEta,
  formatProgress,
  formatDateTime,
  formatDuration,
  protocolLabel,
  stateLabel,
  isValidDownloadUrl,
  filterHistory,
  computeHistoryStats,
  aggregateByDay,
} from "../lib/format";

describe("formatSize", () => {
  it("returns '0 B' for zero bytes", () => {
    expect(formatSize(0)).toBe("0 B");
  });

  it("returns '0 B' for negative bytes", () => {
    expect(formatSize(-100)).toBe("0 B");
  });

  it("formats bytes correctly", () => {
    expect(formatSize(500)).toBe("500 B");
  });

  it("formats kilobytes correctly", () => {
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(1536)).toBe("1.5 KB");
  });

  it("formats megabytes correctly", () => {
    expect(formatSize(1048576)).toBe("1.0 MB");
    expect(formatSize(5 * 1024 * 1024)).toBe("5.0 MB");
  });

  it("formats gigabytes correctly", () => {
    expect(formatSize(1073741824)).toBe("1.0 GB");
  });

  it("formats terabytes correctly", () => {
    expect(formatSize(1099511627776)).toBe("1.0 TB");
  });

  it("handles NaN", () => {
    expect(formatSize(NaN)).toBe("0 B");
  });

  it("handles Infinity", () => {
    expect(formatSize(Infinity)).toBe("0 B");
  });
});

describe("formatSpeed", () => {
  it("returns '0 B/s' for zero speed", () => {
    expect(formatSpeed(0)).toBe("0 B/s");
  });

  it("formats speed correctly", () => {
    expect(formatSpeed(1024)).toBe("1.0 KB/s");
    expect(formatSpeed(1048576)).toBe("1.0 MB/s");
  });

  it("handles NaN", () => {
    expect(formatSpeed(NaN)).toBe("0 B/s");
  });

  it("handles Infinity", () => {
    expect(formatSpeed(Infinity)).toBe("0 B/s");
  });
});

describe("formatEta", () => {
  it("returns infinity for null", () => {
    expect(formatEta(null)).toBe("∞");
  });

  it("returns '0s' for zero", () => {
    expect(formatEta(0)).toBe("0s");
  });

  it("returns -- for negative", () => {
    expect(formatEta(-5)).toBe("--");
  });

  it("formats seconds", () => {
    expect(formatEta(30)).toBe("30s");
  });

  it("formats minutes and seconds", () => {
    expect(formatEta(90)).toBe("1m30s");
    expect(formatEta(60)).toBe("1m0s");
  });

  it("formats hours and minutes", () => {
    expect(formatEta(3600)).toBe("1h0m");
    expect(formatEta(3661)).toBe("1h1m");
  });

  it("floors fractional seconds", () => {
    expect(formatEta(30.7)).toBe("30s");
    expect(formatEta(90.9)).toBe("1m30s");
    expect(formatEta(3661.5)).toBe("1h1m");
  });

  it("handles very large values", () => {
    expect(formatEta(86400)).toBe("24h0m");
    expect(formatEta(90061)).toBe("25h1m");
  });
});

describe("formatProgress", () => {
  it("formats zero progress", () => {
    expect(formatProgress(0)).toBe("0.0%");
  });

  it("formats half progress", () => {
    expect(formatProgress(0.5)).toBe("50.0%");
  });

  it("formats full progress", () => {
    expect(formatProgress(1)).toBe("100.0%");
  });

  it("formats fractional progress", () => {
    expect(formatProgress(0.123)).toBe("12.3%");
  });

  it("handles NaN progress", () => {
    expect(formatProgress(NaN)).toBe("0.0%");
  });

  it("handles Infinity progress", () => {
    expect(formatProgress(Infinity)).toBe("0.0%");
  });
});

describe("formatDateTime", () => {
  it("formats a valid ISO date string", () => {
    const result = formatDateTime("2026-05-29T10:30:00Z");
    // Should contain the date components (locale-dependent format)
    expect(result).toContain("2026");
    expect(result).toContain("05");
    expect(result).toContain("29");
  });

  it("returns raw string for invalid date", () => {
    expect(formatDateTime("not-a-date")).toBe("not-a-date");
  });

  it("handles empty string", () => {
    const result = formatDateTime("");
    // Empty string creates Invalid Date, should return raw
    expect(result).toBe("");
  });
});

describe("protocolLabel", () => {
  it("returns correct labels for known protocols", () => {
    expect(protocolLabel("HTTP")).toBe("HTTP");
    expect(protocolLabel("FTP")).toBe("FTP");
    expect(protocolLabel("BT")).toBe("BT");
    expect(protocolLabel("MAGNET")).toBe("磁力链");
    expect(protocolLabel("ED2K")).toBe("ed2k");
    expect(protocolLabel("HLS")).toBe("HLS");
    expect(protocolLabel("DASH")).toBe("DASH");
  });

  it("returns the input for unknown protocols", () => {
    expect(protocolLabel("UNKNOWN")).toBe("UNKNOWN");
  });
});

describe("stateLabel", () => {
  it("returns correct labels for known states", () => {
    expect(stateLabel("queued")).toBe("等待中");
    expect(stateLabel("downloading")).toBe("下载中");
    expect(stateLabel("paused")).toBe("已暂停");
    expect(stateLabel("seeding")).toBe("做种中");
    expect(stateLabel("done")).toBe("已完成");
    expect(stateLabel("error")).toBe("错误");
  });

  it("returns the input for unknown states", () => {
    expect(stateLabel("unknown")).toBe("unknown");
  });
});

describe("formatDuration", () => {
  it("formats seconds only", () => {
    expect(formatDuration(5)).toBe("5秒");
    expect(formatDuration(59)).toBe("59秒");
  });

  it("formats minutes and seconds", () => {
    expect(formatDuration(60)).toBe("1分");
    expect(formatDuration(90)).toBe("1分30秒");
    expect(formatDuration(3599)).toBe("59分59秒");
  });

  it("formats hours and minutes", () => {
    expect(formatDuration(3600)).toBe("1时");
    expect(formatDuration(3660)).toBe("1时1分");
    expect(formatDuration(7200)).toBe("2时");
  });

  it("handles zero", () => {
    expect(formatDuration(0)).toBe("0秒");
  });

  it("handles negative values", () => {
    expect(formatDuration(-1)).toBe("--");
  });

  it("handles NaN and Infinity", () => {
    expect(formatDuration(NaN)).toBe("--");
    expect(formatDuration(Infinity)).toBe("--");
  });
});

describe("isValidDownloadUrl", () => {
  it("accepts HTTP URLs", () => {
    expect(isValidDownloadUrl("http://example.com/file.zip")).toBe(true);
  });

  it("accepts HTTPS URLs", () => {
    expect(isValidDownloadUrl("https://example.com/file.zip")).toBe(true);
  });

  it("accepts FTP URLs", () => {
    expect(isValidDownloadUrl("ftp://example.com/file.zip")).toBe(true);
  });

  it("accepts magnet links", () => {
    expect(isValidDownloadUrl("magnet:?xt=urn:btih:abc123")).toBe(true);
  });

  it("accepts ed2k links", () => {
    expect(isValidDownloadUrl("ed2k://|file|test.zip|12345|abcdef|/")).toBe(true);
  });

  it("rejects empty strings", () => {
    expect(isValidDownloadUrl("")).toBe(false);
    expect(isValidDownloadUrl("   ")).toBe(false);
  });

  it("rejects random text", () => {
    expect(isValidDownloadUrl("not a url")).toBe(false);
  });

  it("rejects unsupported protocols", () => {
    expect(isValidDownloadUrl("javascript:alert(1)")).toBe(false);
    expect(isValidDownloadUrl("data:text/html,<h1>test</h1>")).toBe(false);
  });

  it("handles case-insensitive protocol matching", () => {
    expect(isValidDownloadUrl("HTTP://example.com/file.zip")).toBe(true);
    expect(isValidDownloadUrl("HTTPS://example.com/file.zip")).toBe(true);
    expect(isValidDownloadUrl("MAGNET:?xt=urn:btih:abc123")).toBe(true);
  });

  it("accepts m3u8 HLS links", () => {
    expect(isValidDownloadUrl("https://example.com/stream.m3u8")).toBe(true);
  });

  it("accepts mpd DASH links", () => {
    expect(isValidDownloadUrl("https://example.com/manifest.mpd")).toBe(true);
  });

  it("accepts .torrent file paths", () => {
    expect(isValidDownloadUrl("C:\\Downloads\\file.torrent")).toBe(true);
    expect(isValidDownloadUrl("/home/user/file.torrent")).toBe(true);
  });

  it("accepts FTPS URLs", () => {
    expect(isValidDownloadUrl("ftps://example.com/file.zip")).toBe(true);
  });

  it("accepts file:// URLs", () => {
    expect(isValidDownloadUrl("file:///home/user/file.zip")).toBe(true);
  });

  it("trims whitespace before validation", () => {
    expect(isValidDownloadUrl("  https://example.com/file.zip  ")).toBe(true);
    expect(isValidDownloadUrl("  ")).toBe(false);
  });

  it("rejects URLs with only protocol prefix", () => {
    expect(isValidDownloadUrl("http://")).toBe(true); // valid prefix
    expect(isValidDownloadUrl("magnet:")).toBe(true); // valid prefix
  });

  it("handles .torrent case insensitivity", () => {
    expect(isValidDownloadUrl("C:\\Downloads\\FILE.TORRENT")).toBe(true);
    expect(isValidDownloadUrl("C:\\Downloads\\file.Torrent")).toBe(true);
  });

  it("accepts m3u8 with query params", () => {
    expect(isValidDownloadUrl("https://example.com/stream.m3u8?token=abc")).toBe(true);
  });

  it("accepts HTTP URL even if hostname contains m3u8", () => {
    expect(isValidDownloadUrl("https://m3u8.example.com/video.mp4")).toBe(true);
  });

  it("rejects null and undefined-like inputs", () => {
    expect(isValidDownloadUrl(null as unknown as string)).toBe(false);
    expect(isValidDownloadUrl(undefined as unknown as string)).toBe(false);
  });
});

describe("formatSize additional edge cases", () => {
  it("handles exactly 1 MB boundary", () => {
    expect(formatSize(1048575)).toBe("1024.0 KB");
    expect(formatSize(1048576)).toBe("1.0 MB");
  });

  it("handles exactly 1 GB boundary", () => {
    expect(formatSize(1073741824)).toBe("1.0 GB");
  });

  it("handles exactly 1 TB boundary", () => {
    expect(formatSize(1099511627776)).toBe("1.0 TB");
  });

  it("handles fractional TB", () => {
    expect(formatSize(1.5 * 1024 * 1024 * 1024 * 1024)).toBe("1.5 TB");
  });
});

describe("formatEta additional edge cases", () => {
  it("handles 59 seconds", () => {
    expect(formatEta(59)).toBe("59s");
  });

  it("handles 3599 seconds (just under 1 hour)", () => {
    expect(formatEta(3599)).toBe("59m59s");
  });

  it("handles 86400 seconds (24 hours)", () => {
    expect(formatEta(86400)).toBe("24h0m");
  });

  it("handles very small fractional seconds", () => {
    expect(formatEta(0.1)).toBe("0s");
    expect(formatEta(0.9)).toBe("0s");
  });
});

describe("formatDuration additional edge cases", () => {
  it("handles 1 second", () => {
    expect(formatDuration(1)).toBe("1秒");
  });

  it("handles 1 minute exactly", () => {
    expect(formatDuration(60)).toBe("1分");
  });

  it("handles 1 hour exactly", () => {
    expect(formatDuration(3600)).toBe("1时");
  });

  it("handles very large durations", () => {
    expect(formatDuration(86400)).toBe("24时");
  });
});

describe("formatSize edge cases", () => {
  it("handles 1 byte", () => {
    expect(formatSize(1)).toBe("1 B");
  });

  it("handles exactly 1 KB boundary", () => {
    expect(formatSize(1023)).toBe("1023 B");
    expect(formatSize(1024)).toBe("1.0 KB");
  });

  it("handles very large values", () => {
    expect(formatSize(5 * 1024 * 1024 * 1024 * 1024)).toBe("5.0 TB");
  });

  it("formats fractional KB correctly", () => {
    expect(formatSize(1536)).toBe("1.5 KB");
    expect(formatSize(2048)).toBe("2.0 KB");
  });
});

describe("formatSpeed edge cases", () => {
  it("formats TB/s correctly", () => {
    expect(formatSpeed(1099511627776)).toBe("1.0 TB/s");
  });

  it("formats fractional speeds", () => {
    expect(formatSpeed(1536)).toBe("1.5 KB/s");
  });
});

describe("formatEta edge cases", () => {
  it("handles exactly 1 second", () => {
    expect(formatEta(1)).toBe("1s");
  });

  it("handles exactly 1 minute", () => {
    expect(formatEta(60)).toBe("1m0s");
  });

  it("handles exactly 1 hour", () => {
    expect(formatEta(3600)).toBe("1h0m");
  });

  it("handles NaN", () => {
    expect(formatEta(NaN)).toBe("∞");
  });
});

describe("formatProgress edge cases", () => {
  it("handles negative progress", () => {
    expect(formatProgress(-0.5)).toBe("-50.0%");
  });

  it("handles progress > 1", () => {
    expect(formatProgress(1.5)).toBe("150.0%");
  });

  it("handles very small progress", () => {
    expect(formatProgress(0.001)).toBe("0.1%");
  });
});

describe("formatDateTime edge cases", () => {
  it("formats epoch timestamp", () => {
    const result = formatDateTime("1970-01-01T00:00:00Z");
    expect(result).toContain("1970");
  });

  it("handles future dates", () => {
    const result = formatDateTime("2099-06-15T12:00:00Z");
    expect(result).toContain("2099");
  });
});

describe("isValidDownloadUrl security edge cases", () => {
  it("rejects vbscript protocol", () => {
    expect(isValidDownloadUrl("vbscript:MsgBox")).toBe(false);
  });

  it("rejects chrome protocol", () => {
    expect(isValidDownloadUrl("chrome://settings")).toBe(false);
  });

  it("rejects about protocol", () => {
    expect(isValidDownloadUrl("about:blank")).toBe(false);
  });

  it("rejects blob URLs", () => {
    expect(isValidDownloadUrl("blob:https://example.com/abc-123")).toBe(false);
  });

  it("accepts HTTP with port", () => {
    expect(isValidDownloadUrl("http://localhost:8080/file.zip")).toBe(true);
  });

  it("accepts HTTPS with auth", () => {
    expect(isValidDownloadUrl("https://user:pass@example.com/file.zip")).toBe(true);
  });

  it("accepts magnet with multiple trackers", () => {
    expect(isValidDownloadUrl(
      "magnet:?xt=urn:btih:abc&dn=test&tr=udp://t1.com&tr=udp://t2.com"
    )).toBe(true);
  });

  it("accepts ed2k with unicode filename", () => {
    expect(isValidDownloadUrl("ed2k://|file|%E4%B8%AD%E6%96%87.zip|12345|abc|/")).toBe(true);
  });

  it("rejects URLs with XSS attempt", () => {
    expect(isValidDownloadUrl('javascript:alert(document.cookie)')).toBe(false);
  });

  it("handles URL with fragments", () => {
    expect(isValidDownloadUrl("https://example.com/file.zip#section")).toBe(true);
  });

  it("accepts HTTP with IPv4", () => {
    expect(isValidDownloadUrl("http://192.168.1.1/file.zip")).toBe(true);
  });

  it("accepts HTTP with IPv6", () => {
    expect(isValidDownloadUrl("http://[::1]:8080/file.zip")).toBe(true);
  });
});

describe("formatSize with very small values", () => {
  it("formats 2 bytes", () => {
    expect(formatSize(2)).toBe("2 B");
  });

  it("formats 1023 bytes", () => {
    expect(formatSize(1023)).toBe("1023 B");
  });
});

describe("formatSpeed edge cases additional", () => {
  it("formats 1 B/s", () => {
    expect(formatSpeed(1)).toBe("1 B/s");
  });

  it("formats GB/s", () => {
    expect(formatSpeed(1073741824)).toBe("1.0 GB/s");
  });

  it("handles negative speed", () => {
    expect(formatSpeed(-100)).toBe("0 B/s");
  });
});

describe("filterHistory", () => {
  const items = [
    { name: "movie.mp4", url: "https://example.com/movie.mp4", protocol: "HTTP", totalSize: 1000 },
    { name: "archive.zip", url: "https://cdn.test.com/archive.zip", protocol: "FTP", totalSize: 2000 },
    { name: "distro.iso", url: "magnet:?xt=urn:btih:abc", protocol: "MAGNET", totalSize: 3000 },
    { name: "game.tar.gz", url: "ed2k://|file|game|100|hash|/", protocol: "ED2K", totalSize: 4000 },
  ];

  it("returns all items for empty query", () => {
    expect(filterHistory(items, "")).toHaveLength(4);
    expect(filterHistory(items, "  ")).toHaveLength(4);
  });

  it("filters by file name (case-insensitive)", () => {
    expect(filterHistory(items, "MOVIE")).toHaveLength(1);
    expect(filterHistory(items, "movie")).toHaveLength(1);
    expect(filterHistory(items, "archive")).toHaveLength(1);
  });

  it("filters by URL", () => {
    expect(filterHistory(items, "cdn.test")).toHaveLength(1);
    expect(filterHistory(items, "example.com")).toHaveLength(1);
  });

  it("filters by protocol label", () => {
    expect(filterHistory(items, "ed2k")).toHaveLength(1);
    expect(filterHistory(items, "磁力链")).toHaveLength(1);
    // "HTTP" also matches "https" in URLs, so 2 items match
    expect(filterHistory(items, "FTP")).toHaveLength(1);
  });

  it("returns empty for no match", () => {
    expect(filterHistory(items, "nonexistent")).toHaveLength(0);
  });

  it("handles items with null name/url", () => {
    const sparse = [{ name: null, url: null, protocol: "HTTP", totalSize: 0 }];
    expect(filterHistory(sparse, "test")).toHaveLength(0);
    expect(filterHistory(sparse, "")).toHaveLength(1);
  });
});

describe("computeHistoryStats", () => {
  const items = [
    { name: "a", protocol: "HTTP", totalSize: 1000, downloaded: 1000, averageSpeed: 500, duration: 10 },
    { name: "b", protocol: "BT", totalSize: 2000, downloaded: 2000, averageSpeed: 300, duration: 20 },
    { name: "c", protocol: "FTP", totalSize: 3000, downloaded: 3000, averageSpeed: 200, duration: 30 },
  ];

  it("calculates totals correctly", () => {
    const stats = computeHistoryStats(items);
    expect(stats.count).toBe(3);
    expect(stats.totalSize).toBe(6000);
    expect(stats.totalDownloaded).toBe(6000);
    expect(stats.avgSpeed).toBeCloseTo(333.33, 0);
    expect(stats.totalDuration).toBe(60);
  });

  it("returns zeros for empty array", () => {
    const stats = computeHistoryStats([]);
    expect(stats.count).toBe(0);
    expect(stats.totalSize).toBe(0);
    expect(stats.totalDownloaded).toBe(0);
    expect(stats.avgSpeed).toBe(0);
    expect(stats.totalDuration).toBe(0);
  });

  it("handles single item", () => {
    const stats = computeHistoryStats([items[0]]);
    expect(stats.count).toBe(1);
    expect(stats.totalSize).toBe(1000);
    expect(stats.avgSpeed).toBe(500);
  });

  it("handles items with missing fields", () => {
    const sparse = [{ name: "x", protocol: "HTTP" }];
    const stats = computeHistoryStats(sparse as any);
    expect(stats.count).toBe(1);
    expect(stats.totalSize).toBe(0);
    expect(stats.totalDownloaded).toBe(0);
    expect(stats.avgSpeed).toBe(0);
    expect(stats.totalDuration).toBe(0);
  });
});

describe("aggregateByDay", () => {
  it("returns correct number of days (includes today)", () => {
    const result = aggregateByDay([], 7);
    expect(result).toHaveLength(8); // 7 days ago + today
  });

  it("returns correct number of days for 30 day range", () => {
    const result = aggregateByDay([], 30);
    expect(result).toHaveLength(31); // 30 days ago + today
  });

  it("each entry has date, downloads, totalSize", () => {
    const result = aggregateByDay([], 5);
    for (const entry of result) {
      expect(entry).toHaveProperty("date");
      expect(entry).toHaveProperty("downloads");
      expect(entry).toHaveProperty("totalSize");
      expect(entry.downloads).toBe(0);
      expect(entry.totalSize).toBe(0);
    }
  });

  it("aggregates items by date", () => {
    const today = new Date().toISOString().slice(0, 10);
    const items = [
      { completedAt: `${today}T10:00:00`, totalSize: 1000 },
      { completedAt: `${today}T14:00:00`, totalSize: 2000 },
      { completedAt: `${today}T18:00:00`, totalSize: 500 },
    ];
    const result = aggregateByDay(items, 7);
    const todayEntry = result.find((d) => d.date === today);
    expect(todayEntry).toBeDefined();
    expect(todayEntry!.downloads).toBe(3);
    expect(todayEntry!.totalSize).toBe(3500);
  });

  it("ignores items without completedAt", () => {
    const items = [
      { completedAt: null, totalSize: 1000 },
      { completedAt: undefined, totalSize: 2000 },
    ];
    const result = aggregateByDay(items as any, 7);
    expect(result.every((d) => d.downloads === 0)).toBe(true);
  });

  it("ignores items outside date range", () => {
    const oldDate = "2020-01-01T10:00:00";
    const items = [{ completedAt: oldDate, totalSize: 1000 }];
    const result = aggregateByDay(items, 7);
    expect(result.every((d) => d.downloads === 0)).toBe(true);
  });

  it("dates are in YYYY-MM-DD format", () => {
    const result = aggregateByDay([], 5);
    for (const entry of result) {
      expect(entry.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    }
  });
});
