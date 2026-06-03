// 剪贴板监听 Hook
// 定期检查剪贴板内容，识别下载链接并通知用户

import { useEffect, useRef, useCallback } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";

/** 识别 URL 中的下载协议 */
function detectDownloadUrl(text: string): string | null {
  const trimmed = text.trim();
  // 匹配 magnet / ed2k / http(s) / ftp 链接
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

/**
 * 剪贴板监听 Hook
 * @param onDetect 检测到下载链接时的回调
 * @param interval 检查间隔（毫秒），默认 2000ms
 * @param cooldown 检测后冷却时间（毫秒），默认 10000ms，避免重复触发
 */
export function useClipboard(
  onDetect: (url: string) => void,
  interval: number = 2000,
  cooldown: number = 10000
) {
  const lastContentRef = useRef<string>("");
  const lastDetectRef = useRef<number>(0);

  const checkClipboard = useCallback(async () => {
    try {
      const text = await readText();
      if (text && text !== lastContentRef.current) {
        lastContentRef.current = text;
        const now = Date.now();
        if (now - lastDetectRef.current < cooldown) return;
        const url = detectDownloadUrl(text);
        if (url) {
          lastDetectRef.current = now;
          onDetect(url);
        }
      }
    } catch {
      // 剪贴板读取失败（可能无权限），忽略
    }
  }, [onDetect, cooldown]);

  useEffect(() => {
    // 首次轮询：读取当前剪贴板内容作为基线，避免启动时误触发
    readText()
      .then((text) => {
        if (text) lastContentRef.current = text;
      })
      .catch(() => {});

    // 窗口可见时才轮询，隐藏时暂停（节省 IPC 开销）
    let timer: ReturnType<typeof setInterval> | null = null;
    const startPolling = () => {
      if (!timer) timer = setInterval(checkClipboard, interval);
    };
    const stopPolling = () => {
      if (timer) { clearInterval(timer); timer = null; }
    };

    const handleVisibility = () => {
      if (document.hidden) stopPolling(); else startPolling();
    };

    document.addEventListener("visibilitychange", handleVisibility);
    // 初始状态：如果可见则开始轮询
    if (!document.hidden) startPolling();

    return () => {
      stopPolling();
      document.removeEventListener("visibilitychange", handleVisibility);
    };
  }, [checkClipboard, interval]);
}
