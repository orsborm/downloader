// WebUI JSON-RPC API 客户端
// 通过 HTTP/WS 与下载器后端通信

const API_URL = `${window.location.protocol}//${window.location.hostname}:${window.location.port}/jsonrpc`;
const WS_URL = `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.hostname}:${window.location.port}/ws`;

/** JSON-RPC 2.0 请求 */
interface JsonRpcRequest {
  jsonrpc: "2.0";
  method: string;
  params?: unknown[];
  id: number;
}

/** JSON-RPC 2.0 响应 */
interface JsonRpcResponse<T = unknown> {
  jsonrpc: "2.0";
  result?: T;
  error?: { code: number; message: string };
  id: number;
}

/** JSON-RPC 请求 ID 计数器（避免同毫秒碰撞） */
let rpcIdCounter = 0;

/** 调用 JSON-RPC 方法 */
async function call<T = unknown>(method: string, params: unknown[] = []): Promise<T> {
  const request: JsonRpcRequest = {
    jsonrpc: "2.0",
    method,
    params,
    id: ++rpcIdCounter,
  };

  const response = await fetch(API_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });

  const data: JsonRpcResponse<T> = await response.json();

  if (data.error) {
    throw new Error(data.error.message);
  }

  return data.result as T;
}

// ==================== 任务管理 API ====================

/** 添加任务 */
export function addTask(urls: string[], options?: { dir?: string; fileName?: string }) {
  return call<string>("downloader.addTask", [urls, options || {}]);
}

/** 删除任务 */
export function removeTask(taskId: string) {
  return call<string>("downloader.removeTask", [taskId]);
}

/** 暂停任务 */
export function pauseTask(taskId: string) {
  return call<string>("downloader.pauseTask", [taskId]);
}

/** 恢复任务 */
export function resumeTask(taskId: string) {
  return call<string>("downloader.resumeTask", [taskId]);
}

/** 查询任务状态 */
export function tellStatus(taskId: string) {
  return call<Record<string, unknown>>("downloader.tellStatus", [taskId]);
}

/** 查询活跃任务 */
export function tellActive() {
  return call<Record<string, unknown>[]>("downloader.tellActive");
}

/** 查询等待中的任务 */
export function tellWaiting() {
  return call<Record<string, unknown>[]>("downloader.tellWaiting");
}

/** 查询已停止的任务 */
export function tellStopped() {
  return call<Record<string, unknown>[]>("downloader.tellStopped");
}

/** 获取全局统计 */
export function getGlobalStat() {
  return call<Record<string, unknown>>("downloader.getGlobalStat");
}

/** 获取版本 */
export function getVersion() {
  return call<{ version: string; enabledFeatures: string[] }>("downloader.getVersion");
}

/** 暂停所有 */
export function pauseAll() {
  return call<string>("downloader.pauseAll");
}

/** 恢复所有 */
export function unpauseAll() {
  return call<string>("downloader.unpauseAll");
}

/** 清除已完成 */
export function purgeCompleted() {
  return call<string>("downloader.purgeCompleted");
}

// ==================== WebSocket 事件订阅 ====================

export type WsEventHandler = (event: { event: string; data: unknown; timestamp: number }) => void;

/** 创建 WebSocket 连接（带自动重连） */
export function createWebSocket(onEvent: WsEventHandler): WebSocket {
  let ws: WebSocket;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let reconnectDelay = 1000;
  const maxDelay = 30000;

  function connect() {
    ws = new WebSocket(WS_URL);

    ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data);
        onEvent(data);
      } catch (e) {
        console.error("WebSocket 消息解析失败:", e);
      }
    };

    ws.onerror = (e) => {
      console.error("WebSocket 错误:", e);
    };

    ws.onclose = () => {
      // 自动重连（指数退避）
      reconnectTimer = setTimeout(() => {
        reconnectDelay = Math.min(reconnectDelay * 2, maxDelay);
        connect();
      }, reconnectDelay);
    };

    ws.onopen = () => {
      reconnectDelay = 1000; // 重连成功后重置延迟
    };
  }

  connect();

  // 返回一个包装对象，close 时清理重连定时器
  const wrapper = Object.assign(ws, {
    close: () => {
      if (reconnectTimer) clearTimeout(reconnectTimer);
      ws.close();
    },
  });

  return wrapper as WebSocket;
}
