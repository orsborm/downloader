// 国际化模块
// 共享类型和常量（实际 i18n 逻辑在 hooks/useI18n.ts 的 Zustand store 中）

/** 支持的语言 */
export type Language = "zh" | "en";

/** 语言显示名称 */
export const LANGUAGE_NAMES: Record<Language, string> = {
  zh: "中文",
  en: "English",
};

/** localStorage 键名 */
export const STORAGE_KEY = "downloader-language";
