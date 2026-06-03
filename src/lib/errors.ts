// 统一错误处理模块
// 提供标准化的错误类型、错误码和错误处理函数

import { showToast } from "../components/Toast";
import { t } from "../hooks/useI18n";

/** 错误码枚举 */
export enum ErrorCode {
  // 网络相关
  NETWORK_ERROR = "NETWORK_ERROR",
  TIMEOUT = "TIMEOUT",
  CONNECTION_REFUSED = "CONNECTION_REFUSED",

  // 下载相关
  DOWNLOAD_FAILED = "DOWNLOAD_FAILED",
  DOWNLOAD_PAUSED = "DOWNLOAD_PAUSED",
  FILE_NOT_FOUND = "FILE_NOT_FOUND",
  DISK_FULL = "DISK_FULL",

  // 任务相关
  TASK_NOT_FOUND = "TASK_NOT_FOUND",
  TASK_ALREADY_EXISTS = "TASK_ALREADY_EXISTS",
  INVALID_URL = "INVALID_URL",

  // 配置相关
  CONFIG_LOAD_FAILED = "CONFIG_LOAD_FAILED",
  CONFIG_SAVE_FAILED = "CONFIG_SAVE_FAILED",

  // 插件相关
  PLUGIN_LOAD_FAILED = "PLUGIN_LOAD_FAILED",
  PLUGIN_EXEC_FAILED = "PLUGIN_EXEC_FAILED",

  // RSS 相关
  RSS_PARSE_FAILED = "RSS_PARSE_FAILED",
  RSS_FETCH_FAILED = "RSS_FETCH_FAILED",

  // 解压相关
  EXTRACT_FAILED = "EXTRACT_FAILED",
  PASSWORD_REQUIRED = "PASSWORD_REQUIRED",
  INVALID_ARCHIVE = "INVALID_ARCHIVE",

  // 通用
  UNKNOWN = "UNKNOWN",
  PERMISSION_DENIED = "PERMISSION_DENIED",
}

/** 下载器错误类 */
export class DownloaderError extends Error {
  /** 错误码 */
  public readonly code: ErrorCode;
  /** 是否可恢复 */
  public readonly recoverable: boolean;
  /** 原始错误 */
  public readonly cause?: Error;

  constructor(
    message: string,
    code: ErrorCode = ErrorCode.UNKNOWN,
    recoverable: boolean = false,
    cause?: Error
  ) {
    super(message);
    this.name = "DownloaderError";
    this.code = code;
    this.recoverable = recoverable;
    this.cause = cause;
  }

  /** 从未知错误创建 DownloaderError */
  static from(error: unknown, defaultCode: ErrorCode = ErrorCode.UNKNOWN): DownloaderError {
    if (error instanceof DownloaderError) {
      return error;
    }

    if (error instanceof Error) {
      // 尝试从错误消息推断错误码
      const code = inferErrorCode(error.message);
      return new DownloaderError(error.message, code, false, error);
    }

    const message = String(error);
    return new DownloaderError(message, defaultCode, false);
  }

  /** 获取用户友好的错误消息 */
  toUserMessage(): string {
    return getErrorMessage(this.code, this.message);
  }
}

/** 从错误消息推断错误码 */
function inferErrorCode(message: string): ErrorCode {
  const lower = message.toLowerCase();

  if (lower.includes("network") || lower.includes("fetch")) {
    return ErrorCode.NETWORK_ERROR;
  }
  if (lower.includes("timeout")) {
    return ErrorCode.TIMEOUT;
  }
  if (lower.includes("connection refused") || lower.includes("econnrefused")) {
    return ErrorCode.CONNECTION_REFUSED;
  }
  if (lower.includes("not found") || lower.includes("404")) {
    return ErrorCode.FILE_NOT_FOUND;
  }
  if (lower.includes("disk") || lower.includes("space") || lower.includes("enospc")) {
    return ErrorCode.DISK_FULL;
  }
  if (lower.includes("permission") || lower.includes("eacces")) {
    return ErrorCode.PERMISSION_DENIED;
  }

  return ErrorCode.UNKNOWN;
}

/** 获取用户友好的错误消息（i18n） */
function getErrorMessage(code: ErrorCode, fallback: string): string {
  const keyMap: Record<ErrorCode, string> = {
    [ErrorCode.NETWORK_ERROR]: "errors.networkError",
    [ErrorCode.TIMEOUT]: "errors.timeout",
    [ErrorCode.CONNECTION_REFUSED]: "errors.serverError",
    [ErrorCode.DOWNLOAD_FAILED]: "errors.downloadFailed",
    [ErrorCode.DOWNLOAD_PAUSED]: "errors.downloadFailed",
    [ErrorCode.FILE_NOT_FOUND]: "errors.fileNotFound",
    [ErrorCode.DISK_FULL]: "errors.diskFull",
    [ErrorCode.TASK_NOT_FOUND]: "errors.taskNotFound",
    [ErrorCode.TASK_ALREADY_EXISTS]: "errors.taskAlreadyExists",
    [ErrorCode.INVALID_URL]: "errors.invalidUrl",
    [ErrorCode.CONFIG_LOAD_FAILED]: "errors.unknownError",
    [ErrorCode.CONFIG_SAVE_FAILED]: "errors.unknownError",
    [ErrorCode.PLUGIN_LOAD_FAILED]: "errors.unknownError",
    [ErrorCode.PLUGIN_EXEC_FAILED]: "errors.unknownError",
    [ErrorCode.RSS_PARSE_FAILED]: "errors.unknownError",
    [ErrorCode.RSS_FETCH_FAILED]: "errors.networkError",
    [ErrorCode.EXTRACT_FAILED]: "errors.unknownError",
    [ErrorCode.PASSWORD_REQUIRED]: "errors.unknownError",
    [ErrorCode.INVALID_ARCHIVE]: "errors.unknownError",
    [ErrorCode.UNKNOWN]: "errors.unknownError",
    [ErrorCode.PERMISSION_DENIED]: "errors.permissionDenied",
  };

  const key = keyMap[code];
  if (key) {
    const translated = t(key);
    if (translated !== key) return translated;
  }
  return fallback;
}

/**
 * 从任意错误值提取用户可读的消息字符串
 * - Error 对象 → message（去掉 "Error: " 前缀）
 * - 字符串 → 直接返回
 * - 其他 → String(value)
 */
export function extractErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  return String(error);
}

/**
 * 处理错误并显示 Toast
 * @param error 原始错误
 * @param context 错误上下文描述（如 "加载任务列表"）
 * @param defaultCode 默认错误码
 */
export function handleError(
  error: unknown,
  context: string,
  defaultCode: ErrorCode = ErrorCode.UNKNOWN
): DownloaderError {
  const downloaderError = DownloaderError.from(error, defaultCode);
  const userMessage = downloaderError.toUserMessage();
  showToast(`${context}: ${userMessage}`, "error");
  return downloaderError;
}

/**
 * 包装异步操作，自动处理错误
 * @param fn 异步操作
 * @param context 错误上下文描述
 * @param defaultCode 默认错误码
 * @returns 操作结果，失败时返回 undefined
 */
export async function withErrorHandling<T>(
  fn: () => Promise<T>,
  context: string,
  defaultCode: ErrorCode = ErrorCode.UNKNOWN
): Promise<T | undefined> {
  try {
    return await fn();
  } catch (error) {
    handleError(error, context, defaultCode);
    return undefined;
  }
}

/**
 * 包装同步操作，自动处理错误
 * @param fn 同步操作
 * @param context 错误上下文描述
 * @param defaultCode 默认错误码
 * @returns 操作结果，失败时返回 undefined
 */
export function withSyncErrorHandling<T>(
  fn: () => T,
  context: string,
  defaultCode: ErrorCode = ErrorCode.UNKNOWN
): T | undefined {
  try {
    return fn();
  } catch (error) {
    handleError(error, context, defaultCode);
    return undefined;
  }
}
