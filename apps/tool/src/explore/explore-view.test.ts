import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import { TOOL_ERROR_EVENT } from "../tool-error.ts";
import "./explore-view.ts";
import { TRAIL_SHOWN, type ExploreView } from "./explore-view.ts";
import { CORNER_NE, CORNER_SW, DOOR_E, DOOR_W, HEDGE, ROOM, STRAIGHT, TILES, fixtureBackend } from "./fixture.ts";
import { BACK_TO_MAZE_EVENT } from "./open-explore.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { TileView } from "../bindings/TileView.ts";

// The fixture's numbers, worked out by hand from its weights:
// East of ─ (row total 3.3): ┐ 1.2 and ┘ 1.2 tie at 36.4, ─ 0.5 (an
// exception, ×0.5) is 15.2, ╢ 0.4 is 12.1; └ · ╟ ░ can't go there.

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

async function mount(): Promise<ExploreView> {
  const el = document.createElement("explore-view");
  document.body.append(el);
  await vi.waitFor(() => expect(picker(el).options.length).toBe(TILES.length + 1));
  return el;
}

const root = (el: Element) => el.shadowRoot!;
const squash = (s: string | null | undefined) => (s ?? "").replace(/\s+/g, " ").trim();
const key = (t: TileView) => `${t.id}#${t.orientation}`;
const picker = (el: Element) => root(el).querySelector<HTMLSelectElement>("#centre")!;
const back = (el: Element) => root(el).querySelector<HTMLButtonElement>("#back")!;
const arm = (el: Element, d: Direction) => root(el).querySelector<HTMLElement>(`section.arm.${d}`)!;
/** The arm's visible rows (not its 0s), as glyphs in order. */
const armGlyphs = (el: Element, d: Direction) =>
  [...arm(el, d).querySelectorAll("ol.rows:not(.zeros) .row .glyph")].map((g) => g.textContent);
const row = (el: Element, d: Direction, t: TileView) => arm(el, d).querySelector<HTMLElement>(`.row[data-tile="${key(t)}"]`)!;
const panelText = (el: Element) => squash(root(el).querySelector("table-explanation")!.shadowRoot!.textContent);
const centreGlyph = (el: Element) => root(el).querySelector(".centre .glyph")?.textContent;
const trailGlyphs = (el: Element) => [...root(el).querySelectorAll(".trail .stop .glyph")].map((g) => g.textContent);

async function pick(el: ExploreView, tile: TileView) {
  const select = picker(el);
  select.value = String(TILES.findIndex((t) => key(t) === key(tile)));
  select.dispatchEvent(new Event("change"));
  await vi.waitFor(() => expect(centreGlyph(el)).toBe(glyph(tile)));
}

async function go(el: ExploreView, d: Direction, tile: TileView) {
  row(el, d, tile).querySelector<HTMLButtonElement>(".go")!.click();
  await vi.waitFor(() => expect(centreGlyph(el)).toBe(glyph(tile)));
}

const GLYPHS: Record<string, string> = {
  "corridor_straight#1": "─",
  "corridor_corner#0": "└",
  "corridor_corner#2": "┐",
  "corridor_corner#3": "┘",
  "open_room#0": "·",
  "room_doorway#1": "╟",
  "room_doorway#3": "╢",
  "hedge#0": "░",
};
const glyph = (t: TileView) => GLYPHS[key(t)];

describe("explore-view", () => {
  test("opens on the Centre picker, with every tile in the table's order and Back disabled", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    expect(ipc.calls.map((c) => c.cmd)).toEqual([COMMANDS.tableView]);
    const options = [...picker(el).options].map((o) => squash(o.textContent));
    expect(options).toEqual([
      "Pick a tile…",
      "─ Corridor straight, E–W",
      "└ Corridor corner, open N and E",
      "┐ Corridor corner, open S and W",
      "┘ Corridor corner, open W and N",
      "· Open room",
      "╟ Room doorway, door E",
      "╢ Room doorway, door W",
      "░ Hedge",
    ]);
    expect(squash(root(el).textContent)).toContain("Pick a centre tile");
    expect(back(el).disabled).toBe(true);
  });

  test("draws four arms around the centre, each headed with its direction and sorted by share", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);

    expect(squash(arm(el, "east").querySelector("h3")!.textContent)).toBe("EAST of ─ (% of ─'s east neighbours)");
    // Ties (┐ and ┘ at 36.4) keep the table's order.
    expect(armGlyphs(el, "east")).toEqual(["┐", "┘", "─", "╢"]);
    expect(armGlyphs(el, "north")).toEqual(["░", "└", "┘", "─"]);
    for (const d of ["north", "east", "south", "west"] as const) expect(arm(el, d)).not.toBeNull();
  });

  test("shows each edge's side kind on the centre box, and each neighbour's side facing back", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, DOOR_W);
    const edge = (d: string) => squash(root(el).querySelector(`.centre [data-edge="${d}"]`)!.textContent);
    expect([edge("north"), edge("east"), edge("south"), edge("west")]).toEqual([
      "Floor, wall at W end",
      "Floor",
      "Floor, wall at W end",
      "Door",
    ]);
    expect(squash(row(el, "west", STRAIGHT).querySelector(".faces")!.textContent)).toBe("faces with Passage");
    expect(squash(row(el, "west", DOOR_E).querySelector(".faces")!.textContent)).toBe("faces with Door");
  });

  test("shares, shade steps, the exception mark and the tooltip are the 1.1 cell's", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    const share = (t: TileView) => row(el, "east", t).querySelector<HTMLElement>(".share")!;
    expect(share(CORNER_SW).textContent?.trim()).toBe("36.4");
    expect(share(CORNER_SW).classList).toContain("step-4");
    expect(share(STRAIGHT).textContent?.trim()).toBe("15.2");
    expect(share(STRAIGHT).classList).toContain("step-3");
    expect(share(STRAIGHT).classList).toContain("exception");
    expect(share(CORNER_SW).classList).not.toContain("exception");
    expect(row(el, "east", CORNER_SW).title).toBe(
      "─ Corridor straight, E–W, then ┐ Corridor corner, open S and W to its east: 36.4%",
    );
  });

  test("hides the 0s behind a count that opens them, blank and without [Go]", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    const toggle = arm(el, "east").querySelector<HTMLButtonElement>(".zeros-toggle")!;
    expect(squash(toggle.textContent)).toBe("+ 4 can't go here");
    expect(arm(el, "east").querySelector(".zeros")).toBeNull();

    toggle.click();
    await vi.waitFor(() => expect(arm(el, "east").querySelector(".zeros")).not.toBeNull());
    const zeros = [...arm(el, "east").querySelectorAll(".zeros .row")];
    expect(zeros.map((r) => r.querySelector(".glyph")!.textContent)).toEqual(["└", "·", "╟", "░"]);
    for (const r of zeros) {
      expect(r.querySelector(".share")!.textContent?.trim()).toBe("");
      expect(r.querySelector(".go")).toBeNull();
    }
    expect(toggle.getAttribute("aria-expanded")).toBe("true");
  });

  test("selecting a row outlines it and DR-0025's panel explains the pair", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    expect(panelText(el)).toContain("Select a cell to see where its number comes from.");

    row(el, "east", CORNER_SW).click();
    await vi.waitFor(() => expect(row(el, "east", CORNER_SW).classList).toContain("selected"));
    const text = panelText(el);
    expect(text).toContain("36.4% of ─'s east neighbours");
    expect(text).toContain("east side: Passage");
    expect(text).toContain("west side: Passage");
    expect(text).toContain("× Corridor straight 1.0");
    expect(text).toContain("× Corridor corner 1.2");
    expect(text).toContain("÷ row total 3.3 = 36.4%");
  });

  test("selecting a 0 explains the clash", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    arm(el, "east").querySelector<HTMLButtonElement>(".zeros-toggle")!.click();
    await vi.waitFor(() => expect(row(el, "east", ROOM)).not.toBeNull());
    row(el, "east", ROOM).click();
    await vi.waitFor(() => expect(panelText(el)).toContain("Can't happen"));
    expect(panelText(el)).toContain("Passage meets Floor.");
  });

  test("[Go] makes the neighbour the centre and selects the tile we came from, marked 'from'", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    await go(el, "east", DOOR_W);

    expect(ipc.calls.at(-1)).toEqual({ cmd: COMMANDS.exploreTile, request: { tileId: "room_doorway", orientation: 3 } });
    const from = row(el, "west", STRAIGHT);
    expect(from.classList).toContain("selected");
    expect(squash(from.querySelector(".from")?.textContent)).toBe("from");
    // The panel shows the edge just crossed, from the new centre's side.
    expect(panelText(el)).toContain("43.1% of ╢'s west neighbours");
    expect(back(el).disabled).toBe(false);
  });

  test("[Go] in the panel, double-click and Enter on a row all step", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);

    row(el, "east", DOOR_W).click();
    await vi.waitFor(() => expect(root(el).querySelector("#explore-selected")).not.toBeNull());
    expect(squash(root(el).querySelector("#explore-selected")!.textContent)).toBe("Explore ╢");
    root(el).querySelector<HTMLButtonElement>("#explore-selected")!.click();
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("╢"));

    row(el, "east", ROOM).dispatchEvent(new MouseEvent("dblclick", { bubbles: true }));
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("·"));

    row(el, "east", DOOR_E).dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("╟"));
    expect(trailGlyphs(el)).toEqual(["─", "╢", "·", "╟"]);
  });

  test("[< Back] and Alt+Left return to the previous centre and its selection", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    await go(el, "east", DOOR_W);
    await go(el, "east", ROOM);

    back(el).click();
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("╢"));
    expect(row(el, "east", ROOM).classList).toContain("selected");

    el.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowLeft", altKey: true, bubbles: true }));
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("─"));
    expect(row(el, "east", DOOR_W).classList).toContain("selected");
    expect(back(el).disabled).toBe(true);
  });

  test("the demo walk: four [Go]s east through two doorways, then the trail home", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    await go(el, "east", DOOR_W);
    await go(el, "east", ROOM);
    await go(el, "east", DOOR_E);
    await go(el, "east", STRAIGHT);
    expect(trailGlyphs(el)).toEqual(["─", "╢", "·", "╟", "─"]);

    root(el).querySelector<HTMLButtonElement>('.trail .stop[data-index="0"]')!.click();
    await vi.waitFor(() => expect(trailGlyphs(el)).toEqual(["─"]));
    expect(row(el, "east", DOOR_W).classList).toContain("selected");
  });

  test("a long trail is cut at its oldest end", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    for (let i = 0; i < TRAIL_SHOWN + 2; i++) await go(el, "east", STRAIGHT);
    await vi.waitFor(() => expect(trailGlyphs(el)).toHaveLength(TRAIL_SHOWN));
    expect(root(el).querySelector(".trail .cut")?.textContent).toBe("…");
  });

  test("the entry point centres on any tile, selects the given neighbour and starts a new trail", async () => {
    fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    await go(el, "east", DOOR_W);

    el.open({ centre: CORNER_NE, select: { direction: "east", neighbour: CORNER_SW } });
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("└"));
    expect(trailGlyphs(el)).toEqual(["└"]);
    expect(row(el, "east", CORNER_SW).classList).toContain("selected");
    expect(panelText(el)).toContain("of └'s east neighbours");
    expect(picker(el).value).toBe(String(TILES.indexOf(CORNER_NE)));
  });

  test("the entry point can select a 0, which opens that arm's 0s", async () => {
    fixtureBackend();
    const el = await mount();
    el.open({ centre: STRAIGHT, select: { direction: "east", neighbour: ROOM } });
    await vi.waitFor(() => expect(row(el, "east", ROOM)?.classList).toContain("selected"));
    expect(panelText(el)).toContain("Can't happen");
  });

  test("refresh after a reload keeps the centre, trail and selection", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    await go(el, "east", DOOR_W);
    const before = ipc.calls.length;

    ipc.revision = 2;
    el.refresh();
    await vi.waitFor(() => expect(ipc.calls.length).toBeGreaterThan(before));
    await vi.waitFor(() => expect(ipc.calls.at(-1)?.cmd).toBe(COMMANDS.tableView));
    expect(centreGlyph(el)).toBe("╢");
    expect(trailGlyphs(el)).toEqual(["─", "╢"]);
    expect(row(el, "west", STRAIGHT).classList).toContain("selected");
  });

  test("a centre the reloaded tile set doesn't have is named, and the picker gets focus", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    await pick(el, DOOR_W);
    ipc.tiles = TILES.filter((t) => t.id !== "room_doorway");
    ipc.revision = 2;

    el.refresh();
    await vi.waitFor(() =>
      expect(squash(root(el).querySelector(".problem")?.textContent)).toBe(
        "Room doorway, door W isn't in the tile set any more. Pick a centre.",
      ),
    );
    expect(root(el).querySelector(".compass")).toBeNull();
    expect(root(el).activeElement).toBe(picker(el));
  });

  test("a failed request goes to the app's banner and the arms stay", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    await pick(el, STRAIGHT);
    const errors: string[] = [];
    el.addEventListener(TOOL_ERROR_EVENT, (e) => errors.push((e as CustomEvent<string>).detail));

    ipc.fails = { kind: "not_loaded", messages: ["data/tile-sets/core.toml: missing [bases]"] };
    el.refresh();
    await vi.waitFor(() => expect(errors).toEqual(["data/tile-sets/core.toml: missing [bases]"]));
    expect(armGlyphs(el, "east")).toEqual(["┐", "┘", "─", "╢"]);
  });
});

describe("explore-view, opened from a maze cell (DR-0033)", () => {
  const MAZE = { revision: 1, seed: 48213, width: 24, height: 16, cell: { x: 14, y: 2 } };
  const band = (el: Element) => root(el).querySelector("why-band");
  const bandRoot = (el: Element) => band(el)!.shadowRoot!;
  const factor = (el: Element, candidate: string, d: Direction) =>
    bandRoot(el).querySelector<HTMLElement>(`tbody tr[data-tile="${candidate}"] td.factor[data-direction="${d}"]`)!;

  async function opened() {
    const ipc = fixtureBackend();
    const el = await mount();
    el.open({ centre: CORNER_SW, maze: MAZE });
    await vi.waitFor(() => expect(bandRoot(el).querySelector("#why-heading")).not.toBeNull());
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("┐"));
    return { ipc, el };
  }

  test("replays the cell, shows the band above the compass, with nothing selected in the arms", async () => {
    const { ipc, el } = await opened();
    expect(ipc.calls.find((c) => c.cmd === COMMANDS.traceCell)?.request).toEqual(MAZE);
    expect(squash(bandRoot(el).querySelector("#why-heading")!.textContent)).toBe(
      "Why ┐ is here · seed 48213 · col 14, row 2 · drawn 41st of 384",
    );
    expect(root(el).querySelector(".row.selected")).toBeNull();
    expect(panelText(el)).toContain("Select a cell");
    expect(trailGlyphs(el)).toEqual(["┐"]);
  });

  test("[Go] away hides the band, and [< Back] brings it back", async () => {
    const { el } = await opened();
    await go(el, "west", STRAIGHT);
    expect(band(el)).toBeNull();
    back(el).click();
    await vi.waitFor(() => expect(centreGlyph(el)).toBe("┐"));
    expect(bandRoot(el).querySelector("#why-heading")).not.toBeNull();
  });

  test("a factor opens DR-0025's panel on its pair, and it and an arm row clear each other", async () => {
    const { el } = await opened();
    factor(el, "corridor_straight#1", "west").click();
    await vi.waitFor(() => expect(factor(el, "corridor_straight#1", "west").classList).toContain("selected"));
    // Candidate ─ as row tile, the counted ─ to its west as the neighbour.
    expect(panelText(el)).toContain("of ─'s west neighbours");
    expect(panelText(el)).toContain("× Long straights are rarer than turns 0.5");

    row(el, "north", HEDGE).click();
    await vi.waitFor(() => expect(row(el, "north", HEDGE).classList).toContain("selected"));
    expect(bandRoot(el).querySelector("td.factor.selected")).toBeNull();
    expect(panelText(el)).toContain("of ┐'s north neighbours");

    factor(el, "corridor_corner#2", "north").click();
    await vi.waitFor(() => expect(root(el).querySelector(".row.selected")).toBeNull());
    expect(panelText(el)).toContain("of ┐'s north neighbours");
    expect(factor(el, "corridor_corner#2", "north").classList).toContain("selected");
  });

  test("each arm marks the tile the maze has on that side", async () => {
    const { el } = await opened();
    const marked = (d: Direction) => [...arm(el, d).querySelectorAll(".row")].filter((r) => r.querySelector(".in-maze"));
    expect(marked("north").map((r) => r.getAttribute("data-tile"))).toEqual(["hedge#0"]);
    expect(marked("west").map((r) => r.getAttribute("data-tile"))).toEqual(["corridor_straight#1"]);
    expect(marked("south").map((r) => r.getAttribute("data-tile"))).toEqual(["corridor_corner#0"]);
    expect(marked("east")).toEqual([]);
  });

  test("< Back to maze at the first stop asks for the Maze tab", async () => {
    const { el } = await opened();
    expect(squash(back(el).textContent)).toBe("< Back to maze");
    expect(back(el).disabled).toBe(false);
    let asked = 0;
    el.addEventListener(BACK_TO_MAZE_EVENT, () => asked++);
    back(el).click();
    expect(asked).toBe(1);

    await go(el, "west", STRAIGHT);
    expect(squash(back(el).textContent)).toBe("< Back");
  });

  test("a maze from a tile set reloaded since is never traced: the band says so", async () => {
    const ipc = fixtureBackend();
    const el = await mount();
    ipc.revision = 2;
    el.open({ centre: CORNER_SW, maze: MAZE });
    await vi.waitFor(() =>
      expect(squash(bandRoot(el).textContent)).toBe("The tile set was reloaded. Pick the cell again in the Maze tab."),
    );
  });

  test("a reload clears the band's trace with the same note, and keeps the arms", async () => {
    const { ipc, el } = await opened();
    factor(el, "corridor_corner#2", "west").click();
    ipc.revision = 2;
    el.refresh();
    await vi.waitFor(() =>
      expect(squash(bandRoot(el).textContent)).toBe("The tile set was reloaded. Pick the cell again in the Maze tab."),
    );
    expect(centreGlyph(el)).toBe("┐");
    expect(panelText(el)).toContain("Select a cell");
  });

  test("opening without a maze cell shows no band", async () => {
    const { el } = await opened();
    el.open({ centre: CORNER_SW });
    await vi.waitFor(() => expect(band(el)).toBeNull());
    expect(squash(back(el).textContent)).toBe("< Back");
    expect(back(el).disabled).toBe(true);
  });
});
