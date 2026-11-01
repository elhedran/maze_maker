/// <reference types="vitest/config" />
import { defineConfig } from "vite";

// Tauri expects a fixed port (build.devUrl in src-tauri/tauri.conf.json).
// The game is on 1430 so it and the tool (1420) can run at once (DR-0043 point 1).
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1430,
    // HMR on the next port, clear of the tool's 1420 (DR-0043 point 1).
    hmr: { port: 1431 },
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
