import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import "./maze-view.ts";
import type { MazeView } from "./maze-view.ts";
import type { GenerateRequest, GeneratedMaze } from "./maze-types.ts";
import type { TableView } from "../bindings/TableView.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { ToolError } from "../bindings/ToolError.ts";

// Five tiles in table order: ─ ┐ (corridor), ═ ╔ (room), ░ (hedge).
const tile = (id: string, name: string, kind: TileView["kind"], orientation: number): TileView => ({
  id,
  name,
  kind,
  tags: [],
  orientation,
  sides: ["hedge", "hedge", "hedge", "hedge"],
  weight: 1,
});
const TILES: TileView[] = [
  tile("corridor_straight", "Corridor straight", "corridor", 1),
  tile("corridor_corner", "Corridor corner", "corridor", 2),
  tile("room_wall", "Room wall", "room", 0),
  tile("room_corner", "Room corner", "room", 3),
  tile("hedge", "Hedge", "hedge", 0),
];

// A 3×2 maze:   ─┐░
//               ╔═░   centre at col 1, row 1 (the ═).
function mazeFor(request: GenerateRequest, overrides: Partial<GeneratedMaze> = {}): GeneratedMaze {
  return {
    revision: 1,
    seed: request.seed,
    width: 3,
    height: 2,
    cells: [0, 1, 4, 3, 2, 4],
    centre: { x: 1, y: 1 },
    restarts: 2,
    failed: false,
    measures: [
      {
        name: "Reachable from centre",
        value: "3 of 4 walkable cells (75%)",
        explanation: "Share of walkable cells the player could walk to from the middle of the maze.",
      },
      { name: "Separate areas", value: "2", explanation: "Groups of walkable cells cut off from each other. 1 means everything joins up." },
    ],
    ...overrides,
  };
}

function tablePayload(revision: number): TableView {
  return {
    revision,
    tileSetPath: "data/tile-sets/core.toml",
    loadedAt: "2026-10-31T19:14:03Z",
    tileSetName: "core",
    noise: null,
    tiles: TILES,
    totalRows: 5,
    totalColumns: 5,
    rows: [],
    columns: [],
    directions: ["east"],
    exceptions: [],
    entries: [],
  };
}

interface Backend {
  generated: GenerateRequest[];
  tableCalls: number;
  revision: number;
  next?: Partial<GeneratedMaze>;
  fails?: ToolError;
  /** Holds generate_maze until released, to see the generating state. */
  hold?: Promise<void>;
}

function backend(): Backend {
  const state: Backend = { generated: [], tableCalls: 0, revision: 1 };
  mockIPC(async (cmd, payload) => {
    if (cmd === COMMANDS.generateMaze) {
      const request = (payload as { request: GenerateRequest }).request;
      state.generated.push(request);
      if (state.hold) await state.hold;
      if (state.fails) throw state.fails;
      return mazeFor(request, { revision: state.revision, ...state.next });
    }
    if (cmd === COMMANDS.tableView) {
      state.tableCalls++;
      return tablePayload(state.revision);
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return state;
}

async function mount(): Promise<MazeView> {
  const el = document.createElement("maze-view");
  document.body.append(el);
  await el.updateComplete;
  return el;
}

const root = (el: Element) => el.shadowRoot!;
const $ = <T extends HTMLElement>(el: Element, selector: string) => root(el).querySelector<T>(selector)!;
const text = (el: Element) => (root(el).textContent ?? "").replace(/\s+/g, " ");
const seedField = (el: Element) => $<HTMLInputElement>(el, "#seed");
const mapText = (el: Element) => [...root(el).querySelectorAll(".row")].map((r) => r.textContent);

function typeSeed(el: Element, value: string) {
  const input = seedField(el);
  input.value = value;
  input.dispatchEvent(new Event("input"));
}

async function generated(el: MazeView) {
  await vi.waitFor(() => expect(root(el).querySelector(".row")).not.toBeNull());
}

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("maze-view", () => {
  test("opens with a random seed and an empty box, and asks Rust for nothing", async () => {
    const ipc = backend();
    const el = await mount();
    expect(seedField(el).value).toMatch(/^\d+$/);
    expect(text(el)).toContain("Press Generate to make a maze from this seed.");
    expect($<HTMLButtonElement>(el, "#generate").disabled).toBe(false);
    expect(ipc.generated).toHaveLength(0);
  });

  test("Generate sends the seed and the default size, and draws two characters per cell", async () => {
    const ipc = backend();
    const el = await mount();
    typeSeed(el, "48213");
    $(el, "#generate").click();
    await generated(el);

    expect(ipc.generated).toEqual([{ seed: 48213, width: 24, height: 16 }]);
    expect(mapText(el)).toEqual(["──┐ ░░", "╔═══░░"]);
    expect($(el, "#status").textContent).toBe("Seed 48213 · 3×2 · restarted 2 times");
  });

  test("cells carry their kind, tint and name, and the centre is outlined", async () => {
    backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);

    const cell = (x: number, y: number) => $(el, `.cell[data-x="${x}"][data-y="${y}"]`);
    expect(cell(0, 0).classList).toContain("corridor");
    expect(cell(0, 0).getAttribute("aria-label")).toBe("Corridor straight, E–W");
    expect(cell(1, 1).classList).toContain("room");
    expect(cell(1, 1).classList).toContain("tint-lower");
    expect(cell(1, 1).classList).toContain("centre");
    expect(cell(0, 1).classList).toContain("tint-lower-right");
    expect(cell(2, 0).classList).toContain("hedge");
    expect(root(el).querySelectorAll(".cell.centre")).toHaveLength(1);
  });

  test("hovering a cell names it in the readout, and leaving clears it", async () => {
    backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);

    $(el, '.cell[data-x="1"][data-y="0"]').dispatchEvent(new MouseEvent("mouseover", { bubbles: true }));
    await vi.waitFor(() => expect($(el, "#readout").textContent).toBe("Corridor corner, open S and W · col 1, row 0"));
    $(el, ".map").dispatchEvent(new MouseEvent("mouseleave"));
    await vi.waitFor(() => expect($(el, "#readout").textContent).toBe(""));
  });

  test("shows each measure with its explanation and the centre key", async () => {
    backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);

    const measures = text(el);
    expect(measures).toContain("Reachable from centre: 3 of 4 walkable cells (75%)");
    expect(measures).toContain("Share of walkable cells the player could walk to from the middle of the maze.");
    expect(measures).toContain("Separate areas: 2");
    expect(measures).toContain("centre");
  });

  test("Enter in the seed field generates", async () => {
    const ipc = backend();
    const el = await mount();
    typeSeed(el, "7");
    seedField(el).dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    await generated(el);
    expect(ipc.generated.map((r) => r.seed)).toEqual([7]);
  });

  test("a seed that isn't a whole number disables Generate and says why", async () => {
    const ipc = backend();
    const el = await mount();
    typeSeed(el, "12a");
    await el.updateComplete;
    expect($<HTMLButtonElement>(el, "#generate").disabled).toBe(true);
    expect($(el, "#seed-problem").textContent).toBe("A seed is a whole number.");
    seedField(el).dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    await el.updateComplete;
    expect(ipc.generated).toHaveLength(0);
  });

  test("Random seed fills the field and generates in one press", async () => {
    const ipc = backend();
    const el = await mount();
    $(el, "#random-seed").click();
    await generated(el);
    expect(ipc.generated).toHaveLength(1);
    expect(seedField(el).value).toBe(String(ipc.generated[0].seed));
    expect($(el, "#status").textContent).toContain(`Seed ${ipc.generated[0].seed}`);
  });

  test("editing the field keeps the maze and its seed until Generate", async () => {
    backend();
    const el = await mount();
    typeSeed(el, "5");
    $(el, "#generate").click();
    await generated(el);
    typeSeed(el, "6");
    await el.updateComplete;
    expect($(el, "#status").textContent).toContain("Seed 5 ");
  });

  test("while generating, the button says so and the old maze is dimmed", async () => {
    const ipc = backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);

    let release!: () => void;
    ipc.hold = new Promise((resolve) => (release = resolve));
    $(el, "#generate").click();
    await vi.waitFor(() => expect($(el, "#generate").textContent?.trim()).toBe("Generating…"));
    expect($<HTMLButtonElement>(el, "#generate").disabled).toBe(true);
    expect($(el, ".map").classList).toContain("dimmed");

    release();
    await vi.waitFor(() => expect($(el, "#generate").textContent?.trim()).toBe("Generate"));
    expect($(el, ".map").classList).not.toContain("dimmed");
  });

  test("a maze that gave up clears the area, says so in red, and hides the measures", async () => {
    const ipc = backend();
    ipc.next = { failed: true, restarts: 20, cells: [0, null, null, null, null, null] };
    const el = await mount();
    typeSeed(el, "48213");
    $(el, "#generate").click();
    await vi.waitFor(() => expect(root(el).querySelector(".failed")).not.toBeNull());

    expect($(el, ".failed").textContent).toBe(
      "No maze for seed 48213: gave up after 20 restarts. Try another seed, or look in the table for tiles that can't meet.",
    );
    expect(root(el).querySelector(".row")).toBeNull();
    expect(text(el)).not.toContain("Reachable from centre");

    // The tool stays usable.
    ipc.next = undefined;
    $(el, "#generate").click();
    await generated(el);
  });

  test("a failed command shows Rust's error and keeps the last maze", async () => {
    const ipc = backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);

    ipc.fails = { kind: "not_loaded", messages: ["data/tile-sets/core.toml: missing [bases]"] };
    $(el, "#generate").click();
    await vi.waitFor(() => expect($(el, ".error").textContent).toBe("data/tile-sets/core.toml: missing [bases]"));
    expect(mapText(el)).toEqual(["──┐ ░░", "╔═══░░"]);
  });

  test("fetches the tile list once per tile set revision", async () => {
    const ipc = backend();
    const el = await mount();
    $(el, "#generate").click();
    await generated(el);
    $(el, "#generate").click();
    await vi.waitFor(() => expect(ipc.generated).toHaveLength(2));
    await vi.waitFor(() => expect($(el, "#generate").textContent?.trim()).toBe("Generate"));
    expect(ipc.tableCalls).toBe(1);

    ipc.revision = 2;
    $(el, "#generate").click();
    await vi.waitFor(() => expect(ipc.tableCalls).toBe(2));
  });

  test("after a reload, a shown maze regenerates from its seed", async () => {
    const ipc = backend();
    const el = await mount();
    typeSeed(el, "48213");
    $(el, "#generate").click();
    await generated(el);
    typeSeed(el, "99");

    el.regenerateAfterReload();
    await vi.waitFor(() => expect($(el, "#status").textContent).toContain("regenerated after reload"));
    expect(ipc.generated.map((r) => r.seed)).toEqual([48213, 48213]);
    expect($(el, "#status").textContent).toContain("Seed 48213 · regenerated after reload");

    // The next Generate is an ordinary one again.
    $(el, "#generate").click();
    await vi.waitFor(() => expect($(el, "#status").textContent).toBe("Seed 99 · 3×2 · restarted 2 times"));
  });

  test("a reload with no maze shown generates nothing", async () => {
    const ipc = backend();
    const el = await mount();
    el.regenerateAfterReload();
    await el.updateComplete;
    expect(ipc.generated).toHaveLength(0);
  });
});
