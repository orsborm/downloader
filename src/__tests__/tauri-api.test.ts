import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";
import {
  addTask,
  pauseTask,
  resumeTask,
  removeTask,
  getTask,
  getAllTasks,
  setTaskPriority,
  getSettings,
  updateSettings,
  getAppInfo,
  openFileLocation,
  getDownloadHistory,
  clearDownloadHistory,
  getRssFeeds,
  addRssFeed,
  removeRssFeed,
  listPlugins,
  installPlugin,
  uninstallPlugin,
  enablePlugin,
  disablePlugin,
  getArchiveConfig,
  updateArchiveConfig,
  extractArchive,
  getApiStatus,
  getBtStatus,
  pauseAllTasks,
  resumeAllTasks,
  removeCompletedTasks,
  resumeUnfinishedTasks,
  importOpml,
  exportOpml,
} from "../lib/tauri-api";

const mockInvoke = vi.mocked(invoke);

describe("Tauri API - Task Operations", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("addTask", () => {
    it("calls invoke with correct parameters", async () => {
      mockInvoke.mockResolvedValue("task-123");

      const result = await addTask({
        url: "https://example.com/file.zip",
        savePath: "/downloads",
        startImmediately: true,
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_task", {
        params: {
          url: "https://example.com/file.zip",
          savePath: "/downloads",
          startImmediately: true,
        },
      });
      expect(result).toBe("task-123");
    });

    it("handles optional parameters", async () => {
      mockInvoke.mockResolvedValue("task-456");

      await addTask({
        url: "https://example.com/file.zip",
        savePath: "/downloads",
        fileName: "custom-name.zip",
        proxy: "socks5://proxy:1080",
        speedLimit: 1024 * 1024,
        startImmediately: false,
      });

      expect(mockInvoke).toHaveBeenCalledWith("add_task", {
        params: {
          url: "https://example.com/file.zip",
          savePath: "/downloads",
          fileName: "custom-name.zip",
          proxy: "socks5://proxy:1080",
          speedLimit: 1048576,
          startImmediately: false,
        },
      });
    });

    it("propagates errors", async () => {
      mockInvoke.mockRejectedValue(new Error("Invalid URL"));

      await expect(
        addTask({
          url: "invalid",
          savePath: "/downloads",
          startImmediately: true,
        })
      ).rejects.toThrow("Invalid URL");
    });
  });

  describe("pauseTask", () => {
    it("calls invoke with task id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await pauseTask("task-123");
      expect(mockInvoke).toHaveBeenCalledWith("pause_task", { id: "task-123" });
    });
  });

  describe("resumeTask", () => {
    it("calls invoke with task id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await resumeTask("task-123");
      expect(mockInvoke).toHaveBeenCalledWith("resume_task", { id: "task-123" });
    });
  });

  describe("removeTask", () => {
    it("calls invoke with task id and deleteFiles=false by default", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await removeTask("task-123");
      expect(mockInvoke).toHaveBeenCalledWith("remove_task", {
        id: "task-123",
        deleteFiles: false,
      });
    });

    it("calls invoke with deleteFiles=true when specified", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await removeTask("task-123", true);
      expect(mockInvoke).toHaveBeenCalledWith("remove_task", {
        id: "task-123",
        deleteFiles: true,
      });
    });
  });

  describe("getTask", () => {
    it("returns task status", async () => {
      const task = {
        id: "task-123",
        name: "file.zip",
        protocol: "HTTP",
        state: "downloading",
        url: "https://example.com",
        savePath: "/downloads",
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
      };
      mockInvoke.mockResolvedValue(task);

      const result = await getTask("task-123");

      expect(mockInvoke).toHaveBeenCalledWith("get_task", { id: "task-123" });
      expect(result).toEqual(task);
    });

    it("returns null when task not found", async () => {
      mockInvoke.mockResolvedValue(null);
      const result = await getTask("nonexistent");
      expect(result).toBeNull();
    });
  });

  describe("getAllTasks", () => {
    it("returns array of tasks", async () => {
      const tasks = [
        { id: "1", name: "file1.zip", state: "downloading" },
        { id: "2", name: "file2.zip", state: "done" },
      ];
      mockInvoke.mockResolvedValue(tasks);

      const result = await getAllTasks();

      expect(mockInvoke).toHaveBeenCalledWith("get_all_tasks");
      expect(result).toHaveLength(2);
    });

    it("returns empty array when no tasks", async () => {
      mockInvoke.mockResolvedValue([]);
      const result = await getAllTasks();
      expect(result).toEqual([]);
    });
  });

  describe("setTaskPriority", () => {
    it("calls invoke with id and priority", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await setTaskPriority("task-123", 2);
      expect(mockInvoke).toHaveBeenCalledWith("set_priority", {
        id: "task-123",
        priority: 2,
      });
    });
  });
});

describe("Tauri API - Batch Operations", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("pauseAllTasks", () => {
    it("returns list of paused task ids", async () => {
      mockInvoke.mockResolvedValue(["task-1", "task-2"]);
      const result = await pauseAllTasks();
      expect(mockInvoke).toHaveBeenCalledWith("pause_all_tasks");
      expect(result).toEqual(["task-1", "task-2"]);
    });
  });

  describe("resumeAllTasks", () => {
    it("returns list of resumed task ids", async () => {
      mockInvoke.mockResolvedValue(["task-1"]);
      const result = await resumeAllTasks();
      expect(mockInvoke).toHaveBeenCalledWith("resume_all_tasks");
      expect(result).toEqual(["task-1"]);
    });
  });

  describe("removeCompletedTasks", () => {
    it("returns list of removed task ids", async () => {
      mockInvoke.mockResolvedValue(["task-done-1", "task-done-2"]);
      const result = await removeCompletedTasks();
      expect(mockInvoke).toHaveBeenCalledWith("remove_completed_tasks");
      expect(result).toHaveLength(2);
    });
  });

  describe("resumeUnfinishedTasks", () => {
    it("returns list of resumed task ids", async () => {
      mockInvoke.mockResolvedValue(["task-1"]);
      const result = await resumeUnfinishedTasks();
      expect(mockInvoke).toHaveBeenCalledWith("resume_unfinished_tasks");
      expect(result).toEqual(["task-1"]);
    });
  });
});

describe("Tauri API - Settings", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("getSettings", () => {
    it("returns app config", async () => {
      const config = {
        general: { language: "zh-CN", theme: "system" },
        download: { maxConcurrentTasks: 3 },
      };
      mockInvoke.mockResolvedValue(config);

      const result = await getSettings();

      expect(mockInvoke).toHaveBeenCalledWith("get_settings");
      expect(result).toEqual(config);
    });
  });

  describe("updateSettings", () => {
    it("calls invoke with new config", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const config = {
        general: { language: "en", theme: "dark" },
      } as any;

      await updateSettings(config);

      expect(mockInvoke).toHaveBeenCalledWith("update_settings", {
        newConfig: config,
      });
    });
  });

  describe("getAppInfo", () => {
    it("returns app info", async () => {
      const info = { version: "1.0.0", platform: "windows", arch: "x86_64" };
      mockInvoke.mockResolvedValue(info);

      const result = await getAppInfo();

      expect(mockInvoke).toHaveBeenCalledWith("get_app_info");
      expect(result.version).toBe("1.0.0");
    });
  });
});

describe("Tauri API - Status", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("getBtStatus", () => {
    it("returns BT engine status", async () => {
      const status = {
        dhtNodes: 150,
        dhtConnected: true,
        uploadSpeed: 1024,
        downloadSpeed: 2048,
      };
      mockInvoke.mockResolvedValue(status);

      const result = await getBtStatus();

      expect(mockInvoke).toHaveBeenCalledWith("get_bt_status");
      expect(result.dhtNodes).toBe(150);
      expect(result.dhtConnected).toBe(true);
    });
  });

  describe("getApiStatus", () => {
    it("returns API service status", async () => {
      const status = {
        enabled: true,
        host: "127.0.0.1",
        port: 6800,
        endpoint: "/jsonrpc",
        wsConnections: 2,
      };
      mockInvoke.mockResolvedValue(status);

      const result = await getApiStatus();

      expect(mockInvoke).toHaveBeenCalledWith("get_api_status");
      expect(result.port).toBe(6800);
    });
  });
});

describe("Tauri API - History", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("getDownloadHistory", () => {
    it("calls invoke without limit", async () => {
      mockInvoke.mockResolvedValue([]);
      await getDownloadHistory();
      expect(mockInvoke).toHaveBeenCalledWith("get_download_history", {
        limit: undefined,
      });
    });

    it("calls invoke with limit", async () => {
      mockInvoke.mockResolvedValue([]);
      await getDownloadHistory(50);
      expect(mockInvoke).toHaveBeenCalledWith("get_download_history", {
        limit: 50,
      });
    });
  });

  describe("clearDownloadHistory", () => {
    it("returns count of cleared items", async () => {
      mockInvoke.mockResolvedValue(15);
      const result = await clearDownloadHistory();
      expect(mockInvoke).toHaveBeenCalledWith("clear_download_history");
      expect(result).toBe(15);
    });
  });
});

describe("Tauri API - RSS", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("getRssFeeds", () => {
    it("returns feeds list", async () => {
      mockInvoke.mockResolvedValue([]);
      const result = await getRssFeeds();
      expect(mockInvoke).toHaveBeenCalledWith("get_rss_feeds");
      expect(result).toEqual([]);
    });
  });

  describe("addRssFeed", () => {
    it("calls invoke with name, url, interval", async () => {
      mockInvoke.mockResolvedValue("feed-123");
      const result = await addRssFeed("Test Feed", "https://example.com/feed.xml", 30);
      expect(mockInvoke).toHaveBeenCalledWith("add_rss_feed", {
        name: "Test Feed",
        url: "https://example.com/feed.xml",
        interval: 30,
      });
      expect(result).toBe("feed-123");
    });
  });

  describe("removeRssFeed", () => {
    it("calls invoke with feed id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await removeRssFeed("feed-123");
      expect(mockInvoke).toHaveBeenCalledWith("remove_rss_feed", { id: "feed-123" });
    });
  });
});

describe("Tauri API - Plugins", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("listPlugins", () => {
    it("returns plugins list", async () => {
      mockInvoke.mockResolvedValue([]);
      const result = await listPlugins();
      expect(mockInvoke).toHaveBeenCalledWith("list_plugins");
      expect(result).toEqual([]);
    });
  });

  describe("installPlugin", () => {
    it("calls invoke with wasm path", async () => {
      mockInvoke.mockResolvedValue("plugin-123");
      const result = await installPlugin("/path/to/plugin.wasm");
      expect(mockInvoke).toHaveBeenCalledWith("install_plugin", {
        wasmPath: "/path/to/plugin.wasm",
      });
      expect(result).toBe("plugin-123");
    });
  });

  describe("uninstallPlugin", () => {
    it("calls invoke with plugin id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await uninstallPlugin("plugin-123");
      expect(mockInvoke).toHaveBeenCalledWith("uninstall_plugin", { id: "plugin-123" });
    });
  });

  describe("enablePlugin", () => {
    it("calls invoke with plugin id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await enablePlugin("plugin-123");
      expect(mockInvoke).toHaveBeenCalledWith("enable_plugin", { id: "plugin-123" });
    });
  });

  describe("disablePlugin", () => {
    it("calls invoke with plugin id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await disablePlugin("plugin-123");
      expect(mockInvoke).toHaveBeenCalledWith("disable_plugin", { id: "plugin-123" });
    });
  });
});

describe("Tauri API - Archive", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("getArchiveConfig", () => {
    it("returns archive config", async () => {
      const config = {
        autoExtract: true,
        deleteAfterExtract: false,
        extractDir: "/downloads/extracted",
        passwords: [],
      };
      mockInvoke.mockResolvedValue(config);
      const result = await getArchiveConfig();
      expect(mockInvoke).toHaveBeenCalledWith("get_archive_config");
      expect(result).toEqual(config);
    });
  });

  describe("updateArchiveConfig", () => {
    it("calls invoke with new config", async () => {
      mockInvoke.mockResolvedValue(undefined);
      const config = {
        autoExtract: false,
        deleteAfterExtract: true,
        extractDir: "/downloads/extracted",
        passwords: [],
      };
      await updateArchiveConfig(config);
      expect(mockInvoke).toHaveBeenCalledWith("update_archive_config", { config });
    });
  });

  describe("extractArchive", () => {
    it("calls invoke with file path", async () => {
      mockInvoke.mockResolvedValue(["file1.txt", "file2.txt"]);
      const result = await extractArchive("/downloads/archive.zip");
      expect(mockInvoke).toHaveBeenCalledWith("extract_archive", {
        filePath: "/downloads/archive.zip",
        password: undefined,
      });
      expect(result).toEqual(["file1.txt", "file2.txt"]);
    });

    it("calls invoke with password", async () => {
      mockInvoke.mockResolvedValue(["file1.txt"]);
      await extractArchive("/downloads/protected.zip", "secret123");
      expect(mockInvoke).toHaveBeenCalledWith("extract_archive", {
        filePath: "/downloads/protected.zip",
        password: "secret123",
      });
    });
  });
});

describe("Tauri API - File Operations", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("openFileLocation", () => {
    it("calls invoke with task id", async () => {
      mockInvoke.mockResolvedValue(undefined);
      await openFileLocation("task-123");
      expect(mockInvoke).toHaveBeenCalledWith("open_file_location", { id: "task-123" });
    });
  });
});

describe("Tauri API - OPML", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  describe("importOpml", () => {
    it("calls invoke with OPML content", async () => {
      mockInvoke.mockResolvedValue(3);
      const content = '<?xml version="1.0"?><opml><body><outline text="feed"/></body></opml>';
      const result = await importOpml(content);
      expect(mockInvoke).toHaveBeenCalledWith("import_opml", { content });
      expect(result).toBe(3);
    });

    it("returns count of imported feeds", async () => {
      mockInvoke.mockResolvedValue(0);
      const result = await importOpml("<opml></opml>");
      expect(result).toBe(0);
    });
  });

  describe("exportOpml", () => {
    it("calls invoke and returns OPML string", async () => {
      const opml = '<?xml version="1.0"?><opml><body><outline text="feed"/></body></opml>';
      mockInvoke.mockResolvedValue(opml);
      const result = await exportOpml();
      expect(mockInvoke).toHaveBeenCalledWith("export_opml");
      expect(result).toBe(opml);
    });

    it("propagates errors", async () => {
      mockInvoke.mockRejectedValue(new Error("Export failed"));
      await expect(exportOpml()).rejects.toThrow("Export failed");
    });
  });
});
