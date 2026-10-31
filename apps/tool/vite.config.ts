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
});
