// Wording for the before-and-after compare (DR-0037 points 8 to 13). Pure,
// so it is tested without a DOM. Rust sends the changed cells and both sets
// of measures; nothing here works out a difference (DR-0037 AI guidance).

import { formatLoadTime } from "../table/format.ts";
import { changeText, editLabel, type TileSetNames } from "../edit/edit-text.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";
import type { EditRecord } from "../pending-bindings.ts";

/** Point 11, under the count. */
export const SPREAD = "One different draw changes the draws after it, so changes spread from the first.";
/** Point 12, when hovering the before maze. */
export const BEFORE_FIXED = "Before is the old tile set, so it can't be replayed.";
/** The status line's words in compare. */
export const COMPARE_STATUS = "before and after your last change";

/** Point 9: "BEFORE · as loaded 20:14:03". */
export function beforeLabel(loadedAt: string): string {
  return `BEFORE · as loaded ${formatLoadTime(loadedAt)}`;
}

/**
 * Point 9: "AFTER · Corridor corner weight 3.0 → 2.0" for the tool's saves,
 * or "AFTER · after reload at 20:31:00" when the file was edited by hand.
 */
export function afterLabel(edits: EditRecord[], names: TileSetNames, loadedAt: string): string {
  if (edits.length === 0) return `AFTER · after reload at ${formatLoadTime(loadedAt)}`;
  return `AFTER · ${edits.map((e) => changeText(e, editLabel(e.target, names))).join("; ")}`;
}

/** Point 11: "37 of 384 cells changed." or "Nothing changed for this seed." */
export function changedText(changed: number, total: number): string {
  if (changed === 0) return "Nothing changed for this seed.";
  return `${changed} of ${total} ${total === 1 ? "cell" : "cells"} changed.`;
}

/** One line of point 10's measures table: DR-0030's wording, before → after, "—" for a side that gave up. */
export interface MeasureRow {
  name: string;
  before: string;
  after: string;
}

/** Pairs the two measure sets by name, in the after maze's order. */
export function measureRows(before: GeneratedMaze | undefined, after: GeneratedMaze): MeasureRow[] {
  const value = (maze: GeneratedMaze | undefined, name: string) =>
    !maze || maze.failed ? "—" : (maze.measures.find((m) => m.name === name)?.value ?? "—");
  const names = after.measures.map((m) => m.name);
  for (const m of before?.measures ?? []) if (!names.includes(m.name)) names.push(m.name);
  return names.map((name) => ({ name, before: value(before, name), after: value(after, name) }));
}
