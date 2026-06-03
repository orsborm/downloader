import { describe, it, expect } from "vitest";
import type {
  Protocol,
  TaskState,
  TaskStatus,
  TaskInfo,
  TaskParams,
  TaskUpdateEvent,
  DownloadHistory,
  GlobalStat,
  SpeedDataPoint,
  AppConfig,
} from "../shared/types";

import type {
  Protocol as DesktopProtocol,
  TaskStatus as DesktopTaskStatus,
} from "../lib/types";

describe("shared types re-export", () => {
  it("Protocol type is accessible from shared", () => {
    const protocol: Protocol = "HTTP";
    expect(protocol).toBe("HTTP");
  });

  it("TaskState type covers all states", () => {
    const states: TaskState[] = [
      "queued",
      "downloading",
      "paused",
      "seeding",
      "done",
      "error",
    ];
    expect(states).toHaveLength(6);
  });

  it("TaskStatus interface has all required fields", () => {
    const status: TaskStatus = {
      id: "1",
      name: "test.zip",
      protocol: "HTTP",
      state: "downloading",
      url: "http://example.com",
      savePath: "/tmp",
      totalSize: 1024,
      downloaded: 512,
      uploaded: 0,
      downloadSpeed: 100,
      uploadSpeed: 0,
      progress: 0.5,
      peers: 1,
      seeds: 0,
      eta: 10,
      files: [],
      error: null,
      addedAt: "2026-01-01",
      completedAt: null,
      priority: 1,
      downloadLimit: 0,
      uploadLimit: 0,
    };
    expect(status.id).toBe("1");
    expect(status.protocol).toBe("HTTP");
  });

  it("desktop re-exports match shared types", () => {
    const protocol: DesktopProtocol = "BT";
    const status: DesktopTaskStatus = {
      id: "1",
      name: "test",
      protocol: "HTTP",
      state: "downloading",
      url: "http://example.com",
      savePath: "/tmp",
      totalSize: 1024,
      downloaded: 512,
      uploaded: 0,
      downloadSpeed: 100,
      uploadSpeed: 0,
      progress: 0.5,
      peers: 1,
      seeds: 0,
      eta: 10,
      files: [],
      error: null,
      addedAt: "2026-01-01",
      completedAt: null,
      priority: 1,
      downloadLimit: 0,
      uploadLimit: 0,
    };
    expect(protocol).toBe("BT");
    expect(status.id).toBe("1");
  });

  it("TaskInfo interface has WebUI fields", () => {
    const info: TaskInfo = {
      taskId: "abc",
      name: "file.zip",
      status: "active",
      totalLength: 1024,
      completedLength: 512,
      downloadSpeed: 100,
      uploadSpeed: 0,
      progress: 0.5,
      dir: "/downloads",
      connections: 5,
      eta: 10,
    };
    expect(info.taskId).toBe("abc");
  });

  it("TaskParams interface has required fields", () => {
    const params: TaskParams = {
      url: "http://example.com/file.zip",
      savePath: "/downloads",
      startImmediately: true,
    };
    expect(params.url).toContain("example.com");
  });

  it("TaskUpdateEvent interface has update fields", () => {
    const event: TaskUpdateEvent = {
      taskId: "abc",
      state: "downloading",
      downloaded: 512,
      totalSize: 1024,
      downloadSpeed: 100,
      uploadSpeed: 0,
      progress: 0.5,
      peers: 1,
      error: null,
      eta: 10,
    };
    expect(event.taskId).toBe("abc");
    expect(event.state).toBe("downloading");
  });

  it("DownloadHistory interface has history fields", () => {
    const history: DownloadHistory = {
      id: "h1",
      taskId: "t1",
      name: "file.zip",
      url: "http://example.com",
      protocol: "HTTP",
      savePath: "/downloads",
      filePath: "/downloads/file.zip",
      totalSize: 1024,
      downloaded: 1024,
      averageSpeed: 100,
      duration: 10,
      completedAt: "2026-01-01",
    };
    expect(history.name).toBe("file.zip");
  });

  it("AppConfig interface has all sections", () => {
    const config: AppConfig = {
      general: { language: "zh-CN", theme: "system", minimizeToTray: true, closeToTray: false, autoStart: false, fontSize: 14 },
      download: { defaultDir: "/downloads", completeDir: "/complete", tempDir: "/temp", maxConcurrentTasks: 3, maxConnectionsPerTask: 64, maxGlobalConnections: 200, maxUploadSpeed: 0, maxDownloadSpeed: 0, autoRetryCount: 3, autoRetryInterval: 5 },
      connection: { btPort: 6881, ed2kPort: 4661, httpPort: 0, apiToken: "", upnp: true, natPmp: true, proxyType: "none", proxyHost: "", proxyPort: 0, proxyUsername: "", proxyPassword: "", connectionTimeout: 30, readTimeout: 60, ed2kServers: ["91.200.42.46:4661", "176.103.48.41:4661"] },
      bt: { dht: true, pex: true, lsd: true, encryption: "enabled", seedRatioLimit: 2.0, seedTimeLimit: 1440, trackersFile: "", stopSeeding: false },
      http: { userAgent: "test", cookiePolicy: "auto", refererPolicy: "strict", maxRedirects: 10, verifySsl: true },
      notification: { taskComplete: true, taskError: true, sound: true, position: "bottom-right", duration: 5 },
      advanced: { logLevel: "info", logMaxSize: 10, logRetainDays: 7, dbBackupInterval: 24, tempCleanupInterval: 1, memoryLimit: 0, postDownloadAction: "none", postDownloadCommand: "" },
    };
    expect(config.general.language).toBe("zh-CN");
    expect(config.download.maxConcurrentTasks).toBe(3);
  });

  it("SpeedDataPoint interface has time/download/upload", () => {
    const point: SpeedDataPoint = {
      time: Date.now(),
      download: 1024,
      upload: 512,
    };
    expect(point.download).toBeGreaterThan(point.upload);
  });

  it("GlobalStat interface has all fields", () => {
    const stat: GlobalStat = {
      downloadSpeed: 1024,
      uploadSpeed: 512,
      numActive: 2,
      numWaiting: 1,
      numStopped: 5,
    };
    expect(stat.numActive + stat.numWaiting + stat.numStopped).toBe(8);
  });
});
