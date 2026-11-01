// Wording for the "Why it's here" band (DR-0033 points 9 to 16). Pure, so
// it is tested without a DOM. Numbers come from Rust; this only formats them.

import { formatNumber, formatShare } from "../table/format.ts";
import type { NeighbourState } from "../bindings/NeighbourState.ts";

/** 1st, 2nd, 3rd, 4th, 11th, 12th, 13th, 21st, 41st. */
export function ordinal(n: number): string {
  const tens = n % 100;
  if (tens >= 11 && tens <= 13) return `${n}th`;
  const suffix = { 1: "st", 2: "nd", 3: "rd" }[n % 10] ?? "th";
  return `${n}${suffix}`;
}

/**
 * Weights, factors and scores: DR-0025's two decimals, but small values
 * keep two significant figures so a fallback's 0.0025 doesn't read "0.00".
 */
export function formatValue(value: number): string {
  if (value === 0) return "0";
  if (Math.abs(value) >= 0.1) return formatNumber(value);
  return String(Number(value.toPrecision(2)));
}

/** Chance in the draw: one decimal and a % sign, "<0.1%" below 0.05% (DR-0033 point 11). */
export function chanceText(share: number): string {
  return share > 0 ? `${formatShare(share)}%` : "0%";
}

/** "Why ═ is here · seed 48213 · col 14, row 2 · drawn 41st of 384" (point 9; columns and rows from 0, as DR-0029). */
export function headingText(glyph: string, seed: number, x: number, y: number, step: number, steps: number): string {
  return `Why ${glyph} is here · seed ${seed} · col ${x}, row ${y} · drawn ${ordinal(step)} of ${steps}`;
}

/** Point 10's words for each neighbour. */
export const STATE_WORD: Record<NeighbourState, string> = {
  counted: "counted",
  later: "came later",
  edge: "edge",
};

/** Point 15. */
export const NOTHING_SETTLED = "Nothing around it was settled yet, so only tile weights counted.";
/** Point 16. */
export const NO_CHOICE = "Only one tile could fit here, so the draw had no choice.";
/** Point 20, and DR-0034's `stale`. */
export const RELOADED = "The tile set was reloaded. Pick the cell again in the Maze tab.";
/** Point 18. */
export const NOT_PLACED = "Nothing was placed here: the attempt stopped first.";

/** "+ 27 ruled out before the draw" (point 12; the list itself is out of 2.2). */
export function ruledOutText(count: number): string {
  return `+ ${count} ruled out before the draw`;
}

/** The formula line under the candidates (point 12), with the sum of the scores. */
export function formulaText(total: number, glyph: string, anyLater: boolean): string {
  const lines = [
    "score = tile weight × one pair factor per counted neighbour (side-pair base × exceptions × noise; tile weights left out).",
    `chance = score ÷ ${formatValue(total)}, the sum of the scores.`,
  ];
  if (anyLater) lines.push(`Neighbours that came later were drawn to fit ${glyph}, so they don't count here.`);
  return lines.join(" ");
}
