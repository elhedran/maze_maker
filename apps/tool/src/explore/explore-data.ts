// The explore view's one source of data (DR-0026, DR-0033). Everything the
// view shows comes through `loadExplore` and `loadWhy`, which turn DR-0034's
// index-based responses into tiles the view can draw. The view never sees a
// Tauri command or an index.
//
// DR-0034's responses carry indices into `TableView.tiles` at a revision but
// not the tiles themselves, so this module also fetches the tile list (a
// tiles-only `table_view`) whenever the revision is one it doesn't hold.

import { exploreTile, tableView, tilesOnlyRequest, traceCell } from "../ipc.ts";
import { DIRECTIONS } from "../table/format.ts";
import type { CellPos } from "../bindings/CellPos.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { ExceptionView } from "../bindings/ExceptionView.ts";
import type { SideKind } from "../bindings/SideKind.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { CellTrace } from "../bindings/CellTrace.ts";
import type { ExploreView } from "../bindings/ExploreView.ts";
import type { NeighbourState } from "../bindings/NeighbourState.ts";
import type { ToolErrorKind } from "../bindings/ToolErrorKind.ts";
import type { TraceCellRequest } from "../bindings/TraceCellRequest.ts";

/** A (tile type, orientation): how the tool names a tile across requests and reloads (DR-0017). */
export interface TileRef {
  id: string;
  orientation: number;
}

export const sameTile = (a: TileRef, b: TileRef): boolean => a.id === b.id && a.orientation === b.orientation;

/** The tiles and exceptions that a response's indices point into, at one revision. */
export interface TileList {
  revision: number;
  /** Every (tile, orientation) in table order; also the Centre picker's list. */
  tiles: TileView[];
  exceptions: ExceptionView[];
}

/** One neighbour in an arm: the tile, and the 1.1 table entry for the centre then it. */
export interface ArmNeighbour {
  tile: TileView;
  /**
   * Centre as row tile, this tile as the neighbour in the arm's direction.
   * Share, exception flag and the neighbour's facing side are `probability`,
   * `exceptions` and `facing.column`; the whole entry feeds DR-0025's panel.
   */
  entry: EntryView;
}

/** One side of the centre (DR-0026 points 3 to 6), in the order Rust sorted it. */
export interface Arm {
  direction: Direction;
  /** The centre's side on this edge. */
  centreSide: SideKind;
  /** Weight above 0, highest share first; ties in table order. */
  neighbours: ArmNeighbour[];
  /** The rest, in table order. The view hides them behind "+ N can't go here". */
  zeros: ArmNeighbour[];
}

export interface Centre {
  tile: TileView;
  arms: Record<Direction, Arm>;
}

export interface ExploreData extends TileList {
  /** Absent when no centre was asked for, or the loaded tile set has no such tile. */
  centre?: Centre;
}

/** The maze cell a "why" is about: the maze's seed, size and revision, and the cell (DR-0033 point 7). */
export type MazeCell = TraceCellRequest;

/** A neighbour of the traced cell at the moment it was drawn (DR-0033 point 10). */
export interface WhyNeighbour {
  direction: Direction;
  /** Counted: settled before the draw. Later: settled after. Edge: past the maze's edge. */
  state: NeighbourState;
  /** The tile in the maze on that side; absent at the edge or where a failed maze left it open. */
  tile?: TileView;
}

/** One counted neighbour's pair factor for one candidate (DR-0033 points 11 and 13). */
export interface WhyFactor {
  direction: Direction;
  neighbour: TileView;
  /** Side-pair base × exceptions × noise, as the solver used it. */
  factor: number;
  /** The 1.1 table entry for candidate then neighbour, for DR-0025's panel. */
  entry: EntryView;
}

export interface WhyCandidate {
  tile: TileView;
  weight: number;
  /** One per counted neighbour, N, E, S, W. */
  factors: WhyFactor[];
  score: number;
  /** Chance in the draw: score ÷ the sum of the scores, from Rust. */
  share: number;
  drawn: boolean;
}

/** How one maze cell was drawn, replayed by Rust (DR-0034 rule 6). */
export interface Why {
  revision: number;
  seed: number;
  cell: CellPos;
  /** When the cell was drawn: `step` of `steps`. */
  step: number;
  steps: number;
  /** The tile drawn, which is the maze's cell. */
  tile: TileView;
  neighbours: WhyNeighbour[];
  /** Highest chance first, ties in table order (Rust sorts). */
  candidates: WhyCandidate[];
  ruledOut: number;
  /** The sum of the scores. */
  total: number;
  exceptions: ExceptionView[];
}

/**
 * A trace, or why there isn't one: `stale` when the maze's tile set has been
 * reloaded since (DR-0034 rule 7), `notPlaced` when the attempt stopped
 * before reaching the cell (DR-0033 point 18).
 */
export type WhyResult = { why: Why } | { stale: true } | { notPlaced: true };

/** The kind of a rejected command, if it was a ToolError. */
export function errorKind(error: unknown): ToolErrorKind | undefined {
  const kind = (error as { kind?: unknown } | null)?.kind;
  return typeof kind === "string" ? (kind as ToolErrorKind) : undefined;
}

export async function loadTileList(): Promise<TileList> {
  const view = await tableView(tilesOnlyRequest());
  return { revision: view.revision, tiles: view.tiles, exceptions: view.exceptions };
}

/** `known` if it is at `revision`, else a fresh tile list, which must be. */
async function tileListAt(revision: number, known?: TileList): Promise<TileList> {
  if (known?.revision === revision) return known;
  const list = await loadTileList();
  // Only a reload between the two requests does this, and the app refreshes
  // the view after every reload, so the newer request wins.
  if (list.revision !== revision) throw new Error("The tile set changed while loading. Press Reload tile set.");
  return list;
}

/**
 * The arms around `centre`, or with no centre just the tile list for the
 * picker. A centre the loaded tile set doesn't have comes back without
 * `centre` (DR-0026 point 1). `known` is the tile list the caller already
 * holds, reused if the revision still matches. Other failures reject as the
 * command did (a ToolError).
 */
export async function loadExplore(centre?: TileRef, known?: TileList): Promise<ExploreData> {
  if (!centre) return loadTileList();
  let view: ExploreView;
  try {
    view = await exploreTile({ tileId: centre.id, orientation: centre.orientation });
  } catch (e) {
    // An unknown pair is `bad_request` (DR-0034 rule 2). If the tile set
    // really has no such tile, that's a missing centre, not an error.
    if (errorKind(e) === "bad_request") {
      const list = await loadTileList();
      if (!list.tiles.some((t) => sameTile(t, centre))) return list;
    }
    throw e;
  }
  const list = await tileListAt(view.revision, known);
  return { ...list, centre: centreFrom(view, list) };
}

/** Resolves an `ExploreView`'s indices. Keeps Rust's order; computes nothing. */
export function centreFrom(view: ExploreView, list: TileList): Centre {
  const tile = tileAt(list, view.centre);
  const neighbour = (entry: EntryView): ArmNeighbour => ({ tile: tileAt(list, entry.column), entry });
  const arms = Object.fromEntries(
    view.arms.map((arm) => [
      arm.direction,
      {
        direction: arm.direction,
        centreSide: arm.centreSide,
        neighbours: arm.neighbours.map(neighbour),
        zeros: arm.zeros.map(neighbour),
      },
    ]),
  ) as Record<Direction, Arm>;
  for (const d of DIRECTIONS) if (!arms[d]) throw new Error(`explore_tile sent no ${d} arm`);
  return { tile, arms };
}

/** How `cell` was drawn, replayed from its maze's seed (DR-0033, DR-0034 rules 4 to 7). */
export async function loadWhy(cell: MazeCell, known?: TileList): Promise<WhyResult> {
  let trace: CellTrace;
  try {
    trace = await traceCell(cell);
  } catch (e) {
    if (errorKind(e) === "stale") return { stale: true };
    throw e;
  }
  if (!trace.collapse) return { notPlaced: true };
  const list = await tileListAt(trace.revision, known);
  return { why: whyFrom(trace, cell.seed, list) };
}

/** Resolves a `CellTrace`'s indices. Keeps Rust's order and numbers; computes nothing. */
export function whyFrom(trace: CellTrace, seed: number, list: TileList): Why {
  const collapse = trace.collapse;
  if (!collapse) throw new Error("whyFrom needs a trace with a collapse");
  return {
    revision: trace.revision,
    seed,
    cell: trace.cell,
    step: collapse.step,
    steps: trace.steps,
    tile: tileAt(list, collapse.chosen),
    neighbours: collapse.neighbours.map((n) => ({
      direction: n.direction,
      state: n.state,
      tile: n.tile === null ? undefined : tileAt(list, n.tile),
    })),
    candidates: collapse.candidates.map((c) => ({
      tile: tileAt(list, c.tile),
      weight: c.tileWeight,
      factors: c.factors.map((f) => ({
        direction: f.direction,
        neighbour: tileAt(list, f.neighbour),
        factor: f.factor,
        entry: f.entry,
      })),
      score: c.score,
      share: c.share,
      drawn: c.tile === collapse.chosen,
    })),
    ruledOut: collapse.ruledOut,
    total: collapse.total,
    exceptions: list.exceptions,
  };
}

function tileAt(list: TileList, index: number): TileView {
  const tile = list.tiles[index];
  if (!tile) throw new Error(`Tile ${index} isn't in the tile set at revision ${list.revision}.`);
  return tile;
}
