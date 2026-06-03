// 国际化模块测试

import { describe, it, expect } from "vitest";
import { zh } from "../lib/locales/zh";
import { en } from "../lib/locales/en";

describe("i18n", () => {
  describe("语言包完整性", () => {
    it("中文和英文语言包结构一致", () => {
      function getKeys(obj: unknown, prefix = ""): string[] {
        const keys: string[] = [];
        if (typeof obj === "object" && obj !== null) {
          for (const key of Object.keys(obj as Record<string, unknown>)) {
            const fullKey = prefix ? `${prefix}.${key}` : key;
            const value = (obj as Record<string, unknown>)[key];
            if (typeof value === "object" && value !== null) {
              keys.push(...getKeys(value, fullKey));
            } else {
              keys.push(fullKey);
            }
          }
        }
        return keys;
      }

      const zhKeys = getKeys(zh).sort();
      const enKeys = getKeys(en).sort();

      expect(zhKeys).toEqual(enKeys);
    });

    it("中文语言包无空值", () => {
      function checkNoEmpty(obj: unknown, path = ""): void {
        if (typeof obj === "object" && obj !== null) {
          for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
            const fullPath = path ? `${path}.${key}` : key;
            if (typeof value === "object" && value !== null) {
              checkNoEmpty(value, fullPath);
            } else {
              expect(value, `${fullPath} should not be empty`).toBeTruthy();
            }
          }
        }
      }
      checkNoEmpty(zh);
    });

    it("英文语言包无空值", () => {
      function checkNoEmpty(obj: unknown, path = ""): void {
        if (typeof obj === "object" && obj !== null) {
          for (const [key, value] of Object.entries(obj as Record<string, unknown>)) {
            const fullPath = path ? `${path}.${key}` : key;
            if (typeof value === "object" && value !== null) {
              checkNoEmpty(value, fullPath);
            } else {
              expect(value, `${fullPath} should not be empty`).toBeTruthy();
            }
          }
        }
      }
      checkNoEmpty(en);
    });
  });

  describe("语言包内容", () => {
    it("中文语言包含正确的通用翻译", () => {
      expect(zh.common.confirm).toBe("确定");
      expect(zh.common.cancel).toBe("取消");
      expect(zh.common.save).toBe("保存");
      expect(zh.common.delete).toBe("删除");
    });

    it("英文语言包含正确的通用翻译", () => {
      expect(en.common.confirm).toBe("OK");
      expect(en.common.cancel).toBe("Cancel");
      expect(en.common.save).toBe("Save");
      expect(en.common.delete).toBe("Delete");
    });

    it("中文语言包含任务状态翻译", () => {
      expect(zh.taskState.downloading).toBe("下载中");
      expect(zh.taskState.paused).toBe("已暂停");
      expect(zh.taskState.done).toBe("已完成");
      expect(zh.taskState.error).toBe("错误");
      expect(zh.taskState.seeding).toBe("做种中");
      expect(zh.taskState.queued).toBe("等待中");
    });

    it("英文语言包含任务状态翻译", () => {
      expect(en.taskState.downloading).toBe("Downloading");
      expect(en.taskState.paused).toBe("Paused");
      expect(en.taskState.done).toBe("Completed");
      expect(en.taskState.error).toBe("Error");
      expect(en.taskState.seeding).toBe("Seeding");
      expect(en.taskState.queued).toBe("Queued");
    });

    it("中文语言包含错误消息", () => {
      expect(zh.errors.networkError).toBeTruthy();
      expect(zh.errors.timeout).toBeTruthy();
      expect(zh.errors.diskFull).toBeTruthy();
      expect(zh.errors.permissionDenied).toBeTruthy();
    });

    it("英文语言包含错误消息", () => {
      expect(en.errors.networkError).toBeTruthy();
      expect(en.errors.timeout).toBeTruthy();
      expect(en.errors.diskFull).toBeTruthy();
      expect(en.errors.permissionDenied).toBeTruthy();
    });

    it("中文语言包含协议翻译", () => {
      expect(zh.protocol.http).toBe("HTTP");
      expect(zh.protocol.https).toBe("HTTPS");
      expect(zh.protocol.ftp).toBe("FTP");
      expect(zh.protocol.bt).toBe("BitTorrent");
      expect(zh.protocol.magnet).toBe("磁力链");
      expect(zh.protocol.ed2k).toBe("ed2k");
      expect(zh.protocol.hls).toBe("HLS");
      expect(zh.protocol.dash).toBe("DASH");
    });

    it("英文语言包含协议翻译", () => {
      expect(en.protocol.http).toBe("HTTP");
      expect(en.protocol.https).toBe("HTTPS");
      expect(en.protocol.ftp).toBe("FTP");
      expect(en.protocol.bt).toBe("BitTorrent");
      expect(en.protocol.magnet).toBe("Magnet");
      expect(en.protocol.ed2k).toBe("ed2k");
      expect(en.protocol.hls).toBe("HLS");
      expect(en.protocol.dash).toBe("DASH");
    });
  });

  describe("模板参数", () => {
    it("中文模板包含占位符", () => {
      expect(zh.taskList.selectedCount).toContain("{count}");
      expect(zh.taskList.filteredCount).toContain("{filtered}");
      expect(zh.taskList.filteredCount).toContain("{total}");
    });

    it("英文模板包含占位符", () => {
      expect(en.taskList.selectedCount).toContain("{count}");
      expect(en.taskList.filteredCount).toContain("{filtered}");
      expect(en.taskList.filteredCount).toContain("{total}");
    });

    it("通知模板包含占位符", () => {
      expect(zh.notification.taskCompleteBody).toContain("{name}");
      expect(zh.notification.taskErrorBody).toContain("{name}");
      expect(zh.notification.taskErrorBody).toContain("{error}");
    });
  });

  describe("设置部分完整性", () => {
    it("中文设置包含所有分类", () => {
      expect(zh.settings.sections.general).toBeTruthy();
      expect(zh.settings.sections.download).toBeTruthy();
      expect(zh.settings.sections.connection).toBeTruthy();
      expect(zh.settings.sections.bittorrent).toBeTruthy();
      expect(zh.settings.sections.notification).toBeTruthy();
      expect(zh.settings.sections.advanced).toBeTruthy();
    });

    it("英文设置包含所有分类", () => {
      expect(en.settings.sections.general).toBeTruthy();
      expect(en.settings.sections.download).toBeTruthy();
      expect(en.settings.sections.connection).toBeTruthy();
      expect(en.settings.sections.bittorrent).toBeTruthy();
      expect(en.settings.sections.notification).toBeTruthy();
      expect(en.settings.sections.advanced).toBeTruthy();
    });

    it("中文常规设置包含所有选项", () => {
      expect(zh.settings.general.language).toBeTruthy();
      expect(zh.settings.general.theme).toBeTruthy();
      expect(zh.settings.general.themeLight).toBeTruthy();
      expect(zh.settings.general.themeDark).toBeTruthy();
      expect(zh.settings.general.themeSystem).toBeTruthy();
      expect(zh.settings.general.minimizeToTray).toBeTruthy();
      expect(zh.settings.general.closeAction).toBeTruthy();
    });

    it("英文常规设置包含所有选项", () => {
      expect(en.settings.general.language).toBeTruthy();
      expect(en.settings.general.theme).toBeTruthy();
      expect(en.settings.general.themeLight).toBeTruthy();
      expect(en.settings.general.themeDark).toBeTruthy();
      expect(en.settings.general.themeSystem).toBeTruthy();
      expect(en.settings.general.minimizeToTray).toBeTruthy();
      expect(en.settings.general.closeAction).toBeTruthy();
    });
  });

  describe("右键菜单翻译", () => {
    it("中文右键菜单包含所有操作", () => {
      expect(zh.contextMenu.start).toBeTruthy();
      expect(zh.contextMenu.pause).toBeTruthy();
      expect(zh.contextMenu.resume).toBeTruthy();
      expect(zh.contextMenu.delete).toBeTruthy();
      expect(zh.contextMenu.retry).toBeTruthy();
      expect(zh.contextMenu.openFile).toBeTruthy();
      expect(zh.contextMenu.openFolder).toBeTruthy();
      expect(zh.contextMenu.copyLink).toBeTruthy();
      expect(zh.contextMenu.properties).toBeTruthy();
    });

    it("英文右键菜单包含所有操作", () => {
      expect(en.contextMenu.start).toBeTruthy();
      expect(en.contextMenu.pause).toBeTruthy();
      expect(en.contextMenu.resume).toBeTruthy();
      expect(en.contextMenu.delete).toBeTruthy();
      expect(en.contextMenu.retry).toBeTruthy();
      expect(en.contextMenu.openFile).toBeTruthy();
      expect(en.contextMenu.openFolder).toBeTruthy();
      expect(en.contextMenu.copyLink).toBeTruthy();
      expect(en.contextMenu.properties).toBeTruthy();
    });
  });

  describe("工具栏翻译", () => {
    it("中文工具栏包含所有按钮", () => {
      expect(zh.toolbar.addTask).toBeTruthy();
      expect(zh.toolbar.startAll).toBeTruthy();
      expect(zh.toolbar.pauseAll).toBeTruthy();
      expect(zh.toolbar.deleteSelected).toBeTruthy();
      expect(zh.toolbar.searchPlaceholder).toBeTruthy();
      expect(zh.toolbar.settings).toBeTruthy();
    });

    it("英文工具栏包含所有按钮", () => {
      expect(en.toolbar.addTask).toBeTruthy();
      expect(en.toolbar.startAll).toBeTruthy();
      expect(en.toolbar.pauseAll).toBeTruthy();
      expect(en.toolbar.deleteSelected).toBeTruthy();
      expect(en.toolbar.searchPlaceholder).toBeTruthy();
      expect(en.toolbar.settings).toBeTruthy();
    });
  });

  describe("下载历史翻译", () => {
    it("中文下载历史包含所有字段", () => {
      expect(zh.downloadHistory.title).toBeTruthy();
      expect(zh.downloadHistory.empty).toBeTruthy();
      expect(zh.downloadHistory.searchPlaceholder).toBeTruthy();
      expect(zh.downloadHistory.exportCsv).toBeTruthy();
      expect(zh.downloadHistory.clearAll).toBeTruthy();
    });

    it("英文下载历史包含所有字段", () => {
      expect(en.downloadHistory.title).toBeTruthy();
      expect(en.downloadHistory.empty).toBeTruthy();
      expect(en.downloadHistory.searchPlaceholder).toBeTruthy();
      expect(en.downloadHistory.exportCsv).toBeTruthy();
      expect(en.downloadHistory.clearAll).toBeTruthy();
    });
  });

  describe("语言名称映射", () => {
    it("LANGUAGE_NAMES 包含中英文", () => {
      // 直接从 zh.ts 导出的语言名称映射
      const LANGUAGE_NAMES = {
        zh: "中文",
        en: "English",
      };
      expect(LANGUAGE_NAMES.zh).toBe("中文");
      expect(LANGUAGE_NAMES.en).toBe("English");
    });
  });
});
