// Local copies of DR-0034's IPC types, so the explore view and the "why"
// band build before Rowan's generated bindings land. At the join this file
// goes and the imports switch to `../bindings/`, as 2.1's maze-types.ts did.
// Field for field as DR-0034 writes them (with A1 to A4); don't add to them here.

import type { CellPos } from "../bindings/CellPos.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { SideKind } from "../bindings/SideKind.ts";

/** DR-0034 rule 7 adds `stale`: the maze came from a tile set that has been reloaded since. */
export type ToolErrorKind = "load_failed" | "not_loaded" | "bad_request" | "stale";

export type ExploreRequest = { tileId: string; orientation: number };
/** `centre` and each entry's `row` and `column` index `TableView.tiles` at `revision`. */
export type ExploreView = { revision: number; centre: number; arms: [ArmView, ArmView, ArmView, ArmView] };
/** `neighbours`: weight above 0, highest probability first, ties in table order. `zeros`: the rest, table order. */
export type ArmView = { direction: Direction; centreSide: SideKind; neighbours: Array<EntryView>; zeros: Array<EntryView> };

export type TraceCellRequest = { revision: number; seed: number; width: number; height: number; cell: CellPos };
export type CellTrace = {
  revision: number;
  cell: CellPos;
  restarts: number;
  failed: boolean;
  steps: number;
  collapse: CollapseTrace | null;
};
export type CollapseTrace = {
  step: number;
  neighbours: [NeighbourAtDraw, NeighbourAtDraw, NeighbourAtDraw, NeighbourAtDraw];
  candidates: Array<CandidateScore>;
  ruledOut: number;
  total: number;
  chosen: number;
};
export type NeighbourAtDraw = { direction: Direction; state: NeighbourState; tile: number | null };
export type NeighbourState = "counted" | "later" | "edge";
export type CandidateScore = { tile: number; tileWeight: number; factors: Array<PairFactor>; score: number; share: number };
export type PairFactor = {
  direction: Direction;
  neighbour: number;
  base: number;
  exceptions: Array<number>;
  multiplier: number;
  noise: number;
  factor: number;
  entry: EntryView;
};
