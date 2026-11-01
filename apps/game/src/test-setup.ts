// Runs after each test in every file (vite.config.ts `setupFiles`, DR-0020):
// happy-dom keeps one document per file, and mockIPC handlers outlive a test.
import { afterEach } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});
