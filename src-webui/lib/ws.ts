// WebSocket 连接管理
// 自动重连、心跳保活、事件分发

type WsEventHandler = (event: { event: string; data: unknown; timestamp: number }) => void;

interface WsManagerOptions {
  url: string;
  onEvent?: WsEventHandler;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (e: Event) => void;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
  heartbeatInterval?: number;
}

export class WsManager {
  private ws: WebSocket | null = null;
  private url: string;
  private onEvent?: WsEventHandler;
  private onOpen?: () => void;
  private onClose?: () => void;
  private onError?: (e: Event) => void;
  private reconnectInterval: number;
  private maxReconnectAttempts: number;
  private heartbeatInterval: number;
  private reconnectAttempts = 0;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private disposed = false;

  constructor(options: WsManagerOptions) {
    this.url = options.url;
    this.onEvent = options.onEvent;
    this.onOpen = options.onOpen;
    this.onClose = options.onClose;
    this.onError = options.onError;
    this.reconnectInterval = options.reconnectInterval ?? 3000;
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 10;
    this.heartbeatInterval = options.heartbeatInterval ?? 30000;
  }

  connect(): void {
    if (this.disposed) return;

    try {
      this.ws = new WebSocket(this.url);
    } catch {
      this.scheduleReconnect();
      return;
    }

    this.ws.onopen = () => {
      this.reconnectAttempts = 0;
      this.startHeartbeat();
      this.onOpen?.();
    };

    this.ws.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data);
        this.onEvent?.(data);
      } catch {
        // ignore parse errors
      }
    };

    this.ws.onerror = (e) => {
      this.onError?.(e);
    };

    this.ws.onclose = () => {
      this.stopHeartbeat();
      this.onClose?.();
      this.scheduleReconnect();
    };
  }

  disconnect(): void {
    this.disposed = true;
    this.stopHeartbeat();
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  send(data: unknown): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(typeof data === "string" ? data : JSON.stringify(data));
    }
  }

  get connected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private scheduleReconnect(): void {
    if (this.disposed) return;
    if (this.reconnectAttempts >= this.maxReconnectAttempts) return;

    this.reconnectAttempts++;
    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, this.reconnectInterval);
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.heartbeatTimer = setInterval(() => {
      this.send({ type: "ping" });
    }, this.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }
}

/** 创建 WebSocket 连接的便捷函数 */
export function createWebSocket(
  url: string,
  onEvent: WsEventHandler,
  options?: Partial<Omit<WsManagerOptions, "url" | "onEvent">>,
): WsManager {
  const manager = new WsManager({ url, onEvent, ...options });
  manager.connect();
  return manager;
}
