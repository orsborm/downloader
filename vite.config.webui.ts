/// <reference types="vitest" />
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";

// Vite 配置：WebUI 独立构建（用于嵌入 Rust 二进制或独立部署）
export default defineConfig({
  plugins: [react()],
  root: path.resolve(__dirname, "src-webui"),
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "src"),
      "@shared": path.resolve(__dirname, "src/lib"),
    },
  },
  server: {
    port: 6801,
    strictPort: true,
  },
  base: "./",
  build: {
    outDir: path.resolve(__dirname, "dist-webui"),
    target: ["es2021", "chrome100", "safari13"],
    minify: "esbuild",
    sourcemap: false,
  },
});
