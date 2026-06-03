// 共享类型定义
// 桌面端和 WebUI 共用的核心接口

/** 协议类型 */
export type Protocol = "HTTP" | "FTP" | "BT" | "MAGNET" | "ED2K" | "HLS" | "DASH";

/** 任务状态 */
export type TaskState =
  | "queued"
  | "downloading"
  | "paused"
  | "seeding"
  | "done"
  | "error";

/** 子文件信息 */
export interface FileInfo {
  index: number;
  path: string;
  size: number;
  priority: number; // 0=跳过 1=正常 2=高
}

/** 任务完整状态（桌面端 IPC 格式） */
export interface TaskStatus {
  id: string;
  name: string;
  protocol: Protocol;
  state: TaskState;
  url: string;
  savePath: string;
  totalSize: number;
  downloaded: number;
  uploaded: number;
  downloadSpeed: number;
  uploadSpeed: number;
  progress: number; // 0.0 ~ 1.0
  peers: number;
  seeds: number;
  eta: number | null;
  files: FileInfo[];
  error: string | null;
  addedAt: string;
  completedAt: string | null;
  priority: number;
  downloadLimit: number; // 单任务下载速度限制 bytes/sec，0=不限速
  uploadLimit: number;   // 单任务上传速度限制 bytes/sec，0=不限速
  mirrorUrls?: string[]; // 镜像URL列表
  activeSource?: string; // 当前使用的源
}

/** WebUI 任务状态（JSON-RPC 格式） */
export interface TaskInfo {
  taskId: string;
  name: string;
  status: string;
  totalLength: number;
  completedLength: number;
  downloadSpeed: number;
  uploadSpeed: number;
  progress: number;
  dir: string;
  connections: number;
  eta: number;
}

/** 添加任务参数 */
export interface TaskParams {
  url: string;
  savePath: string;
  fileName?: string;
  proxy?: string;
  speedLimit?: number;
  startImmediately: boolean;
  onlyFiles?: number[]; // BT/Magnet 任务的文件选择
  mirrorUrls?: string[]; // 镜像URL列表，用于多源加速下载
}

/** 镜像源信息 */
export interface MirrorInfo {
  url: string;
  status: "active" | "failed" | "pending";
  speed: number;
  lastChecked: string;
}

/** 任务状态更新事件（从后端推送） */
export interface TaskUpdateEvent {
  taskId: string;
  state: TaskState;
  downloaded: number;
  totalSize: number;
  downloadSpeed: number;
  uploadSpeed: number;
  progress: number;
  peers: number;
  error: string | null;
  eta: number | null;
}

/** 下载历史记录 */
export interface DownloadHistory {
  id: string;
  taskId: string | null;
  name: string;
  url: string | null;
  protocol: string;
  savePath: string;
  filePath: string | null;
  totalSize: number;
  downloaded: number;
  averageSpeed: number;
  duration: number;
  completedAt: string;
}

/** 全局统计（WebUI） */
export interface GlobalStat {
  downloadSpeed: number;
  uploadSpeed: number;
  numActive: number;
  numWaiting: number;
  numStopped: number;
}

/** 速度历史数据点 */
export interface SpeedDataPoint {
  time: number;
  download: number;
  upload: number;
}

/** 应用配置 */
export interface AppConfig {
  general: {
    language: string;
    theme: string;
    minimizeToTray: boolean;
    closeToTray: boolean;
    autoStart: boolean;
    fontSize: number;
  };
  download: {
    defaultDir: string;
    completeDir: string;
    tempDir: string;
    maxConcurrentTasks: number;
    maxConnectionsPerTask: number;
    maxGlobalConnections: number;
    maxUploadSpeed: number;
    maxDownloadSpeed: number;
    autoRetryCount: number;
    autoRetryInterval: number;
  };
  connection: {
    btPort: number;
    ed2kPort: number;
    httpPort: number;
    apiToken: string;
    upnp: boolean;
    natPmp: boolean;
    proxyType: string;
    proxyHost: string;
    proxyPort: number;
    proxyUsername: string;
    proxyPassword: string;
    connectionTimeout: number;
    readTimeout: number;
    ed2kServers: string[];
  };
  bt: {
    dht: boolean;
    pex: boolean;
    lsd: boolean;
    encryption: string;
    seedRatioLimit: number;
    seedTimeLimit: number;
    trackersFile: string;
    stopSeeding: boolean;
  };
  http: {
    userAgent: string;
    cookiePolicy: string;
    refererPolicy: string;
    maxRedirects: number;
    verifySsl: boolean;
  };
  notification: {
    taskComplete: boolean;
    taskError: boolean;
    sound: boolean;
    position: string;
    duration: number;
  };
  advanced: {
    logLevel: string;
    logMaxSize: number;
    logRetainDays: number;
    dbBackupInterval: number;
    tempCleanupInterval: number;
    memoryLimit: number;
    postDownloadAction: string;
    postDownloadCommand: string;
  };
}
