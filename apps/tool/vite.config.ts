/// <reference types="vitest/config" />
import { defineConfig } from "vite";

// Tauri expects a fixed port (build.devUrl in src-tauri/tauri.conf.json).
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Rust changes are handled by `tauri dev`, not Vite.
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    // Lit components need a DOM with custom elements and shadow roots.
    // happy-dom is lighter than jsdom and covers what Lit uses.
    environment: "happy-dom",
  },
});
