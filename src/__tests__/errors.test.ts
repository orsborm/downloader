import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  DownloaderError,
  ErrorCode,
  handleError,
  withErrorHandling,
} from "../lib/errors";

// Mock showToast
vi.mock("../components/Toast", () => ({
  showToast: vi.fn(),
}));

// Mock i18n store (errors.ts imports t() from useI18n)
vi.mock("../hooks/useI18n", () => {
  const zh = {
    errors: {
      networkError: "网络连接失败，请检查网络设置",
      timeout: "连接超时，请稍后重试",
      diskFull: "磁盘空间不足",
      unknownError: "发生未知错误",
      downloadFailed: "下载失败",
      fileNotFound: "文件不存在",
      permissionDenied: "权限不足",
      taskNotFound: "任务不存在",
      taskAlreadyExists: "任务已存在",
      invalidUrl: "无效的下载链接",
      serverError: "服务器错误",
    },
  };
  const getNestedValue = (obj: unknown, path: string): unknown =>
    path.split(".").reduce((cur: unknown, key: string) =>
      cur && typeof cur === "object" ? (cur as Record<string, unknown>)[key] : undefined, obj);
  const t = (key: string, params?: Record<string, string | number>): string => {
    const value = getNestedValue(zh, key);
    if (value === undefined || typeof value !== "string") return key;
    if (!params) return value;
    return Object.entries(params).reduce(
      (str, [k, v]) => str.replace(new RegExp(`\\{${k}\\}`, "g"), String(v)), value
    );
  };
  return { t, useI18nStore: { getState: () => ({ language: "zh", locale: zh }) } };
});

import { showToast } from "../components/Toast";

describe("DownloaderError", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("constructor", () => {
    it("creates error with default values", () => {
      const error = new DownloaderError("test message");
      expect(error.message).toBe("test message");
      expect(error.code).toBe(ErrorCode.UNKNOWN);
      expect(error.recoverable).toBe(false);
      expect(error.cause).toBeUndefined();
      expect(error.name).toBe("DownloaderError");
    });

    it("creates error with custom code", () => {
      const error = new DownloaderError("network issue", ErrorCode.NETWORK_ERROR);
      expect(error.code).toBe(ErrorCode.NETWORK_ERROR);
    });

    it("creates error with recoverable flag", () => {
      const error = new DownloaderError("timeout", ErrorCode.TIMEOUT, true);
      expect(error.recoverable).toBe(true);
    });

    it("creates error with cause", () => {
      const cause = new Error("original error");
      const error = new DownloaderError("wrapped", ErrorCode.UNKNOWN, false, cause);
      expect(error.cause).toBe(cause);
    });
  });

  describe("from", () => {
    it("returns same DownloaderError if passed one", () => {
      const original = new DownloaderError("test", ErrorCode.NETWORK_ERROR);
      const result = DownloaderError.from(original);
      expect(result).toBe(original);
    });

    it("creates DownloaderError from regular Error", () => {
      const error = new Error("network failed");
      const result = DownloaderError.from(error);
      expect(result).toBeInstanceOf(DownloaderError);
      expect(result.message).toBe("network failed");
      expect(result.code).toBe(ErrorCode.NETWORK_ERROR);
    });

    it("creates DownloaderError from string", () => {
      const result = DownloaderError.from("something went wrong");
      expect(result).toBeInstanceOf(DownloaderError);
      expect(result.message).toBe("something went wrong");
      expect(result.code).toBe(ErrorCode.UNKNOWN);
    });

    it("uses default code for unknown errors", () => {
      const result = DownloaderError.from(42, ErrorCode.CONFIG_LOAD_FAILED);
      expect(result.code).toBe(ErrorCode.CONFIG_LOAD_FAILED);
    });
  });

  describe("toUserMessage", () => {
    it("returns Chinese message for known error codes", () => {
      const error = new DownloaderError("original", ErrorCode.NETWORK_ERROR);
      expect(error.toUserMessage()).toBe("网络连接失败，请检查网络设置");
    });

    it("returns Chinese message for timeout", () => {
      const error = new DownloaderError("timeout", ErrorCode.TIMEOUT);
      expect(error.toUserMessage()).toBe("连接超时，请稍后重试");
    });

    it("returns Chinese message for disk full", () => {
      const error = new DownloaderError("no space", ErrorCode.DISK_FULL);
      expect(error.toUserMessage()).toBe("磁盘空间不足");
    });

    it("returns fallback message for unknown code", () => {
      const error = new DownloaderError("custom message", ErrorCode.UNKNOWN);
      expect(error.toUserMessage()).toBe("发生未知错误");
    });
  });

  describe("inferErrorCode", () => {
    it("infers NETWORK_ERROR from message", () => {
      const error = DownloaderError.from(new Error("Network request failed"));
      expect(error.code).toBe(ErrorCode.NETWORK_ERROR);
    });

    it("infers TIMEOUT from message", () => {
      const error = DownloaderError.from(new Error("Connection timeout"));
      expect(error.code).toBe(ErrorCode.TIMEOUT);
    });

    it("infers CONNECTION_REFUSED from message", () => {
      const error = DownloaderError.from(new Error("ECONNREFUSED"));
      expect(error.code).toBe(ErrorCode.CONNECTION_REFUSED);
    });

    it("infers FILE_NOT_FOUND from message", () => {
      const error = DownloaderError.from(new Error("File not found"));
      expect(error.code).toBe(ErrorCode.FILE_NOT_FOUND);
    });

    it("infers DISK_FULL from message", () => {
      const error = DownloaderError.from(new Error("ENOSPC: no space left"));
      expect(error.code).toBe(ErrorCode.DISK_FULL);
    });

    it("infers PERMISSION_DENIED from message", () => {
      const error = DownloaderError.from(new Error("EACCES: permission denied"));
      expect(error.code).toBe(ErrorCode.PERMISSION_DENIED);
    });

    it("defaults to UNKNOWN for unrecognized messages", () => {
      const error = DownloaderError.from(new Error("something weird"));
      expect(error.code).toBe(ErrorCode.UNKNOWN);
    });
  });
});

describe("handleError", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows toast with context and user message", () => {
    handleError(new Error("network failed"), "加载任务列表");
    expect(showToast).toHaveBeenCalledWith(
      "加载任务列表: 网络连接失败，请检查网络设置",
      "error"
    );
  });

  it("returns DownloaderError instance", () => {
    const result = handleError(new Error("timeout"), "测试操作");
    expect(result).toBeInstanceOf(DownloaderError);
    expect(result.code).toBe(ErrorCode.TIMEOUT);
  });

  it("handles string errors", () => {
    handleError("something failed", "测试操作");
    expect(showToast).toHaveBeenCalledWith(
      "测试操作: 发生未知错误",
      "error"
    );
  });

  it("handles DownloaderError instances", () => {
    const error = new DownloaderError("custom", ErrorCode.DISK_FULL);
    handleError(error, "下载文件");
    expect(showToast).toHaveBeenCalledWith(
      "下载文件: 磁盘空间不足",
      "error"
    );
  });
});

describe("withErrorHandling", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns result on success", async () => {
    const fn = vi.fn().mockResolvedValue("success");
    const result = await withErrorHandling(fn, "测试操作");
    expect(result).toBe("success");
    expect(showToast).not.toHaveBeenCalled();
  });

  it("returns undefined on failure", async () => {
    const fn = vi.fn().mockRejectedValue(new Error("failed"));
    const result = await withErrorHandling(fn, "测试操作");
    expect(result).toBeUndefined();
    expect(showToast).toHaveBeenCalled();
  });

  it("shows error toast on failure", async () => {
    const fn = vi.fn().mockRejectedValue(new Error("network failed"));
    await withErrorHandling(fn, "加载数据");
    expect(showToast).toHaveBeenCalledWith(
      "加载数据: 网络连接失败，请检查网络设置",
      "error"
    );
  });
});


describe("ErrorCode", () => {
  it("has all expected error codes", () => {
    expect(ErrorCode.NETWORK_ERROR).toBe("NETWORK_ERROR");
    expect(ErrorCode.TIMEOUT).toBe("TIMEOUT");
    expect(ErrorCode.DOWNLOAD_FAILED).toBe("DOWNLOAD_FAILED");
    expect(ErrorCode.DISK_FULL).toBe("DISK_FULL");
    expect(ErrorCode.TASK_NOT_FOUND).toBe("TASK_NOT_FOUND");
    expect(ErrorCode.INVALID_URL).toBe("INVALID_URL");
    expect(ErrorCode.CONFIG_LOAD_FAILED).toBe("CONFIG_LOAD_FAILED");
    expect(ErrorCode.PLUGIN_LOAD_FAILED).toBe("PLUGIN_LOAD_FAILED");
    expect(ErrorCode.RSS_PARSE_FAILED).toBe("RSS_PARSE_FAILED");
    expect(ErrorCode.EXTRACT_FAILED).toBe("EXTRACT_FAILED");
    expect(ErrorCode.UNKNOWN).toBe("UNKNOWN");
    expect(ErrorCode.PERMISSION_DENIED).toBe("PERMISSION_DENIED");
  });
});
