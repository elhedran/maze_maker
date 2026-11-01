// Test fixture for the explore view: eight tiles from the core set and
// stand-ins for Rust's `table_view` and `explore_tile`, which work entries
// out the way DR-0022 does (side-pair base × both weights × exception, no
// noise) and sort arms the way DR-0034 rule 3 does. `trace_cell` answers
// whatever a test sets. Only tests import this file; the app never does.
//
// The tiles are enough for DR-0026's demo walk, east all the way:
// ─ (Passage | Door) ╢ (Floor | Floor) · (Floor | Floor) ╟ (Door | Passage) ─.
// ┐ and ┘ have the same weight, so East of ─ has a tie to sort.

import { mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS } from "../ipc.ts";
import { DIRECTIONS } from "../table/format.ts";
import type { SideKind } from "../bindings/SideKind.ts";
import type { CompareRequest, CompareResult, EditRequest, EditResult, EntryView, TableView } from "../pending-bindings.ts";
import type { TableViewRequest } from "../bindings/TableViewRequest.ts";
import type { TileKind } from "../bindings/TileKind.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { CellTrace } from "../bindings/CellTrace.ts";
import type { ExploreRequest } from "../bindings/ExploreRequest.ts";
import type { ExploreView } from "../bindings/ExploreView.ts";
import type { PairFactor } from "../bindings/PairFactor.ts";
import type { TraceCellRequest } from "../bindings/TraceCellRequest.ts";
import type { GenerateRequest } from "../bindings/GenerateRequest.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";

/** A rejected command, as Rust sends it; `kind` may be DR-0034's `stale`. */
export type FixtureError = { kind: string; messages: string[] };

const tile = (
  id: string,
  name: string,
  kind: TileKind,
  orientation: number,
  sides: [SideKind, SideKind, SideKind, SideKind],
  weight: number,
): TileView => ({ id, name, kind, tags: [], orientation, sides, weight });

/** In table order. Index: 0 ─, 1 └, 2 ┐, 3 ┘, 4 ·, 5 ╟, 6 ╢, 7 ░. */
export const TILES: TileView[] = [
  tile("corridor_straight", "Corridor straight", "corridor", 1, ["hedge", "passage", "hedge", "passage"], 1),
  tile("corridor_corner", "Corridor corner", "corridor", 0, ["passage", "passage", "hedge", "hedge"], 1.2),
  tile("corridor_corner", "Corridor corner", "corridor", 2, ["hedge", "hedge", "passage", "passage"], 1.2),
  tile("corridor_corner", "Corridor corner", "corridor", 3, ["passage", "hedge", "hedge", "passage"], 1.2),
  tile("open_room", "Open room", "room", 0, ["floor", "floor", "floor", "floor"], 0.8),
  tile("room_doorway", "Room doorway", "room", 1, ["floor_wall_e", "door", "floor_wall_e", "floor"], 0.4),
  tile("room_doorway", "Room doorway", "room", 3, ["floor_wall_w", "floor", "floor_wall_w", "door"], 0.4),
  tile("hedge", "Hedge", "hedge", 0, ["hedge", "hedge", "hedge", "hedge"], 1.5),
];

export const [STRAIGHT, CORNER_NE, CORNER_SW, CORNER_NW, ROOM, DOOR_E, DOOR_W, HEDGE] = TILES;

/** Which sides may meet (DR-0015) and the bases from core.toml. */
function mayMeet(side: SideKind): SideKind[] {
  if (side === "passage" || side === "door") return ["passage", "door"];
  return [side];
}

/** The `[bases]` key for two sides that may meet (side.rs's BasePair, for the fixture only). */
function basePair(a: SideKind, b: SideKind): string {
  const word = (s: SideKind) => (s.startsWith("floor_wall") ? "floor_wall" : s);
  const [x, y] = [word(a), word(b)].sort((p, q) => (p === "passage" ? -1 : q === "passage" ? 1 : p.localeCompare(q)));
  return `${x}_${y}`;
}

function base(a: SideKind, b: SideKind): number {
  return a === "door" && b === "door" ? 0.3 : 1;
}

/** "Long straights are rarer than turns": straight beside straight, ×0.5. */
const STRAIGHTS = { name: "Long straights are rarer than turns", multiplier: 0.5, rule: "tile corridor_straight (passage) beside tile corridor_straight (passage)" };
const isStraights = (r: TileView, c: TileView) => r.id === "corridor_straight" && c.id === "corridor_straight";

/** The `[bases]` key for two sides that may meet, as `BasePair::key()`. */
function basePair(a: SideKind, b: SideKind): string {
  const kind = (s: SideKind) => (s.startsWith("floor_wall") ? "floor_wall" : s);
  return kind(a) === kind(b) ? `${kind(a)}_${kind(b)}` : "passage_door";
}

function rawEntry(tiles: TileView[], direction: number, r: number, c: number): Omit<EntryView, "rowTotal" | "probability"> {
  const row = tiles[r];
  const column = tiles[c];
  const rowSide = row.sides[direction];
  const columnSide = column.sides[(direction + 2) % 4];
  const allowed = mayMeet(rowSide).includes(columnSide);
  const exceptions = allowed && isStraights(row, column) ? [0] : [];
  const b = allowed ? base(rowSide, columnSide) : 0;
  const multiplier = exceptions.length > 0 ? STRAIGHTS.multiplier : 1;
  return {
    direction: DIRECTIONS[direction],
    row: r,
    column: c,
    facing: { row: rowSide, column: columnSide },
    allowed,
    mayMeet: mayMeet(rowSide),
    base: b,
    basePair: allowed ? basePair(rowSide, columnSide) : null,
    rowWeight: row.weight,
    columnWeight: column.weight,
    exceptions,
    noise: 1,
    weight: allowed ? b * row.weight * column.weight * multiplier : 0,
  };
}

/** One entry of the fixture table, with its row total over all columns. */
export function entryFor(direction: number, r: number, c: number, tiles: TileView[] = TILES): EntryView {
  const e = rawEntry(tiles, direction, r, c);
  let rowTotal = 0;
  for (let k = 0; k < tiles.length; k++) rowTotal += rawEntry(tiles, direction, r, k).weight;
  return { ...e, rowTotal, probability: rowTotal > 0 ? e.weight / rowTotal : 0 };
}

/**
 * What Rust's `table_view` answers for this fixture. Filters by kind and
 * tile type only; a tag or side filter throws, so a test can't silently get
 * an unfiltered answer.
 */
export function fixtureTableView(
  request: TableViewRequest,
  revision = 1,
  tiles: TileView[] = TILES,
  openingPressure = 1,
): TableView {
  for (const f of [request.rowFilter, request.columnFilter]) {
    if (f.tags.length > 0 || f.sides.length > 0) throw new Error("the fixture doesn't filter by tag or side");
  }
  const pick = (f: TableViewRequest["rowFilter"]) =>
    tiles
      .map((t, i) => ({ t, i }))
      .filter(({ t }) => (f.kinds.length === 0 || f.kinds.includes(t.kind)) && (f.tileIds.length === 0 || f.tileIds.includes(t.id)))
      .map(({ i }) => i);
  const allRows = pick(request.rowFilter);
  const allColumns = pick(request.columnFilter);
  const rows = allRows.slice(request.rows.start, request.rows.start + request.rows.count);
  const columns = allColumns.slice(request.columns.start, request.columns.start + request.columns.count);
  const directions = DIRECTIONS.filter((d) => request.directions.length === 0 || request.directions.includes(d));
  const entries: EntryView[] = [];
  for (const d of directions) {
    for (const r of rows) for (const c of columns) entries.push(entryFor(DIRECTIONS.indexOf(d), r, c, tiles));
  }
  return {
    revision,
    tileSetPath: "data/tile-sets/core.toml",
    loadedAt: "2026-11-01T09:00:00Z",
    tileSetName: "core",
    openingPressure: 1,
    noise: null,
    tiles,
    totalRows: allRows.length,
    totalColumns: allColumns.length,
    rows,
    columns,
    directions,
    exceptions: [STRAIGHTS],
    entries,
    openingPressure,
  };
}

/**
 * What Rust's `explore_tile` answers (DR-0034 rules 1 to 3): the centre's
 * row in each direction, weight above 0 sorted by probability with ties in
 * table order, then the 0s in table order. An unknown pair is `bad_request`.
 */
export function fixtureExplore(request: ExploreRequest, revision = 1, tiles: TileView[] = TILES): ExploreView {
  const centre = tiles.findIndex((t) => t.id === request.tileId && t.orientation === request.orientation);
  if (centre < 0) {
    const error: FixtureError = { kind: "bad_request", messages: [`No tile ${request.tileId} at orientation ${request.orientation}.`] };
    throw error;
  }
  const columns = tiles.map((_, i) => i);
  const arms = DIRECTIONS.map((direction, d) => {
    const entries = columns.map((c) => entryFor(d, centre, c, tiles));
    return {
      direction,
      centreSide: tiles[centre].sides[d],
      neighbours: entries.filter((e) => e.weight > 0).sort((a, b) => b.probability - a.probability),
      zeros: entries.filter((e) => !(e.weight > 0)),
    };
  });
  return { revision, centre, arms: arms as unknown as ExploreView["arms"] };
}

const NORTH = 0;
const WEST = 3;

/**
 * A trace that fits the fixture: ┐ drawn 41st of 384, with ░ to its north
 * and ─ to its west counted, └ to its south came later, and the maze's edge
 * to the east. Only ┐ and ─ have a hedge north and a passage west, so they
 * are the candidates: ┐ scores 1.2 × 1 × 1, ─ 1 × 1 × 0.5 (straight beside
 * straight), so ┐ is 70.6% and ─ 29.4%; the other six were ruled out.
 */
export function fixtureTrace(request: TraceCellRequest): CellTrace {
  const factor = (direction: number, candidate: number, neighbour: number): PairFactor => {
    const entry = entryFor(direction, candidate, neighbour);
    const multiplier = entry.exceptions.length > 0 ? 0.5 : 1;
    return {
      direction: DIRECTIONS[direction],
      neighbour,
      base: entry.base,
      exceptions: entry.exceptions,
      multiplier,
      noise: entry.noise,
      factor: entry.base * multiplier * entry.noise,
      entry,
    };
  };
  const candidate = (tile: number, score: number) => ({
    tile,
    tileWeight: TILES[tile].weight,
    factors: [factor(NORTH, tile, 7), factor(WEST, tile, 0)],
    pressure: 1,
    score,
    share: score / 1.7,
  });
  return {
    revision: request.revision,
    cell: request.cell,
    restarts: 0,
    failed: false,
    steps: 384,
    collapse: {
      step: 41,
      neighbours: [
        { direction: "north", state: "counted", tile: 7 },
        { direction: "east", state: "edge", tile: null },
        { direction: "south", state: "later", tile: 1 },
        { direction: "west", state: "counted", tile: 0 },
      ],
      candidates: [candidate(2, 1.2), candidate(0, 0.5)],
      ruledOut: 6,
      total: 1.7,
      chosen: 2,
    },
  };
}

export interface FixtureBackend {
  /** Every request, by command name. */
  calls: { cmd: string; request?: unknown }[];
  /** When set, `table_view` and `explore_tile` reject with it. */
  fails?: FixtureError;
  /** Replaces the tile list, as if the file had been edited and reloaded. */
  tiles: TileView[];
  revision: number;
  /** What `trace_cell` answers. */
  trace: (request: TraceCellRequest) => CellTrace;
  /** What `generate_maze` answers: by default ─┐, one row of two cells. */
  maze: (request: GenerateRequest) => GeneratedMaze;
  /** The loaded `opening_pressure` (DR-0038), 1 when the key is absent. */
  openingPressure: number;
  /**
   * What `edit_numbers` does. By default it applies tile weight and
   * pressure edits, bumps the revision and records each edit; a test can
   * make it throw a `refused` or `stale` error instead.
   */
  edit: (request: EditRequest) => EditResult;
  /** What `compare_mazes` answers. By default `before` is null. */
  compare: (request: CompareRequest) => CompareResult;
}

/** A mockIPC backend for the whole tool over the fixture (DR-0020). */
export function fixtureBackend(): FixtureBackend {
  const state: FixtureBackend = {
    calls: [],
    tiles: TILES,
    revision: 1,
    trace: fixtureTrace,
    maze: (request) => ({
      revision: state.revision,
      seed: request.seed,
      width: 2,
      height: 1,
      cells: [0, 2],
      centre: { x: 0, y: 0 },
      restarts: 0,
      failed: false,
      measures: [],
    }),
    openingPressure: 1,
    edit: (request) => {
      if (request.revision !== state.revision) {
        const error: FixtureError = { kind: "stale", messages: ["The tile set was reloaded since. Press Reload tile set."] };
        throw error;
      }
      const edits = request.edits.map(({ target, value }) => {
        let from: number | null = null;
        if (target.kind === "tileWeight") {
          from = state.tiles.find((t) => t.id === target.tileId)?.weight ?? null;
          state.tiles = state.tiles.map((t) => (t.id === target.tileId ? { ...t, weight: value } : t));
        } else if (target.kind === "openingPressure") {
          from = state.openingPressure;
          state.openingPressure = value;
        }
        return { at: "2026-11-01T20:31:00Z", target, number: "fixture", from, to: value };
      });
      state.revision++;
      return {
        reload: { revision: state.revision, tileSetPath: "data/tile-sets/core.toml", loadedAt: "2026-11-01T20:31:00Z" },
        edits,
        warnings: [],
      };
    },
    compare: (request) => ({ before: null, after: state.maze(request), changed: [], edits: [] }),
  };
  mockIPC((cmd, payload) => {
    const request = (payload as { request?: unknown } | undefined)?.request;
    state.calls.push({ cmd, request });
    switch (cmd) {
      case COMMANDS.coreVersion:
        return "9.9.9";
      case COMMANDS.tableView:
        if (state.fails) throw state.fails;
        return fixtureTableView(request as TableViewRequest, state.revision, state.tiles, state.openingPressure);
      case COMMANDS.exploreTile:
        if (state.fails) throw state.fails;
        return fixtureExplore(request as ExploreRequest, state.revision, state.tiles);
      case COMMANDS.traceCell: {
        const r = request as TraceCellRequest;
        if (r.revision !== state.revision) {
          const error: FixtureError = { kind: "stale", messages: ["The maze's tile set has been reloaded since."] };
          throw error;
        }
        return state.trace(r);
      }
      case COMMANDS.generateMaze:
        return state.maze(request as GenerateRequest);
      case COMMANDS.editNumbers:
        return state.edit(request as EditRequest);
      case COMMANDS.compareMazes:
        return state.compare(request as CompareRequest);
      case COMMANDS.reloadTileSet:
        state.revision++;
        return { revision: state.revision, tileSetPath: "data/tile-sets/core.toml", loadedAt: "2026-11-01T09:30:00Z" };
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return state;
}
