// Wording for editing a number (DR-0037 points 3 to 7, 14 and 16). Pure, so
// it is tested without a DOM. The frontend checks only that the text is a
// number; ranges and completeness are the loader's (DR-0037 point 5).

import { formatLoadTime } from "../table/format.ts";
import type { ExceptionView } from "../bindings/ExceptionView.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { EditRecord, NumberTarget } from "../pending-bindings.ts";

/** What a label or reach line needs to name a target. */
export interface TileSetNames {
  tiles: TileView[];
  exceptions: ExceptionView[];
}

/** Point 5, under the field. */
export const NOT_A_NUMBER = "A number, like 2.0 or 0.5.";
/** Point 14, beside the pressure. */
export const PRESSURE_MEANING = "Below 1, a tile that adds no new way onward is less likely.";
/** Point 14, under the pressure field. */
export const PRESSURE_HINT = "Between 0 and 1. 1 is off.";

/** The field's text as a number, or undefined. Plain decimals only, signs allowed ("-1" goes to the loader). */
export function parseNumber(text: string): number | undefined {
  const trimmed = text.trim();
  if (!/^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/.test(trimmed)) return undefined;
  const value = Number(trimmed);
  return Number.isFinite(value) ? value : undefined;
}

/**
 * A stored number in full, with at least one decimal: "3.0", "0.125".
 * Used where Ian may type the number back (the field, the saved line), so
 * nothing is rounded away (DR-0037 point 3).
 */
export function exactText(value: number): string {
  return Number.isInteger(value) ? value.toFixed(1) : String(value);
}

/** Point 14: "0.5", or "1.0 (off)" when the pressure is 1 (no key reads as 1). */
export function pressureText(value: number): string {
  return value === 1 ? "1.0 (off)" : exactText(value);
}

// `[bases]` keys as words. Names only; which sides pair is Rust's (EntryView.basePair).
const BASE_WORDS: Record<string, string> = {
  hedge_hedge: "Hedge–Hedge",
  passage_passage: "Passage–Passage",
  passage_door: "Passage–Door",
  door_door: "Door–Door",
  floor_floor: "Floor–Floor",
  floor_wall_floor_wall: "Floor wall–Floor wall",
};

/** The number's name: "Corridor corner weight", "Passage–Door base", "Long straights … multiplier", "Opening pressure". */
export function editLabel(target: NumberTarget, names: TileSetNames): string {
  switch (target.kind) {
    case "tileWeight": {
      const tile = names.tiles.find((t) => t.id === target.tileId);
      return `${tile?.name ?? target.tileId} weight`;
    }
    case "base":
      return `${BASE_WORDS[target.pair] ?? target.pair} base`;
    case "exception": {
      const e = names.exceptions[target.index];
      return e ? `${e.name} multiplier` : `Exception ${target.index + 1} multiplier`;
    }
    case "openingPressure":
      return "Opening pressure";
  }
}

const COUNT_WORDS = ["", "one", "two", "three", "four", "five", "six", "seven", "eight"];

/** Point 4: what a change to this number reaches, in words. */
export function reachText(target: NumberTarget, names: TileSetNames): string {
  switch (target.kind) {
    case "tileWeight": {
      const variants = names.tiles.filter((t) => t.id === target.tileId);
      const name = variants[0]?.name ?? target.tileId;
      const n = variants.length;
      const orientations =
        n <= 1 ? "in its one orientation" : n === 2 ? "in both orientations" : `in all ${COUNT_WORDS[n] ?? n} orientations`;
      return `Every ${name}, ${orientations} and every direction.`;
    }
    case "base":
      return `Every ${BASE_WORDS[target.pair] ?? target.pair} meeting.`;
    case "exception": {
      const e = names.exceptions[target.index];
      return e ? `Every pair this rule matches: ${e.rule}` : "Every pair this rule matches.";
    }
    case "openingPressure":
      return "Every draw where a tile would add no new way onward.";
  }
}

/** Point 16: "Not saved: <the loader's reason, as given>. core.toml is unchanged." */
export function notSavedText(messages: string[]): string {
  const reason = messages.map((m) => m.trim()).filter((m) => m !== "").join(" ") || "no reason was given";
  return `Not saved: ${reason}${/[.!?]$/.test(reason) ? "" : "."} core.toml is unchanged.`;
}

/** "Corridor corner weight 3.0 → 2.0"; a key that wasn't in the file reads "not set". */
export function changeText(record: EditRecord, label: string): string {
  const from = record.from === null ? "not set" : exactText(record.from);
  return `${label} ${from} → ${exactText(record.to)}`;
}

/** Point 7: "Saved 20:31 · Corridor corner weight 3.0 → 2.0". */
export function savedText(record: EditRecord, label: string): string {
  const clock = formatLoadTime(record.at);
  return `Saved ${/^\d\d:\d\d/.test(clock) ? clock.slice(0, 5) : clock} · ${changeText(record, label)}`;
}
