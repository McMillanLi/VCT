import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],

  // Tauri 期望产物在 src-tauri 的同级，且使用固定端口
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 不监听 Rust 后端变更，避免触发不必要的重启
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // Tauri 在桌面端使用 WebKit，对 chunk 拆分无强需求；关闭以减少产物文件数
    chunkSizeWarningLimit: 2000,
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
  resolve: {
    alias: {
      "@": "/src",
    },
  },
});
