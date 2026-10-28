import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
// @ts-expect-error 本地 Node 验收桥接由 Vite 加载。
import { nlpBridge } from "./scripts/nlp_bridge.mjs";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async ({ mode }) => ({
  plugins: [vue(), ...(mode === "frontend" ? [] : [nlpBridge()])],
  // 两个开发模式独立维护预构建结果，支持同时预览与验收。
  cacheDir: `node_modules/.vite/${mode}`,

  optimizeDeps: {
    entries: ["index.html"],
    include: ["@lucide/vue", "@tanstack/vue-virtual", "vue"]
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: [
        "**/src-tauri/**", "**/.agents/**", "**/target/**", "**/target-*/**",
        "**/experiments/**", "**/release-resources/**", "**/upload-resources/**",
        "**/unidic-cwj-*/**", "**/unidic-csj-*/**", "**/ipadic/**", "**/nlp_retraining/**",
        "**/packages/**", "**/vendor/**", "**/data/**",
      ],
    },
  },
}));
