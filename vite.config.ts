/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// Vite 配置：React + Tauri 开发环境
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Tauri 开发服务器端口
  server: {
    port: 5173,
    strictPort: true,
  },
  // Tauri 需要相对路径
  base: "./",
  build: {
    // Tauri 打包目标（WebView2 支持 ES2022+）
    target: ["es2022", "chrome105", "safari15"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      output: {
        // 分包策略：vendor 库独立 chunk，减少首屏加载
        manualChunks: {
          "vendor-react": ["react", "react-dom"],
          "vendor-charts": ["recharts"],
        },
      },
    },
  },
  // Vitest 配置
  test: {
    globals: true,
    environment: "node",
  },
});
