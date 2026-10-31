import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import "./adjacency-table.ts";
import type { AdjacencyTable } from "./adjacency-table.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { TableView } from "../bindings/TableView.ts";
import type { TableViewRequest } from "../bindings/TableViewRequest.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { ToolError } from "../bindings/ToolError.ts";

// A four-tile table: ─ and ┐ (corridor), ═ (room), ░ (hedge). Only the
// entries the tests read have real numbers; the rest are allowed with a
// small share, except ─ then ═, which is a 0.

const TILES: TileView[] = [
  { id: "corridor_straight", name: "Corridor straight", kind: "corridor", tags: [], orientation: 1, sides: ["hedge", "passage", "hedge", "passage"], weight: 2 },
  { id: "corridor_corner", name: "Corridor corner", kind: "corridor", tags: [], orientation: 2, sides: ["hedge", "hedge", "passage", "passage"], weight: 3 },
  { id: "room_wall", name: "Room wall", kind: "room", tags: ["room_edge"], orientation: 0, sides: ["hedge", "floor_wall_n", "floor", "floor_wall_n"], weight: 1 },
  { id: "hedge", name: "Hedge", kind: "hedge", tags: [], orientation: 0, sides: ["hedge", "hedge", "hedge", "hedge"], weight: 4 },
];

function entry(direction: Direction, row: number, column: number, overrides: Partial<EntryView> = {}): EntryView {
  return {
    direction,
    row,
    column,
    facing: { row: "passage", column: "passage" },
    allowed: true,
    base: 1,
    rowWeight: TILES[row].weight,
    columnWeight: TILES[column].weight,
    exceptions: [],
    noise: 1,
    weight: 1,
    rowTotal: 40,
    probability: 0.0004,
    mayMeet: ["passage", "door"],
    ...overrides,
  };
}

function viewFor(direction: Direction, revision = 1): TableView {
  const entries: EntryView[] = [];
  for (let r = 0; r < TILES.length; r++) {
    for (let c = 0; c < TILES.length; c++) {
      if (r === 0 && c === 1) {
        // ─ then ┐: the demo's first task.
        entries.push(
          entry(direction, r, c, {
            base: 1,
            rowWeight: 2,
            columnWeight: 3,
            exceptions: [0],
            noise: 1.07,
            weight: 6,
            rowTotal: 40,
            probability: 0.15,
          }),
        );
      } else if (r === 0 && c === 2) {
        // ─ then ═: the demo's second task, a 0.
        entries.push(
          entry(direction, r, c, {
            facing: { row: "passage", column: "floor_wall_n" },
            allowed: false,
            weight: 0,
            probability: 0,
            noise: 1,
          }),
        );
      } else {
        entries.push(entry(direction, r, c));
      }
    }
  }
  return {
    revision,
    tileSetPath: "data/tile-sets/core.toml",
    loadedAt: "2026-10-31T19:14:03Z",
    tileSetName: "core",
    noise: { seed: 1234, amount: 0.15 },
    tiles: TILES,
    totalRows: 4,
    totalColumns: 4,
    rows: [0, 1, 2, 3],
    columns: [0, 1, 2, 3],
    directions: [direction],
    exceptions: [
      { name: "Straight likes a corner", multiplier: 1, rule: "tile corridor_straight (passage) beside tile corridor_corner (passage)" },
    ],
    entries,
  };
}

interface Backend {
  requests: TableViewRequest[];
  reloadFails?: ToolError;
}

function backend(): Backend {
  const state: Backend = { requests: [] };
  mockIPC((cmd, payload) => {
    if (cmd === COMMANDS.tableView) {
      const request = (payload as { request: TableViewRequest }).request;
      state.requests.push(request);
      return viewFor(request.directions[0]);
    }
    if (cmd === COMMANDS.reloadTileSet) {
      if (state.reloadFails) throw state.reloadFails;
      return { revision: 2, tileSetPath: "data/tile-sets/core.toml", loadedAt: "2026-10-31T20:20:00+01:00" };
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return state;
}

async function mount(): Promise<AdjacencyTable> {
  const el = document.createElement("adjacency-table");
  document.body.append(el);
  await vi.waitFor(() => expect(root(el).querySelector("tbody")).not.toBeNull());
  return el;
}

const root = (el: Element) => el.shadowRoot!;
const cell = (el: Element, row: number, column: number) =>
  root(el).querySelector<HTMLElement>(`td[data-row="${row}"][data-column="${column}"]`)!;
const panelText = (el: Element) => root(el).querySelector("table-explanation")!.shadowRoot!.textContent ?? "";
const squash = (s: string) => s.replace(/\s+/g, " ");

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("adjacency-table", () => {
  test("asks for the whole table to the east on open", async () => {
    const ipc = backend();
    await mount();
    expect(ipc.requests).toHaveLength(1);
    expect(ipc.requests[0].directions).toEqual(["east"]);
    expect(ipc.requests[0].rowFilter).toEqual({ kinds: [], tileIds: [], tags: [], sides: [] });
  });

  test("draws a row and a column per tile, grouped by kind, with glyph headers", async () => {
    backend();
    const el = await mount();
    const text = squash(root(el).textContent ?? "");

    const rowGlyphs = [...root(el).querySelectorAll('tbody th[scope="row"] .glyph')].map((g) => g.textContent);
    const columnGlyphs = [...root(el).querySelectorAll('thead th[scope="col"] .glyph')].map((g) => g.textContent);
    expect(rowGlyphs).toEqual(["─", "┐", "═", "░"]);
    expect(columnGlyphs).toEqual(["─", "┐", "═", "░"]);
    expect(root(el).querySelectorAll("tbody td")).toHaveLength(16);

    expect(text).toContain("Row tile, then its neighbour to the EAST");
    const kinds = [...root(el).querySelectorAll('th[scope="colgroup"]')].map((t) => t.textContent?.trim());
    expect(kinds).toEqual(["Corridor", "Room", "Hedge"]);

    const header = root(el).querySelector('thead th[scope="col"]')!;
    expect(header.getAttribute("title")).toBe("Corridor straight, E–W · weight 2.0");
    expect(text).toContain("data/tile-sets/core.toml");
  });

  test("shows the share of row with one decimal, <0.1 for tiny shares, blank for 0", async () => {
    backend();
    const el = await mount();
    expect(cell(el, 0, 1).textContent?.trim()).toBe("15.0");
    expect(cell(el, 0, 1).classList).toContain("step-3");
    expect(cell(el, 0, 1).classList).toContain("exception");
    expect(cell(el, 1, 1).textContent?.trim()).toBe("<0.1");
    expect(cell(el, 0, 2).textContent?.trim()).toBe("");
    expect(cell(el, 0, 2).classList).toContain("step-0");
  });

  test("switching direction sends a new request and relabels the grid", async () => {
    const ipc = backend();
    const el = await mount();
    root(el).querySelector<HTMLButtonElement>('button[data-direction="north"]')!.click();

    await vi.waitFor(() => expect(squash(root(el).textContent ?? "")).toContain("neighbour to the NORTH"));
    expect(ipc.requests.map((r) => r.directions)).toEqual([["east"], ["north"]]);
  });

  test("the S key switches direction", async () => {
    const ipc = backend();
    const el = await mount();
    el.dispatchEvent(new KeyboardEvent("keydown", { key: "s", bubbles: true }));
    await vi.waitFor(() => expect(ipc.requests.map((r) => r.directions)).toEqual([["east"], ["south"]]));
  });

  test("selecting a cell explains where its number comes from", async () => {
    backend();
    const el = await mount();
    expect(panelText(el)).toContain("Select a cell to see where its number comes from.");

    cell(el, 0, 1).click();
    await vi.waitFor(() => expect(cell(el, 0, 1).classList).toContain("selected"));
    const text = squash(panelText(el));
    expect(text).toContain("15.0% of ─'s east neighbours");
    expect(text).toContain("east side: Passage");
    expect(text).toContain("west side: Passage");
    expect(text).toContain("Passage–Passage base 1.0");
    expect(text).toContain("× Corridor straight 2.0");
    expect(text).toContain("× Corridor corner 3.0");
    expect(text).toContain("× Straight likes a corner 1.0");
    expect(text).toContain("tile corridor_straight (passage) beside tile corridor_corner (passage)");
    expect(text).toContain("× noise 1.07");
    expect(text).toContain("= entry 6.0");
    expect(text).toContain("÷ row total 40.0 = 15.0%");
  });

  test("selecting a 0 says it can't happen and names the clash", async () => {
    backend();
    const el = await mount();
    cell(el, 0, 2).click();
    await vi.waitFor(() => expect(panelText(el)).toContain("Can't happen"));
    expect(squash(panelText(el))).toContain(
      "Passage meets Floor, wall at N end. A passage only meets a passage or a door.",
    );
  });

  test("the selection stays on the same tiles across a direction switch", async () => {
    backend();
    const el = await mount();
    cell(el, 0, 1).click();
    root(el).querySelector<HTMLButtonElement>('button[data-direction="west"]')!.click();
    await vi.waitFor(() => expect(squash(panelText(el))).toContain("of ─'s west neighbours"));
    expect(cell(el, 0, 1).classList).toContain("selected");
  });

  test("arrow keys move the selection once the grid has focus", async () => {
    backend();
    const el = await mount();
    cell(el, 0, 0).click();
    const grid = root(el).querySelector("table")!;
    grid.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true }));
    await vi.waitFor(() => expect(cell(el, 0, 1).classList).toContain("selected"));
  });

  test("a failed reload shows Rust's error and keeps the last good table", async () => {
    const ipc = backend();
    const el = await mount();
    cell(el, 0, 1).click();
    ipc.reloadFails = {
      kind: "load_failed",
      messages: ["tile room_wall: weight must be above 0", "exception 2: unknown tag doorway"],
    };

    root(el).querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() =>
      expect(root(el).querySelector(".banner")?.textContent).toBe(
        "tile room_wall: weight must be above 0\nexception 2: unknown tag doorway",
      ),
    );
    expect(cell(el, 0, 1).textContent?.trim()).toBe("15.0");
    expect(squash(panelText(el))).toContain("× noise 1.07");

    ipc.reloadFails = undefined;
    root(el).querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() => expect(root(el).querySelector(".banner")).toBeNull());
    expect(ipc.requests).toHaveLength(2);
    expect(cell(el, 0, 1).classList).toContain("selected");
  });

  test("a table that can't load at all shows the error and no grid", async () => {
    mockIPC(() => {
      const error: ToolError = { kind: "not_loaded", messages: ["data/tile-sets/core.toml: missing [bases]"] };
      throw error;
    });
    const el = document.createElement("adjacency-table");
    document.body.append(el);
    await vi.waitFor(() =>
      expect(root(el).querySelector(".banner")?.textContent).toBe("data/tile-sets/core.toml: missing [bases]"),
    );
    expect(root(el).querySelector("table")).toBeNull();
  });
});
