// The save flow end to end through the app (DR-0037 points 1, 3, 6, 7, 16),
// over the fixture backend: a number in the table's panel, saved, refetched.

import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";
import "../maze-tool-app.ts";
import type { MazeToolApp } from "../maze-tool-app.ts";
import { fixtureBackend, type FixtureError } from "../explore/fixture.ts";
import type { NumberEditor } from "./number-editor.ts";

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

const squash = (s: string | null | undefined) => (s ?? "").replace(/\s+/g, " ").trim();

async function mountOnPair() {
  const ipc = fixtureBackend();
  const el = document.createElement("maze-tool-app");
  document.body.append(el);
  await el.updateComplete;
  const table = el.shadowRoot!.querySelector("adjacency-table")!;
  // Row ─, column ╢, east: Passage meets Door.
  const td = () => table.shadowRoot!.querySelector<HTMLElement>('td[data-row="0"][data-column="6"]');
  await vi.waitFor(() => expect(td()).not.toBeNull());
  td()!.click();
  const panel = () => table.shadowRoot!.querySelector("table-explanation")!;
  await vi.waitFor(() => expect(panel().shadowRoot!.querySelector("button.number")).not.toBeNull());
  return { ipc, el, panel };
}

function editorIn(panel: Element): NumberEditor | null {
  return panel.shadowRoot!.querySelector("number-editor");
}

async function typeAndSave(editor: NumberEditor, text: string) {
  await editor.updateComplete;
  const input = editor.shadowRoot!.querySelector("input")!;
  input.value = text;
  input.dispatchEvent(new Event("input"));
  await editor.updateComplete;
  input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
}

const tab = (el: MazeToolApp, id: string) => el.shadowRoot!.querySelector<HTMLButtonElement>(`#tab-${id}`)!;

describe("saving a number", () => {
  test("the panel's stored numbers are edit buttons; worked-out ones stay text (point 1)", async () => {
    const { panel } = await mountOnPair();
    const buttons = [...panel().shadowRoot!.querySelectorAll("button.number")];
    expect(buttons.map((b) => b.getAttribute("aria-label"))).toEqual([
      "Edit Passage–Door base, 1.0",
      "Edit Corridor straight weight, 1.0",
      "Edit Room doorway weight, 0.4",
    ]);
    expect(squash(panel().shadowRoot!.querySelector(".noise")!.textContent)).toBe("× noise 1.0");
  });

  test("a base edit names the [bases] key Rust sent", async () => {
    const { ipc, panel } = await mountOnPair();
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="base"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    expect(squash(editorIn(panel())!.shadowRoot!.querySelector(".reach")!.textContent)).toBe("Every Passage–Door meeting.");
    await typeAndSave(editorIn(panel())!, "0.5");
    await vi.waitFor(() => expect(ipc.calls.some((c) => c.cmd === "edit_numbers")).toBe(true));
    expect(ipc.calls.find((c) => c.cmd === "edit_numbers")!.request).toEqual({
      revision: 1,
      edits: [{ target: { kind: "base", pair: "passage_door" }, value: 0.5 }],
    });
  });

  test("a save writes through Rust, refetches, and shows the saved line under the sum (points 6 and 7)", async () => {
    const { ipc, el, panel } = await mountOnPair();
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="row"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    expect(squash(editorIn(panel())!.shadowRoot!.querySelector(".reach")!.textContent)).toBe(
      "Every Corridor straight, in its one orientation and every direction.",
    );
    const tableCallsBefore = ipc.calls.filter((c) => c.cmd === "table_view").length;
    await typeAndSave(editorIn(panel())!, "2");

    await vi.waitFor(() => expect(panel().shadowRoot!.querySelector(".saved")).not.toBeNull());
    expect(ipc.calls.find((c) => c.cmd === "edit_numbers")!.request).toEqual({
      revision: 1,
      edits: [{ target: { kind: "tileWeight", tileId: "corridor_straight" }, value: 2 }],
    });
    expect(editorIn(panel())).toBeNull();
    expect(squash(panel().shadowRoot!.querySelector(".saved")!.textContent)).toMatch(
      /^Saved \d\d:\d\d · Corridor straight weight 1\.0 → 2\.0$/,
    );
    // No maze is shown, so there is nothing to compare.
    expect(panel().shadowRoot!.querySelector("#see-compare")).toBeNull();
    // The same path as Reload: the table refetched and shows the new weight, still on the same pair.
    expect(ipc.calls.filter((c) => c.cmd === "table_view").length).toBeGreaterThan(tableCallsBefore);
    await vi.waitFor(() =>
      expect(panel().shadowRoot!.querySelector('[data-edit="row"]')!.getAttribute("aria-label")).toBe(
        "Edit Corridor straight weight, 2.0",
      ),
    );
    expect(tab(el, "table").getAttribute("aria-selected")).toBe("true");
    // The next save names the new revision.
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="row"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    await typeAndSave(editorIn(panel())!, "1");
    await vi.waitFor(() => expect(ipc.calls.filter((c) => c.cmd === "edit_numbers").length).toBe(2));
    expect((ipc.calls.filter((c) => c.cmd === "edit_numbers")[1].request as { revision: number }).revision).toBe(2);
  });

  test("with a maze shown, [See before and after] opens the Maze tab", async () => {
    const { el, panel } = await mountOnPair();
    tab(el, "maze").click();
    const maze = el.shadowRoot!.querySelector("maze-view")!;
    await maze.updateComplete;
    maze.shadowRoot!.querySelector<HTMLButtonElement>("#generate")!.click();
    await vi.waitFor(() => expect(maze.hasMaze).toBe(true));
    tab(el, "table").click();

    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="row"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    await typeAndSave(editorIn(panel())!, "2");
    await vi.waitFor(() => expect(panel().shadowRoot!.querySelector("#see-compare")).not.toBeNull());
    // No tab switches by itself (point 7).
    expect(tab(el, "table").getAttribute("aria-selected")).toBe("true");
    panel().shadowRoot!.querySelector<HTMLButtonElement>("#see-compare")!.click();
    await vi.waitFor(() => expect(tab(el, "maze").getAttribute("aria-selected")).toBe("true"));
  });

  test("a refused edit stays at the field; nothing reloads and the banner stays clear (point 16)", async () => {
    const { ipc, el, panel } = await mountOnPair();
    ipc.edit = () => {
      const error: FixtureError = { kind: "refused", messages: ["corridor_straight's weight is -1; weights must be 0 or more."] };
      throw error;
    };
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="row"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    const tableCallsBefore = ipc.calls.filter((c) => c.cmd === "table_view").length;
    await typeAndSave(editorIn(panel())!, "-1");
    await vi.waitFor(() => expect(editorIn(panel())!.shadowRoot!.querySelector(".refused")).not.toBeNull());
    expect(editorIn(panel())!.shadowRoot!.querySelector(".refused")!.textContent).toBe(
      "Not saved: corridor_straight's weight is -1; weights must be 0 or more. core.toml is unchanged.",
    );
    expect(el.shadowRoot!.querySelector(".banner")).toBeNull();
    expect(ipc.calls.filter((c) => c.cmd === "table_view").length).toBe(tableCallsBefore);
    expect(panel().shadowRoot!.querySelector(".saved")).toBeNull();
  });

  test("one editor at a time: opening another closes the first", async () => {
    const { panel } = await mountOnPair();
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="row"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())).not.toBeNull());
    panel().shadowRoot!.querySelector<HTMLButtonElement>('[data-edit="column"]')!.click();
    await vi.waitFor(() => expect(editorIn(panel())!.label).toBe("Room doorway weight"));
    expect(panel().shadowRoot!.querySelectorAll("number-editor").length).toBe(1);
  });
});
