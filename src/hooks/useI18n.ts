// 国际化 React Hook
// 集成 Zustand 状态管理，语言切换时自动重新渲染组件

import { create } from "zustand";
import { zh, type Locale } from "../lib/locales/zh";
import { en } from "../lib/locales/en";
import type { Language } from "../lib/i18n";
import { LANGUAGE_NAMES } from "../lib/i18n";

/** localStorage 键名 */
const STORAGE_KEY_NAME = "downloader-language";

/** 语言包映射 */
const LOCALES: Record<Language, Locale> = { zh, en };

/** 从 localStorage 加载语言偏好 */
function loadLanguage(): Language {
  try {
    const stored = localStorage.getItem(STORAGE_KEY_NAME);
    if (stored === "zh" || stored === "en") return stored;
  } catch {
    // localStorage 不可用时忽略
  }
  // 默认跟随浏览器语言
  const browserLang = navigator.language.toLowerCase();
  return browserLang.startsWith("zh") ? "zh" : "en";
}

/** i18n Store 状态 */
interface I18nState {
  language: Language;
  locale: Locale;
  setLanguage: (lang: Language) => void;
}

/** i18n Store */
export const useI18nStore = create<I18nState>((set) => {
  const lang = loadLanguage();
  return {
    language: lang,
    locale: LOCALES[lang],
    setLanguage: (lang: Language) => {
      try {
        localStorage.setItem(STORAGE_KEY_NAME, lang);
      } catch {
        // localStorage 不可用时忽略
      }
      set({ language: lang, locale: LOCALES[lang] });
    },
  };
});

/**
 * 获取嵌套对象的值
 */
function getNestedValue(obj: unknown, path: string): unknown {
  return path.split(".").reduce((current: unknown, key: string) => {
    if (current && typeof current === "object" && key in current) {
      return (current as Record<string, unknown>)[key];
    }
    return undefined;
  }, obj);
}

/**
 * React Hook：获取翻译函数
 * 语言切换时自动重新渲染组件
 */
export function useI18n() {
  const { language, locale, setLanguage } = useI18nStore();

  /**
   * 获取翻译文本
   * 支持简单的模板替换：t("key", { name: "test" }) → "xxx test xxx"
   */
  const t = (key: string, params?: Record<string, string | number>): string => {
    const value = getNestedValue(locale, key);
    if (value === undefined) return key;
    if (typeof value !== "string") return key;
    if (!params) return value;
    return Object.entries(params).reduce(
      (str, [k, v]) => str.replace(new RegExp(`\\{${k}\\}`, "g"), String(v)),
      value
    );
  };

  return { t, language, setLanguage, locale };
}

/**
 * 独立翻译函数（可在 React 组件外使用，如 hooks、工具函数）
 * 直接从 Zustand store 读取当前语言
 */
export function t(key: string, params?: Record<string, string | number>): string {
  const { locale } = useI18nStore.getState();
  const value = getNestedValue(locale, key);
  if (value === undefined) return key;
  if (typeof value !== "string") return key;
  if (!params) return value;
  return Object.entries(params).reduce(
    (str, [k, v]) => str.replace(new RegExp(`\\{${k}\\}`, "g"), String(v)),
    value
  );
}

// Re-export types
export type { Language };
export { LANGUAGE_NAMES };
