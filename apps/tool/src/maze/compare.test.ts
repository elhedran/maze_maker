// The before-and-after compare and the opening-pressure line (DR-0037
// points 8 to 14), against a mock backend.

import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import "./maze-view.ts";
import { afterLabel, beforeLabel, changedText, measureRows } from "./compare-text.ts";
import { SAVE_NUMBER_EVENT, type SaveRequest } from "../edit/save-number.ts";
import type { GenerateRequest } from "../bindings/GenerateRequest.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { CompareResult, EditRecord, TableView } from "../pending-bindings.ts";

const tile = (id: string, name: string, kind: TileView["kind"], orientation: number): TileView => ({
  id,
  name,
  kind,
  tags: [],
  orientation,
  sides: ["hedge", "hedge", "hedge", "hedge"],
  weight: 1,
});
// 0 ─, 1 ┐, 2 ░
const TILES = [
  tile("corridor_straight", "Corridor straight", "corridor", 1),
  tile("corridor_corner", "Corridor corner", "corridor", 2),
  tile("hedge", "Hedge", "hedge", 0),
];

const measures = (reach: string, areas: string) => [
  { name: "Reachable from centre", value: reach, explanation: "…" },
  { name: "Separate areas", value: areas, explanation: "…" },
];

// 2×2 mazes; the after one differs in cell 1 (row 0, col 1).
function maze(revision: number, seed: number, cells: number[], m = measures("3 of 4 (75%)", "2")): GeneratedMaze {
  return { revision, seed, width: 2, height: 2, cells, centre: { x: 0, y: 0 }, restarts: 0, failed: false, measures: m };
}

const EDIT: EditRecord = {
  at: "2026-11-01T20:31:00Z",
  target: { kind: "tileWeight", tileId: "corridor_corner" },
  number: "tile.corridor_corner.weight",
  from: 3,
  to: 2,
};

interface Backend {
  revision: number;
  openingPressure: number;
  compare?: (r: GenerateRequest) => CompareResult;
}

function backend(): Backend {
  const state: Backend = { revision: 1, openingPressure: 1 };
  mockIPC((cmd, payload) => {
    const request = (payload as { request: GenerateRequest }).request;
    switch (cmd) {
      case COMMANDS.tableView: {
        const view: TableView = {
          revision: state.revision,
          tileSetPath: "data/tile-sets/core.toml",
          loadedAt: "2026-11-01T20:31:05Z",
          tileSetName: "core",
          noise: null,
          tiles: TILES,
          totalRows: 3,
          totalColumns: 3,
          rows: [],
          columns: [],
          directions: ["east"],
          exceptions: [],
          entries: [],
          openingPressure: state.openingPressure,
        };
        return view;
      }
      case COMMANDS.generateMaze:
        return maze(state.revision, request.seed, [0, 0, 2, 2]);
      case COMMANDS.compareMazes:
        return state.compare!(request);
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return state;
}

/** A shown maze, then a change to revision 2 that alters one cell. */
async function compared(edit: Partial<CompareResult> = {}) {
  const ipc = backend();
  const el = document.createElement("maze-view");
  document.body.append(el);
  await el.updateComplete;
  const seed = $<HTMLInputElement>(el, "#seed");
  seed.value = "48213";
  seed.dispatchEvent(new Event("input"));
  await el.updateComplete;
  $(el, "#generate").click();
  await vi.waitFor(() => expect(el.hasMaze).toBe(true));
  ipc.revision = 2;
  ipc.compare = (r) => ({
    before: { maze: maze(1, r.seed, [0, 0, 2, 2]), tiles: TILES, loadedAt: "2026-11-01T20:14:03Z" },
    after: maze(2, r.seed, [0, 1, 2, 2], measures("2 of 4 (50%)", "3")),
    changed: [1],
    edits: [EDIT],
    ...edit,
  });
  el.showChange();
  await vi.waitFor(() => expect(root(el).querySelector(".sides")).not.toBeNull());
  return { ipc, el };
}

const root = (el: Element) => el.shadowRoot!;
const $ = <T extends HTMLElement>(el: Element, sel: string) => root(el).querySelector<T>(sel)!;
const squash = (s: string | null | undefined) => (s ?? "").replace(/\s+/g, " ").trim();
const cell = (el: Element, side: "before" | "after", x: number, y: number) =>
  $(el, `.side.${side} .cell[data-x="${x}"][data-y="${y}"]`);

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("compare text", () => {
  test("labels say which side is which in words (point 9)", () => {
    expect(beforeLabel("not a time")).toBe("BEFORE · as loaded not a time");
    expect(afterLabel([EDIT], { tiles: TILES, exceptions: [] }, "x")).toBe("AFTER · Corridor corner weight 3.0 → 2.0");
    expect(afterLabel([], { tiles: TILES, exceptions: [] }, "x")).toBe("AFTER · after reload at x");
  });

  test("the count, and nothing changed", () => {
    expect(changedText(37, 384)).toBe("37 of 384 cells changed.");
    expect(changedText(0, 384)).toBe("Nothing changed for this seed.");
  });

  test("measures pair by name; a side that gave up reads —", () => {
    const a = maze(1, 1, [0], measures("1", "2"));
    const b = maze(2, 1, [0], measures("3", "4"));
    expect(measureRows(a, b)).toEqual([
      { name: "Reachable from centre", before: "1", after: "3" },
      { name: "Separate areas", before: "2", after: "4" },
    ]);
    expect(measureRows({ ...a, failed: true }, b)[0]).toEqual({ name: "Reachable from centre", before: "—", after: "3" });
  });
});

describe("maze-view compare (DR-0037 points 8 to 13)", () => {
  test("after a change, the same seed shows before and after side by side, labelled", async () => {
    const { el } = await compared();
    expect($(el, "#status").textContent).toBe("Seed 48213 · 2×2 · before and after your last change");
    expect($(el, "#before-label").textContent).toMatch(/^BEFORE · as loaded /);
    expect($(el, "#after-label").textContent).toBe("AFTER · Corridor corner weight 3.0 → 2.0");
    expect(cell(el, "before", 1, 0).textContent).toBe("──");
    expect(cell(el, "after", 1, 0).textContent!.startsWith("┐")).toBe(true);
    expect($(el, "#changed-count").textContent).toBe("1 of 4 cells changed.");
    expect(squash($(el, ".spread").textContent)).toContain("changes spread from the first");
    // Centre outline on both (point 12).
    expect(cell(el, "before", 0, 0).classList).toContain("centre");
    expect(cell(el, "after", 0, 0).classList).toContain("centre");
  });

  test("unchanged cells fade in both mazes; unticking shows both in full (point 11)", async () => {
    const { el } = await compared();
    expect(cell(el, "before", 0, 0).classList).toContain("same");
    expect(cell(el, "after", 0, 0).classList).toContain("same");
    expect(cell(el, "before", 1, 0).classList).not.toContain("same");
    expect(cell(el, "after", 1, 0).classList).not.toContain("same");
    const fade = $<HTMLInputElement>(el, "#fade");
    expect(fade.checked).toBe(true);
    fade.checked = false;
    fade.dispatchEvent(new Event("change"));
    await el.updateComplete;
    expect(root(el).querySelectorAll(".cell.same")).toHaveLength(0);
  });

  test("nothing changed: says so and fades nothing", async () => {
    const { el } = await compared({ changed: [] });
    expect($(el, "#changed-count").textContent).toBe("Nothing changed for this seed.");
    expect(root(el).querySelector("#fade")).toBeNull();
    expect(root(el).querySelectorAll(".cell.same")).toHaveLength(0);
  });

  test("the measures sit in a before → after table, as Rust worded them (point 10)", async () => {
    const { el } = await compared();
    const rows = [...root(el).querySelectorAll(".compare-measures tbody tr")].map((tr) => squash(tr.textContent));
    expect(rows).toEqual(["Reachable from centre 3 of 4 (75%) → 2 of 4 (50%)", "Separate areas 2 → 3"]);
  });

  test("only the after maze selects; hovering names both tiles (points 10 and 12)", async () => {
    const { el } = await compared();
    cell(el, "before", 1, 0).click();
    await el.updateComplete;
    expect(root(el).querySelector(".cell.selected")).toBeNull();
    cell(el, "after", 1, 0).click();
    await el.updateComplete;
    expect(cell(el, "after", 1, 0).classList).toContain("selected");
    expect($(el, "#selected").textContent).toBe("Selected (after): ┐ Corridor corner, open S and W · col 1, row 0");

    cell(el, "after", 1, 0).dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    await el.updateComplete;
    expect($(el, "#readout").textContent).toBe(
      "col 1, row 0 · before: ─ Corridor straight, E–W · after: ┐ Corridor corner, open S and W",
    );
    expect(cell(el, "before", 1, 0).classList).toContain("twin");

    cell(el, "before", 0, 1).dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    await el.updateComplete;
    expect($(el, "#readout").textContent).toContain("Before is the old tile set, so it can't be replayed.");
    expect(cell(el, "after", 0, 1).classList).toContain("twin");
  });

  test("a side that gave up shows the message and — in its measures (point 13)", async () => {
    const { el } = await compared({
      before: { maze: { ...maze(1, 48213, [0, null, null, null] as never), failed: true, restarts: 5 }, tiles: TILES, loadedAt: "x" },
    });
    expect($(el, ".side.before .failed").textContent).toContain("No maze for seed 48213");
    expect(squash(root(el).querySelector(".compare-measures tbody tr")!.textContent)).toBe("Reachable from centre — → 2 of 4 (50%)");
  });

  test("Close compare and Generate go back to the single maze", async () => {
    const { el } = await compared();
    $(el, "#close-compare").click();
    await el.updateComplete;
    expect(root(el).querySelector(".sides")).toBeNull();
    expect(squash($(el, "#status").textContent)).toBe("Seed 48213 · 2×2 · first try");

    el.showChange();
    await vi.waitFor(() => expect(root(el).querySelector(".sides")).not.toBeNull());
    $(el, "#generate").click();
    await vi.waitFor(() => expect(root(el).querySelector(".sides")).toBeNull());
  });

  test("with no previous load there's no before: the maze just regenerates", async () => {
    const ipc = backend();
    const el = document.createElement("maze-view");
    document.body.append(el);
    await el.updateComplete;
    $(el, "#generate").click();
    await vi.waitFor(() => expect(el.hasMaze).toBe(true));
    ipc.compare = (r) => ({ before: null, after: maze(1, r.seed, [0, 0, 2, 2]), changed: [], edits: [] });
    el.showChange();
    await vi.waitFor(() => expect($(el, "#status").textContent).toContain("regenerated after reload"));
    expect(root(el).querySelector(".sides")).toBeNull();
  });
});

describe("opening pressure (DR-0037 point 14)", () => {
  async function mounted(pressure: number) {
    const ipc = backend();
    ipc.openingPressure = pressure;
    const el = document.createElement("maze-view");
    document.body.append(el);
    await vi.waitFor(() => expect(root(el).querySelector("#pressure")).not.toBeNull());
    return { ipc, el };
  }

  test("sits under the seed row, reading (off) at 1, with its meaning", async () => {
    const { el } = await mounted(1);
    expect(squash($(el, ".pressure").textContent)).toBe(
      "Opening pressure 1.0 (off) Below 1, a tile that adds no new way onward is less likely.",
    );
    document.body.innerHTML = "";
    const { el: half } = await mounted(0.5);
    expect(squash($(half, "#pressure").textContent)).toBe("0.5");
  });

  test("is edited like the panel's numbers, with its range as a hint, and saves through the app", async () => {
    const { el } = await mounted(1);
    const requests: SaveRequest[] = [];
    el.addEventListener(SAVE_NUMBER_EVENT, (e) => {
      const detail = (e as CustomEvent<SaveRequest>).detail;
      requests.push(detail);
      detail.result = Promise.resolve({
        record: { ...EDIT, target: { kind: "openingPressure" }, from: null, to: 0.5 },
        warnings: [],
        canCompare: false,
      });
    });
    $(el, "#pressure").click();
    await el.updateComplete;
    const editor = root(el).querySelector("number-editor")!;
    await editor.updateComplete;
    expect(squash(editor.shadowRoot!.querySelector(".reach")!.textContent)).toBe(
      "Every draw where a tile would add no new way onward.",
    );
    expect(squash(editor.shadowRoot!.querySelector(".hint")!.textContent)).toBe("Between 0 and 1. 1 is off.");
    const input = editor.shadowRoot!.querySelector("input")!;
    expect(input.value).toBe("1.0");
    input.value = "0.5";
    input.dispatchEvent(new Event("input"));
    await editor.updateComplete;
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    await vi.waitFor(() => expect(root(el).querySelector("number-editor")).toBeNull());
    expect(requests.map((r) => r.edit)).toEqual([{ target: { kind: "openingPressure" }, value: 0.5 }]);
  });
});
