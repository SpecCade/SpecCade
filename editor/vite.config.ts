import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Tell vite to watch for changes in the src-tauri directory
      ignored: ["!**/src-tauri/**"],
    },
  },
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  // Tauri expects window.__TAURI__ to be available
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari14",
    // Don't minify for better debugging
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    // Produce sourcemaps for debugging
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  // Optimize Monaco editor bundling
  optimizeDeps: {
    include: ["monaco-editor"],
  },
});
