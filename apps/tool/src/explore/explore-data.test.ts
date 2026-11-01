import { afterEach, describe, expect, test } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import { loadExplore, loadWhy, type MazeCell } from "./explore-data.ts";
import {
  CORNER_NE,
  CORNER_SW,
  DOOR_W,
  HEDGE,
  ROOM,
  STRAIGHT,
  TILES,
  entryFor,
  fixtureBackend,
  fixtureExplore,
  fixtureTableView,
} from "./fixture.ts";
import type { CellTrace } from "../bindings/CellTrace.ts";
import type { ExploreView } from "../bindings/ExploreView.ts";
import type { TableViewRequest } from "../bindings/TableViewRequest.ts";

afterEach(() => clearMocks());

const glyphIds = (tiles: { id: string; orientation: number }[]) => tiles.map((t) => `${t.id}#${t.orientation}`);
const idOf = (t: { id: string; orientation: number }) => `${t.id}#${t.orientation}`;
const EAST = 1;

/** A backend that answers `explore_tile` with `view` as given, and table_view from the fixture. */
function backendWith(view: ExploreView, revision = 1) {
  const calls: string[] = [];
  mockIPC((cmd, payload) => {
    calls.push(cmd);
    if (cmd === COMMANDS.exploreTile) return view;
    if (cmd === COMMANDS.tableView) return fixtureTableView((payload as { request: TableViewRequest }).request, revision);
    throw new Error(`unexpected command ${cmd}`);
  });
  return calls;
}

describe("loadExplore", () => {
  test("with no centre, asks only for the tile list, for the Centre picker", async () => {
    const ipc = fixtureBackend();
    const data = await loadExplore();
    expect(ipc.calls.map((c) => c.cmd)).toEqual([COMMANDS.tableView]);
    const request = ipc.calls[0].request as TableViewRequest;
    expect(request.rows.count).toBe(0);
    expect(glyphIds(data.tiles)).toEqual(glyphIds(TILES));
    expect(data.centre).toBeUndefined();
  });

  test("names the centre by tile type and orientation, and resolves every index to its tile", async () => {
    const ipc = fixtureBackend();
    const data = await loadExplore({ id: "corridor_straight", orientation: 1 });
    expect(ipc.calls[0]).toEqual({ cmd: COMMANDS.exploreTile, request: { tileId: "corridor_straight", orientation: 1 } });
    const centre = data.centre!;
    expect(idOf(centre.tile)).toBe(idOf(STRAIGHT));
    expect(Object.keys(centre.arms)).toEqual(["north", "east", "south", "west"]);
    const east = centre.arms.east;
    expect(east.centreSide).toBe("passage");
    for (const n of [...east.neighbours, ...east.zeros]) expect(idOf(n.tile)).toBe(idOf(TILES[n.entry.column]));
    expect(data.exceptions[0].name).toBe("Long straights are rarer than turns");
  });

  test("keeps Rust's order and numbers: it never sorts or recalculates", async () => {
    // Deliberately unsorted, to show the adapter passes the order through.
    const odd = { ...entryFor(EAST, 0, 7), probability: 0.9 };
    const view: ExploreView = {
      revision: 1,
      centre: 0,
      arms: (["north", "east", "south", "west"] as const).map((direction) => ({
        direction,
        centreSide: "hedge" as const,
        neighbours: direction === "east" ? [entryFor(EAST, 0, 6), entryFor(EAST, 0, 2), odd] : [],
        zeros: direction === "east" ? [entryFor(EAST, 0, 4), entryFor(EAST, 0, 1)] : [],
      })) as unknown as ExploreView["arms"],
    };
    backendWith(view);
    const east = (await loadExplore({ id: "corridor_straight", orientation: 1 })).centre!.arms.east;
    expect(east.neighbours.map((n) => idOf(n.tile))).toEqual([idOf(DOOR_W), idOf(CORNER_SW), idOf(HEDGE)]);
    expect(east.neighbours[2].entry).toBe(odd);
    expect(east.zeros.map((n) => idOf(n.tile))).toEqual([idOf(ROOM), idOf(CORNER_NE)]);
    expect(east.centreSide).toBe("hedge");
  });

  test("reuses a tile list it already holds at the same revision, and refetches at a new one", async () => {
    const calls = backendWith(fixtureExplore({ tileId: "hedge", orientation: 0 }, 2), 2);
    const known = { revision: 2, tiles: TILES, exceptions: [] };
    await loadExplore(HEDGE, known);
    expect(calls).toEqual([COMMANDS.exploreTile]);

    calls.length = 0;
    await loadExplore(HEDGE, { ...known, revision: 1 });
    expect(calls).toEqual([COMMANDS.exploreTile, COMMANDS.tableView]);
  });

  test("a centre the tile set no longer has comes back without a centre, not as an error", async () => {
    const ipc = fixtureBackend();
    ipc.tiles = TILES.filter((t) => t.id !== "room_doorway");
    const data = await loadExplore({ id: "room_doorway", orientation: 3 });
    expect(data.centre).toBeUndefined();
    expect(data.tiles).toHaveLength(6);
  });

  test("a bad_request for a tile that does exist, and any other error, rejects as given", async () => {
    mockIPC((cmd, payload) => {
      if (cmd === COMMANDS.exploreTile) throw { kind: "bad_request", messages: ["something else"] };
      return fixtureTableView((payload as { request: TableViewRequest }).request);
    });
    await expect(loadExplore(HEDGE)).rejects.toEqual({ kind: "bad_request", messages: ["something else"] });

    const ipc = fixtureBackend();
    ipc.fails = { kind: "not_loaded", messages: ["no tile set"] };
    await expect(loadExplore(HEDGE)).rejects.toEqual({ kind: "not_loaded", messages: ["no tile set"] });
  });
});

const CELL: MazeCell = { revision: 1, seed: 48213, width: 24, height: 16, cell: { x: 14, y: 2 } };

/** ┐ drawn east of a counted ─, with ┘ the other candidate; north is later, south the edge. */
function trace(): CellTrace {
  return {
    revision: 1,
    cell: CELL.cell,
    restarts: 0,
    failed: false,
    steps: 384,
    collapse: {
      step: 41,
      neighbours: [
        { direction: "north", state: "later", tile: null },
        { direction: "east", state: "edge", tile: null },
        { direction: "south", state: "later", tile: 7 },
        { direction: "west", state: "counted", tile: 0 },
      ],
      candidates: [
        {
          tile: 3,
          tileWeight: 1.2,
          factors: [
            { direction: "west", neighbour: 0, base: 1, exceptions: [], multiplier: 1, noise: 1.04, factor: 1.04, entry: entryFor(3, 3, 0) },
          ],
          pressure: 1,
          score: 1.248,
          share: 0.51,
        },
        {
          tile: 2,
          tileWeight: 1.2,
          factors: [
            { direction: "west", neighbour: 0, base: 1, exceptions: [], multiplier: 1, noise: 1, factor: 1, entry: entryFor(3, 2, 0) },
          ],
          pressure: 1,
          score: 1.2,
          share: 0.49,
        },
      ],
      ruledOut: 6,
      total: 2.448,
      chosen: 2,
    },
  };
}

describe("loadWhy", () => {
  test("sends the maze cell and resolves the trace's tiles, keeping Rust's order and numbers", async () => {
    const ipc = fixtureBackend();
    ipc.trace = trace;
    const result = await loadWhy(CELL);
    expect(ipc.calls[0]).toEqual({ cmd: COMMANDS.traceCell, request: CELL });
    if (!("why" in result)) throw new Error("expected a trace");
    const why = result.why;
    expect(idOf(why.tile)).toBe(idOf(CORNER_SW));
    expect([why.step, why.steps, why.ruledOut, why.total, why.seed]).toEqual([41, 384, 6, 2.448, 48213]);
    expect(why.neighbours.map((n) => [n.direction, n.state, n.tile && idOf(n.tile)])).toEqual([
      ["north", "later", undefined],
      ["east", "edge", undefined],
      ["south", "later", idOf(HEDGE)],
      ["west", "counted", idOf(STRAIGHT)],
    ]);
    // Rust's order, even with the drawn tile second; "drawn" is the chosen tile.
    expect(why.candidates.map((c) => [idOf(c.tile), c.drawn, c.score, c.share])).toEqual([
      ["corridor_corner#3", false, 1.248, 0.51],
      ["corridor_corner#2", true, 1.2, 0.49],
    ]);
    expect(why.candidates[0].factors.map((f) => [f.direction, idOf(f.neighbour), f.factor])).toEqual([
      ["west", idOf(STRAIGHT), 1.04],
    ]);
    expect(why.candidates[0].factors[0].entry).toEqual(entryFor(3, 3, 0));
  });

  test("a maze from a reloaded tile set is stale, not an error", async () => {
    const ipc = fixtureBackend();
    ipc.trace = trace;
    ipc.revision = 2;
    expect(await loadWhy(CELL)).toEqual({ stale: true });
  });

  test("a cell the attempt never reached has nothing placed", async () => {
    const ipc = fixtureBackend();
    ipc.trace = () => ({ ...trace(), failed: true, collapse: null });
    expect(await loadWhy(CELL)).toEqual({ notPlaced: true });
  });
});
