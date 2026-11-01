import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "./ipc.ts";
import "./maze-tool-app.ts";
import type { MazeToolApp } from "./maze-tool-app.ts";
import type { GenerateRequest } from "./bindings/GenerateRequest.ts";
import type { TableView } from "./bindings/TableView.ts";
import type { TileView } from "./bindings/TileView.ts";
import type { ToolError } from "./bindings/ToolError.ts";
import { fixtureBackend } from "./explore/fixture.ts";

// mockIPC fakes the Tauri backend, so these tests go through the real
// `invoke` call and check the command name the component asks for.

function mount(): MazeToolApp {
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
      // The table view has its own tests; here it only has to not crash the app.
      throw new Error(`not under test: ${cmd}`);
    });

    const el = mount();

    await vi.waitFor(() =>
      expect(el.shadowRoot?.querySelector("#core-version")?.textContent).toBe("9.9.9"),
    );
    expect(calls).toContain("core_version");
    expect(el.shadowRoot?.querySelector("adjacency-table")).not.toBeNull();
  });

  test("shows an error when the command fails", async () => {
    mockIPC(() => {
      throw new Error("backend down");
    });

    const el = mount();

    await vi.waitFor(() => expect(text(el)).toContain("Could not reach maze-core"));
    expect(text(el)).not.toContain("Error:");
    expect(text(el)).toContain("backend down");
    expect(el.shadowRoot?.querySelector("#core-version")).toBeNull();
  });

  test("the tile set bar sits above both tabs, and Table opens first", async () => {
    backend();
    const el = mount();
    await vi.waitFor(() => expect(text(el)).toContain("core.toml"));
    const path = el.shadowRoot!.querySelector<HTMLElement>(".path")!;
    expect(path.textContent).toBe("core.toml");
    expect(path.title).toBe("data/tile-sets/core.toml");
    const labels = [...el.shadowRoot!.querySelectorAll('[role="tab"]')].map((t) => t.textContent?.trim());
    expect(labels).toEqual(["Table", "Explore", "Maze", "World"]);
    expect(tab(el, "table").getAttribute("aria-selected")).toBe("true");
    expect(panel(el, "table").hidden).toBe(false);
    expect(panel(el, "explore").hidden).toBe(true);
    expect(panel(el, "maze").hidden).toBe(true);
    expect(el.shadowRoot!.querySelector("#reload")).not.toBeNull();
  });

  test("switching tabs hides a view without destroying it", async () => {
    backend();
    const el = mount();
    await el.updateComplete;
    const table = el.shadowRoot!.querySelector("adjacency-table");
    const maze = el.shadowRoot!.querySelector("maze-view");

    tab(el, "maze").click();
    await el.updateComplete;
    expect(tab(el, "maze").getAttribute("aria-selected")).toBe("true");
    expect(panel(el, "maze").hidden).toBe(false);
    expect(panel(el, "table").hidden).toBe(true);

    tab(el, "table").click();
    await el.updateComplete;
    expect(el.shadowRoot!.querySelector("adjacency-table")).toBe(table);
    expect(el.shadowRoot!.querySelector("maze-view")).toBe(maze);
  });

  test("a failed reload shows Rust's error and keeps the table and the maze", async () => {
    const ipc = backend();
    const el = mount();
    const maze = await generateOnce(el);
    ipc.reloadFails = {
      kind: "load_failed",
      messages: ["tile room_wall: weight must be above 0", "exception 2: unknown tag doorway"],
    };

    el.shadowRoot!.querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() =>
      expect(el.shadowRoot!.querySelector(".banner")?.textContent).toBe(
        "tile room_wall: weight must be above 0\nexception 2: unknown tag doorway",
      ),
    );
    expect(ipc.generated).toHaveLength(1);
    expect(maze.shadowRoot!.querySelector(".row")).not.toBeNull();
  });

  test("a good reload clears the banner, refetches the table and regenerates the same seed", async () => {
    const ipc = backend();
    const el = mount();
    const maze = await generateOnce(el);
    await vi.waitFor(() => expect(ipc.tableRequests).toBeGreaterThan(0));
    ipc.reloadFails = { kind: "load_failed", messages: ["bad"] };
    el.shadowRoot!.querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() => expect(el.shadowRoot!.querySelector(".banner")).not.toBeNull());
    const tableRequestsBefore = ipc.tableRequests;

    ipc.reloadFails = undefined;
    el.shadowRoot!.querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() => expect(el.shadowRoot!.querySelector(".banner")).toBeNull());
    await vi.waitFor(() => expect(ipc.generated.map((r) => r.seed)).toEqual([48213, 48213]));
    await vi.waitFor(() =>
      expect(maze.shadowRoot!.querySelector("#status")?.textContent).toContain("Seed 48213 · regenerated after reload"),
    );
    expect(ipc.tableRequests).toBeGreaterThan(tableRequestsBefore);
    expect(text(el)).toContain("loaded");
  });

  test("Explore this tile in the table opens the Explore tab on that pair", async () => {
    fixtureBackend();
    const el = mount();
    await el.updateComplete;
    const table = el.shadowRoot!.querySelector("adjacency-table")!;
    // Row ─, column ╢: the first step of DR-0026's demo walk.
    const td = () => table.shadowRoot!.querySelector<HTMLElement>('td[data-row="0"][data-column="6"]');
    await vi.waitFor(() => expect(td()).not.toBeNull());
    td()!.click();
    await vi.waitFor(() => expect(table.shadowRoot!.querySelector("#explore-this")).not.toBeNull());
    table.shadowRoot!.querySelector<HTMLButtonElement>("#explore-this")!.click();

    await vi.waitFor(() => expect(tab(el, "explore").getAttribute("aria-selected")).toBe("true"));
    expect(panel(el, "explore").hidden).toBe(false);
    const explore = el.shadowRoot!.querySelector("explore-view")!;
    await vi.waitFor(() => expect(explore.shadowRoot!.querySelector(".centre .glyph")?.textContent).toBe("─"));
    const selected = explore.shadowRoot!.querySelector('section.arm.east .row.selected .glyph');
    expect(selected?.textContent).toBe("╢");
  });

  test("a good reload refetches the explore view on its centre", async () => {
    const ipc = fixtureBackend();
    const el = mount();
    await el.updateComplete;
    const explore = el.shadowRoot!.querySelector("explore-view")!;
    explore.open({ centre: { id: "hedge", orientation: 0 } });
    await vi.waitFor(() => expect(explore.shadowRoot!.querySelector(".centre .glyph")?.textContent).toBe("░"));
    const asked = () => ipc.calls.filter((c) => c.cmd === COMMANDS.exploreTile).length;
    const before = asked();

    el.shadowRoot!.querySelector<HTMLButtonElement>("#reload")!.click();
    await vi.waitFor(() => expect(asked()).toBe(before + 1));
  });

  test("Why is this here? in the Maze tab opens Explore on the cell, and Back to maze returns to it", async () => {
    fixtureBackend();
    const el = mount();
    await el.updateComplete;
    tab(el, "maze").click();
    const maze = el.shadowRoot!.querySelector("maze-view")!;
    await maze.updateComplete;
    maze.shadowRoot!.querySelector<HTMLButtonElement>("#generate")!.click();
    const cell = () => maze.shadowRoot!.querySelector<HTMLElement>('.cell[data-x="1"][data-y="0"]');
    await vi.waitFor(() => expect(cell()).not.toBeNull());
    cell()!.click();
    await vi.waitFor(() => expect(maze.shadowRoot!.querySelector<HTMLButtonElement>("#why")!.disabled).toBe(false));
    maze.shadowRoot!.querySelector<HTMLButtonElement>("#why")!.click();

    await vi.waitFor(() => expect(tab(el, "explore").getAttribute("aria-selected")).toBe("true"));
    const explore = el.shadowRoot!.querySelector("explore-view")!;
    await vi.waitFor(() => expect(explore.shadowRoot!.querySelector(".centre .glyph")?.textContent).toBe("┐"));
    const heading = () => explore.shadowRoot!.querySelector("why-band")?.shadowRoot!.querySelector("#why-heading");
    await vi.waitFor(() => expect(heading()?.textContent).toContain("col 1, row 0"));

    explore.shadowRoot!.querySelector<HTMLButtonElement>("#back")!.click();
    await vi.waitFor(() => expect(tab(el, "maze").getAttribute("aria-selected")).toBe("true"));
    expect(cell()!.classList).toContain("selected");
  });

  test("a tile set that can't load at all shows the error in the banner", async () => {
    mockIPC((cmd) => {
      if (cmd === COMMANDS.coreVersion) return "9.9.9";
      const error: ToolError = { kind: "not_loaded", messages: ["data/tile-sets/core.toml: missing [bases]"] };
      throw error;
    });
    const el = mount();
    await vi.waitFor(() =>
      expect(el.shadowRoot!.querySelector(".banner")?.textContent).toBe("data/tile-sets/core.toml: missing [bases]"),
    );
  });
});

// A backend for the whole app: an empty table and a one-cell maze.

const HEDGE: TileView = {
  id: "hedge",
  name: "Hedge",
  kind: "hedge",
  tags: [],
  orientation: 0,
  sides: ["hedge", "hedge", "hedge", "hedge"],
  weight: 1,
};

interface Backend {
  generated: GenerateRequest[];
  tableRequests: number;
  reloadFails?: ToolError;
}

function backend(): Backend {
  const state: Backend = { generated: [], tableRequests: 0 };
  mockIPC((cmd, payload) => {
    switch (cmd) {
      case COMMANDS.coreVersion:
        return "9.9.9";
      case COMMANDS.tableView: {
        state.tableRequests++;
        const view: TableView = {
          revision: 1,
          tileSetPath: "data/tile-sets/core.toml",
          loadedAt: "2026-10-31T19:14:03Z",
          tileSetName: "core",
          openingPressure: 1,
          noise: null,
          tiles: [HEDGE],
          totalRows: 1,
          totalColumns: 1,
          rows: [],
          columns: [],
          directions: ["east"],
          exceptions: [],
          entries: [],
        };
        return view;
      }
      case COMMANDS.reloadTileSet:
        if (state.reloadFails) throw state.reloadFails;
        return { revision: 1, tileSetPath: "data/tile-sets/core.toml", loadedAt: "2026-10-31T20:20:00+01:00" };
      case COMMANDS.compareMazes:
      case COMMANDS.generateMaze: {
        const request = (payload as { request: GenerateRequest }).request;
        state.generated.push(request);
        const maze = {
          revision: 1,
          seed: request.seed,
          width: 1,
          height: 1,
          cells: [0],
          centre: { x: 0, y: 0 },
          restarts: 0,
          failed: false,
          measures: [],
        };
        // No previous load here, so compare has no before and the maze just regenerates.
        return cmd === COMMANDS.compareMazes ? { before: null, after: maze, changed: [], edits: [] } : maze;
      }
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return state;
}

const tab = (el: MazeToolApp, id: string) => el.shadowRoot!.querySelector<HTMLButtonElement>(`#tab-${id}`)!;
const panel = (el: MazeToolApp, id: string) => el.shadowRoot!.querySelector<HTMLElement>(`#panel-${id}`)!;

/** Opens the Maze tab and generates seed 48213; returns the maze view. */
async function generateOnce(el: MazeToolApp) {
  await el.updateComplete;
  tab(el, "maze").click();
  const maze = el.shadowRoot!.querySelector("maze-view")!;
  await maze.updateComplete;
  const seed = maze.shadowRoot!.querySelector<HTMLInputElement>("#seed")!;
  seed.value = "48213";
  seed.dispatchEvent(new Event("input"));
  await maze.updateComplete;
  maze.shadowRoot!.querySelector<HTMLButtonElement>("#generate")!.click();
  await vi.waitFor(() => expect(maze.shadowRoot!.querySelector(".row")).not.toBeNull());
  return maze;
}
