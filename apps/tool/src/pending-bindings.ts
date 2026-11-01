// Hand-written stand-ins for the ts-rs bindings DR-0038 adds, typed exactly
// as DR-0038's "IPC contract" gives them, so the 2.3 views can be built while
// the Rust side is built in parallel (DR-0019). They live outside
// `bindings/` so the Rust staleness test (DR-0023) stays green.
//
// When the generated bindings land: import these names from `bindings/`
// instead, and delete this file. The four extended types below become the
// generated ones of the same name.

import type { CandidateScore as CandidateScore22 } from "./bindings/CandidateScore.ts";
import type { EntryView as EntryView22 } from "./bindings/EntryView.ts";
import type { GeneratedMaze } from "./bindings/GeneratedMaze.ts";
import type { ReloadResult } from "./bindings/ReloadResult.ts";
import type { TableView as TableView22 } from "./bindings/TableView.ts";
import type { TileView } from "./bindings/TileView.ts";
import type { ToolErrorKind as ToolErrorKind22 } from "./bindings/ToolErrorKind.ts";

export type ToolErrorKind = ToolErrorKind22 | "refused";

export type NumberTarget =
  | { kind: "tileWeight"; tileId: string }
  | { kind: "base"; pair: string }
  | { kind: "exception"; index: number }
  | { kind: "openingPressure" };
export type NumberEdit = { target: NumberTarget; value: number };
export type EditRequest = { revision: number; edits: Array<NumberEdit> };
export type EditRecord = { at: string; target: NumberTarget; number: string; from: number | null; to: number };
export type EditResult = { reload: ReloadResult; edits: Array<EditRecord>; warnings: Array<string> };

export type CompareRequest = { seed: number; width: number; height: number };
export type BeforeMaze = { maze: GeneratedMaze; tiles: Array<TileView>; loadedAt: string };
export type CompareResult = {
  before: BeforeMaze | null;
  after: GeneratedMaze;
  changed: Array<number>;
  edits: Array<EditRecord>;
};

/** DR-0038 rule 11: `score = (tileWeight × Π factor) × pressure`. */
export type CandidateScore = CandidateScore22 & { pressure: number };
/** DR-0038, Ash's addition: the loaded `opening_pressure`, 1 when the key is absent. */
export type TableView = TableView22 & { openingPressure: number };
/** DR-0038, Ash's addition: the `[bases]` key for the pair, `null` when it isn't allowed. */
export type EntryView = EntryView22 & { basePair: string | null };
