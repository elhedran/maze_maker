import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import "./maze-tool-app.ts";

// mockIPC fakes the Tauri backend, so these tests go through the real
// `invoke` call and check the command name the component asks for.

function mount() {
  const el = document.createElement("maze-tool-app");
  document.body.append(el);
  return el;
}

function text(el: HTMLElement) {
  return el.shadowRoot?.textContent ?? "";
}

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("maze-tool-app", () => {
  test("shows the version that core_version returns", async () => {
    const calls: string[] = [];
    mockIPC((cmd) => {
      calls.push(cmd);
      if (cmd === "core_version") return "9.9.9";
      throw new Error(`unexpected command ${cmd}`);
    });

    const el = mount();

    await vi.waitFor(() =>
      expect(el.shadowRoot?.querySelector("#core-version")?.textContent).toBe("9.9.9"),
    );
    expect(calls).toEqual(["core_version"]);
  });

  test("shows an error when the command fails", async () => {
    mockIPC(() => {
      throw new Error("backend down");
    });

    const el = mount();

    await vi.waitFor(() => expect(text(el)).toContain("Could not reach maze-core"));
    expect(text(el)).toContain("backend down");
    expect(el.shadowRoot?.querySelector("#core-version")).toBeNull();
  });
});
