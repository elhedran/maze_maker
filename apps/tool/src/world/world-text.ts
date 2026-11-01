// The world view's wording (DR-0039). Pure functions, tested without a DOM.

import type { Direction } from "../bindings/Direction.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { PatchDelta, ReplayCheck } from "./world-types.ts";

/** Seeded walks are capped per world by Rust (DR-0041 rule 9); Steps can't ask for more. */
export const MAX_STEPS = 1_000_000;
export const DEFAULT_STEPS = 200;

/** DR-0039 rule 3's speeds, in milliseconds per tick. */
export const SPEEDS = { slow: 400, normal: 100, fast: 25 } as const;
export type Speed = keyof typeof SPEEDS;

/** "1,912": thousands separated, whatever the machine's locale. */
export function count(n: number): string {
  return n.toLocaleString("en-US");
}

/** The Steps field as a number, or why it isn't one. */
export function parseSteps(text: string): { steps: number } | { problem: string } {
  const trimmed = text.trim();
  const steps = Number(trimmed);
  if (!/^\d+$/.test(trimmed) || steps < 1 || steps > MAX_STEPS) {
    return { problem: `Steps is a whole number from 1 to ${count(MAX_STEPS)}.` };
  }
  return { steps };
}

const ARROWS: Record<Direction, string> = { north: "↑", east: "→", south: "↓", west: "←" };
const LETTERS: Record<Direction, string> = { north: "N", east: "E", south: "S", west: "W" };
const SIDE_INDEX: Record<Direction, number> = { north: 0, east: 1, south: 2, west: 3 };

/** An outward endpoint's mark: an arrow pointing away from the stored cell (DR-0039 rule 7). */
export function endpointMark(facing: Direction, outward: boolean): string {
  return outward ? ARROWS[facing] : "○";
}

/** "x 12, y -4": world coordinates, y growing south (DR-0039 rule 5). */
export function positionText(x: number, y: number): string {
  return `x ${x}, y ${y}`;
}

/** "112 steps · 37 patches · 1,912 cells": a world's totals. */
function totalsText(steps: number, patches: number, cells: number): string {
  return `${count(steps)} ${steps === 1 ? "step" : "steps"} · ${count(patches)} ${
    patches === 1 ? "patch" : "patches"
  } · ${count(cells)} cells`;
}

/** "World 48213 · 112 steps · 37 patches · 1,912 cells" (DR-0039 rule 4). */
export function worldStatus(seed: number, steps: number, patches: number, cells: number): string {
  return `World ${seed} · ${totalsText(steps, patches, cells)}`;
}

/**
 * The status line during replay, until Rust's check arrives. The totals are the
 * walk's, said so, so they don't read as the replay's own count; that is the
 * timeline's "Replay · patch k of n" (Damson, 2.6 step 3 review, fix 4).
 */
export function replayStatus(seed: number, steps: number, patches: number, cells: number): string {
  return `Replaying world ${seed} · walked: ${totalsText(steps, patches, cells)}`;
}

/** DR-0039 rule 12, after a Reload: the world stays on the set it was grown with. */
export function reloadNote(grownLoadTime: string): string {
  return `grown with the tile set loaded ${grownLoadTime} · New world uses the reloaded one`;
}

/** The hover readout and each cell's aria-label (DR-0039 rule 10). */
export function storedCellText(tileLabel: string, x: number, y: number, patchIndex: number): string {
  return `${tileLabel} · ${positionText(x, y)} · stored by patch ${patchIndex + 1}`;
}

export function unsettledCellText(x: number, y: number, endpoint?: { outward: boolean }): string {
  const base = `Unsettled · ${positionText(x, y)}`;
  if (!endpoint) return base;
  return `${base} · ${endpoint.outward ? "open endpoint, outward" : "open endpoint"}`;
}

/**
 * Why a key step didn't move (DR-0039 rule 3): "Hedge to the north" when the
 * player's tile has hedge that way, else the way is open but nothing is stored there.
 */
export function blockedText(tile: TileView | undefined, direction: Direction): string {
  const side = tile?.sides[SIDE_INDEX[direction]];
  if (!tile || side === "hedge") return `Hedge to the ${direction}`;
  return `Nothing grown to the ${direction} yet`;
}

/** The side panel's lines (DR-0039 rule 9, cut to words by Marram). `debug` is the D overlay. */
export function patchPanel(patch: PatchDelta, of: number, debug: boolean): string[] {
  const lines = [`PATCH ${patch.index + 1} of ${of}`];
  if (debug) {
    const w = patch.window;
    lines.push(
      `anchor ${positionText(patch.anchor.x, patch.anchor.y)}, ${patch.from ? `grown ${LETTERS[patch.from]}` : "origin patch"}`,
      `window x ${w.x}..${w.x + w.width - 1}, y ${w.y}..${w.y + w.height - 1}`,
    );
  }
  lines.push(`attempts ${patch.attempts} (1 = clean run)`);
  if (debug) {
    lines.push(`spine: ${patch.spine ? "yes" : "no"}   fallback: ${patch.fallback ? "yes" : "no"}`, `stored ${count(patch.cells.length)}`);
  }
  return lines;
}

export function frontierText(open: number, outward: number): string {
  return `Frontier: ${count(open)} open, ${count(outward)} outward`;
}

/** "Replay · patch 12 of 37" (DR-0039 rule 11). */
export function replayProgress(shown: number, total: number): string {
  return `Replay · patch ${shown} of ${total}`;
}

/** Rust's same-world answer, in words (DR-0039 rule 11). */
export function checkText(replayed: number, check: ReplayCheck): string {
  const head = `Replayed ${count(replayed)} ${replayed === 1 ? "patch" : "patches"} · same world as walked: `;
  if (check.same) return `${head}yes (${count(check.cells)} cells match)`;
  const at = check.firstDifference;
  return `${head}no${at ? ` · first difference at ${positionText(at.x, at.y)}` : ""}`;
}
