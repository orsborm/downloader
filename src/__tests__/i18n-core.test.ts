import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock localStorage
const store: Record<string, string> = {};
const localStorageMock = {
  getItem: vi.fn((key: string) => store[key] ?? null),
  setItem: vi.fn((key: string, value: string) => { store[key] = value; }),
  removeItem: vi.fn((key: string) => { delete store[key]; }),
  clear: vi.fn(() => { for (const k in store) delete store[k]; }),
};
Object.defineProperty(globalThis, "localStorage", { value: localStorageMock });

// Mock navigator.language
Object.defineProperty(globalThis, "navigator", {
  value: { language: "zh-CN" },
  writable: true,
});

describe("i18n core functions", () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    vi.resetModules();
  });

  describe("useI18nStore", () => {
    it("defaults to zh when browser language starts with zh", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      expect(useI18nStore.getState().language).toBe("zh");
    });

    it("defaults to en when browser language is English", async () => {
      Object.defineProperty(globalThis, "navigator", {
        value: { language: "en-US" },
        writable: true,
      });
      const { useI18nStore } = await import("../hooks/useI18n");
      expect(useI18nStore.getState().language).toBe("en");
      // Restore
      Object.defineProperty(globalThis, "navigator", {
        value: { language: "zh-CN" },
        writable: true,
      });
    });

    it("setLanguage updates the language", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      useI18nStore.getState().setLanguage("en");
      expect(useI18nStore.getState().language).toBe("en");
    });

    it("persists language to localStorage", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      useI18nStore.getState().setLanguage("en");
      expect(localStorageMock.setItem).toHaveBeenCalledWith("downloader-language", "en");
    });

    it("returns zh locale by default", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      const locale = useI18nStore.getState().locale;
      expect(locale.common.confirm).toBe("确定");
    });

    it("returns en locale after switching", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      useI18nStore.getState().setLanguage("en");
      const locale = useI18nStore.getState().locale;
      expect(locale.common.confirm).toBe("OK");
    });
  });

  describe("t function (via store locale)", () => {
    // Helper: create t function from locale object
    function makeT(locale: Record<string, unknown>) {
      return (key: string, params?: Record<string, string | number>): string => {
        const value = key.split(".").reduce((obj: unknown, k: string) =>
          obj && typeof obj === "object" && k in obj ? (obj as Record<string, unknown>)[k] : undefined
        , locale);
        if (value === undefined || typeof value !== "string") return key;
        if (!params) return value;
        return Object.entries(params).reduce(
          (str, [k, v]) => str.replace(new RegExp(`\\{${k}\\}`, "g"), String(v)),
          value
        );
      };
    }

    it("returns translation for simple key", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      const t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      expect(t("common.confirm")).toBe("确定");
    });

    it("returns translation for nested key", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      const t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      expect(t("taskState.downloading")).toBe("下载中");
    });

    it("returns key itself when key does not exist", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      const t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      expect(t("nonexistent.key")).toBe("nonexistent.key");
    });

    it("supports template parameter substitution", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      const t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      const result = t("taskList.selectedCount", { count: 5 });
      expect(result).toContain("5");
      expect(result).not.toContain("{count}");
    });

    it("returns correct translations after language switch", async () => {
      const { useI18nStore } = await import("../hooks/useI18n");
      let t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      expect(t("common.confirm")).toBe("确定");
      useI18nStore.getState().setLanguage("en");
      t = makeT(useI18nStore.getState().locale as unknown as Record<string, unknown>);
      expect(t("common.confirm")).toBe("OK");
    });
  });

  describe("LANGUAGE_NAMES", () => {
    it("contains zh and en labels", async () => {
      const { LANGUAGE_NAMES } = await import("../lib/i18n");
      expect(LANGUAGE_NAMES.zh).toBe("中文");
      expect(LANGUAGE_NAMES.en).toBe("English");
    });
  });

  describe("STORAGE_KEY", () => {
    it("is downloader-language", async () => {
      const { STORAGE_KEY } = await import("../lib/i18n");
      expect(STORAGE_KEY).toBe("downloader-language");
    });
  });
});
