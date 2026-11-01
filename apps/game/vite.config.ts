/// <reference types="vitest/config" />
import { defineConfig } from "vite";

// Tauri expects a fixed port (build.devUrl in src-tauri/tauri.conf.json).
// The tool uses 1420; the game takes the next one so both can run at once.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1421,
    strictPort: true,
    watch: {
      // Rust changes are handled by `tauri dev`, not Vite.
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    // Lit components need a DOM with custom elements and shadow roots (DR-0020).
    environment: "happy-dom",
    // Per-test cleanup in one place, so a new test file can't forget it.
    setupFiles: ["src/test-setup.ts"],
  },
});
