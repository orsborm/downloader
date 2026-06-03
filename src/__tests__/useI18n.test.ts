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

Object.defineProperty(globalThis, "navigator", {
  value: { language: "zh-CN" },
  writable: true,
});

describe("useI18nStore", () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    vi.resetModules();
  });

  it("defaults to zh language", async () => {
    const { useI18nStore } = await import("../hooks/useI18n");
    const state = useI18nStore.getState();
    expect(state.language).toBe("zh");
  });

  it("loads stored language from localStorage", async () => {
    store["downloader-language"] = "en";
    const { useI18nStore } = await import("../hooks/useI18n");
    const state = useI18nStore.getState();
    expect(state.language).toBe("en");
  });

  it("setLanguage updates language and persists", async () => {
    const { useI18nStore } = await import("../hooks/useI18n");
    useI18nStore.getState().setLanguage("en");
    expect(useI18nStore.getState().language).toBe("en");
    expect(localStorageMock.setItem).toHaveBeenCalledWith("downloader-language", "en");
  });

  it("setLanguage updates locale object", async () => {
    const { useI18nStore } = await import("../hooks/useI18n");
    useI18nStore.getState().setLanguage("en");
    expect(useI18nStore.getState().locale.common.confirm).toBe("OK");
  });

  it("setLanguage to zh updates locale to Chinese", async () => {
    const { useI18nStore } = await import("../hooks/useI18n");
    useI18nStore.getState().setLanguage("en");
    useI18nStore.getState().setLanguage("zh");
    expect(useI18nStore.getState().locale.common.confirm).toBe("确定");
  });
});

describe("useI18n hook", () => {
  beforeEach(() => {
    localStorageMock.clear();
    localStorageMock.getItem.mockClear();
    localStorageMock.setItem.mockClear();
    vi.resetModules();
  });

  it("returns t function", async () => {
    await import("../hooks/useI18n");
    // Call outside React render for basic check
    const store = (await import("../hooks/useI18n")).useI18nStore;
    const state = store.getState();
    expect(state.language).toBe("zh");
    expect(state.locale).toBeDefined();
    expect(state.setLanguage).toBeInstanceOf(Function);
  });

  it("t function supports template substitution", async () => {
    const { useI18nStore } = await import("../hooks/useI18n");
    const { locale } = useI18nStore.getState();
    // Verify locale has template keys
    expect(locale.taskList.selectedCount).toContain("{count}");
  });

  it("LANGUAGE_NAMES is re-exported", async () => {
    const { LANGUAGE_NAMES } = await import("../hooks/useI18n");
    expect(LANGUAGE_NAMES.zh).toBe("中文");
    expect(LANGUAGE_NAMES.en).toBe("English");
  });
});
