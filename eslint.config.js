// ESLint 平面配置（Flat Config）
// React + TypeScript + Vite 项目

import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  // 全局忽略
  {
    ignores: ["dist/", "dist-webui/", "node_modules/", "src-tauri/", "extension/", "*.config.js", "*.config.ts"],
  },
  // 基础推荐规则
  js.configs.recommended,
  ...tseslint.configs.recommended,
  // React Hooks 规则
  {
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      // React Hooks 规则
      "react-hooks/rules-of-hooks": "error",
      "react-hooks/exhaustive-deps": "warn",
      // React Refresh（HMR 安全）
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
      // TypeScript 放松规则
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_", varsIgnorePattern: "^_" }],
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-non-null-assertion": "warn",
      // 通用规则
      "no-console": ["warn", { allow: ["warn", "error"] }],
      "prefer-const": "error",
      "no-var": "error",
    },
  }
);
