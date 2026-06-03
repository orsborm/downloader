import { describe, it, expect } from "vitest";

// Mirror extension/utils.js logic for testing

function isDownloadableUrl(url: string): boolean {
  const downloadableExtensions = [
    '.zip', '.rar', '.7z', '.tar', '.gz', '.bz2', '.xz',
    '.exe', '.msi', '.dmg', '.deb', '.rpm',
    '.iso', '.img',
    '.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv', '.webm',
    '.mp3', '.wav', '.flac', '.aac', '.ogg',
    '.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
    '.apk', '.ipa',
    '.torrent'
  ];
  const lower = url.toLowerCase();
  return downloadableExtensions.some(ext => lower.includes(ext)) ||
         lower.startsWith('magnet:') ||
         lower.startsWith('ed2k://');
}

function getFilename(url: string): string {
  try {
    const pathname = new URL(url).pathname;
    const parts = pathname.split('/');
    return decodeURIComponent(parts[parts.length - 1]) || '';
  } catch {
    return '';
  }
}

function getFormat(url: string): string {
  const filename = getFilename(url);
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  return ext;
}

function getTypeLabel(url: string): string {
  const lower = url.toLowerCase();
  if (lower.includes('.m3u8')) return 'HLS';
  if (lower.match(/\.(mp4|mkv|avi|mov|webm)/)) return '视频';
  if (lower.match(/\.(mp3|wav|flac|aac)/)) return '音频';
  if (lower.startsWith('magnet:')) return '磁力链';
  if (lower.startsWith('ed2k://')) return 'ed2k';
  return '文件';
}

function getTypeClass(url: string): string {
  const lower = url.toLowerCase();
  if (lower.includes('.m3u8')) return 'type-hls';
  if (lower.match(/\.(mp4|mkv|avi|mov|webm|mp3|wav|flac)/)) return 'type-video';
  return 'type-file';
}

describe("extension: isDownloadableUrl", () => {
  it("detects .zip files", () => {
    expect(isDownloadableUrl("https://example.com/file.zip")).toBe(true);
  });

  it("detects .exe files", () => {
    expect(isDownloadableUrl("https://example.com/setup.exe")).toBe(true);
  });

  it("detects .mp4 files", () => {
    expect(isDownloadableUrl("https://example.com/video.mp4")).toBe(true);
  });

  it("detects .torrent files", () => {
    expect(isDownloadableUrl("https://example.com/file.torrent")).toBe(true);
  });

  it("detects magnet links", () => {
    expect(isDownloadableUrl("magnet:?xt=urn:btih:abc123")).toBe(true);
  });

  it("detects ed2k links", () => {
    expect(isDownloadableUrl("ed2k://|file|test.zip|12345|abc|/")).toBe(true);
  });

  it("rejects plain HTML pages", () => {
    expect(isDownloadableUrl("https://example.com/page")).toBe(false);
  });

  it("rejects empty string", () => {
    expect(isDownloadableUrl("")).toBe(false);
  });

  it("handles case insensitivity", () => {
    expect(isDownloadableUrl("https://example.com/FILE.ZIP")).toBe(true);
    expect(isDownloadableUrl("MAGNET:?xt=urn:btih:abc")).toBe(true);
  });

  it("detects archive formats", () => {
    expect(isDownloadableUrl("https://example.com/file.rar")).toBe(true);
    expect(isDownloadableUrl("https://example.com/file.7z")).toBe(true);
    expect(isDownloadableUrl("https://example.com/file.tar.gz")).toBe(true);
  });

  it("detects document formats", () => {
    expect(isDownloadableUrl("https://example.com/doc.pdf")).toBe(true);
    expect(isDownloadableUrl("https://example.com/sheet.xlsx")).toBe(true);
  });

  it("detects audio formats", () => {
    expect(isDownloadableUrl("https://example.com/song.mp3")).toBe(true);
    expect(isDownloadableUrl("https://example.com/track.flac")).toBe(true);
  });

  it("detects webm video", () => {
    expect(isDownloadableUrl("https://example.com/clip.webm")).toBe(true);
  });

  it("detects ISO images", () => {
    expect(isDownloadableUrl("https://example.com/ubuntu.iso")).toBe(true);
  });
});

describe("extension: getFilename", () => {
  it("extracts filename from URL", () => {
    expect(getFilename("https://example.com/path/file.zip")).toBe("file.zip");
  });

  it("handles URL with query params", () => {
    expect(getFilename("https://example.com/file.zip?token=abc")).toBe("file.zip");
  });

  it("handles URL with encoded characters", () => {
    expect(getFilename("https://example.com/%E4%B8%AD%E6%96%87.zip")).toBe("中文.zip");
  });

  it("returns empty for invalid URL", () => {
    expect(getFilename("not a url")).toBe("");
  });

  it("handles root URL", () => {
    expect(getFilename("https://example.com/")).toBe("");
  });

  it("handles nested paths", () => {
    expect(getFilename("https://example.com/a/b/c/file.txt")).toBe("file.txt");
  });
});

describe("extension: getFormat", () => {
  it("extracts file extension", () => {
    expect(getFormat("https://example.com/file.zip")).toBe("zip");
  });

  it("handles multi-part extensions", () => {
    expect(getFormat("https://example.com/file.tar.gz")).toBe("gz");
  });

  it("returns last path segment if no extension", () => {
    expect(getFormat("https://example.com/path/file")).toBe("file");
  });

  it("handles mp4 extension", () => {
    expect(getFormat("https://example.com/video.mp4")).toBe("mp4");
  });
});

describe("extension: getTypeLabel", () => {
  it("labels HLS streams", () => {
    expect(getTypeLabel("https://example.com/stream.m3u8")).toBe("HLS");
  });

  it("labels video files", () => {
    expect(getTypeLabel("https://example.com/video.mp4")).toBe("视频");
    expect(getTypeLabel("https://example.com/video.mkv")).toBe("视频");
    expect(getTypeLabel("https://example.com/clip.webm")).toBe("视频");
  });

  it("labels audio files", () => {
    expect(getTypeLabel("https://example.com/song.mp3")).toBe("音频");
    expect(getTypeLabel("https://example.com/track.flac")).toBe("音频");
  });

  it("labels magnet links", () => {
    expect(getTypeLabel("magnet:?xt=urn:btih:abc")).toBe("磁力链");
  });

  it("labels ed2k links", () => {
    expect(getTypeLabel("ed2k://|file|test|123|abc|/")).toBe("ed2k");
  });

  it("labels other files", () => {
    expect(getTypeLabel("https://example.com/file.zip")).toBe("文件");
    expect(getTypeLabel("https://example.com/doc.pdf")).toBe("文件");
  });
});

describe("extension: getTypeClass", () => {
  it("returns type-hls for HLS", () => {
    expect(getTypeClass("https://example.com/stream.m3u8")).toBe("type-hls");
  });

  it("returns type-video for video", () => {
    expect(getTypeClass("https://example.com/video.mp4")).toBe("type-video");
    expect(getTypeClass("https://example.com/video.mkv")).toBe("type-video");
  });

  it("returns type-video for audio", () => {
    expect(getTypeClass("https://example.com/song.mp3")).toBe("type-video");
  });

  it("returns type-file for other", () => {
    expect(getTypeClass("https://example.com/file.zip")).toBe("type-file");
  });
});
