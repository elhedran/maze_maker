// Input rules for the game screen: which keys move, and what a seed is.
// Pure functions, tested without a DOM.

import type { Direction } from "./ipc.ts";

/** Arrow keys and WASD. Matched on `key`, lower-cased so Caps Lock doesn't matter. */
const KEYS: Record<string, Direction> = {
  arrowup: "north",
  arrowright: "east",
  arrowdown: "south",
  arrowleft: "west",
  w: "north",
  d: "east",
  s: "south",
  a: "west",
};

export function directionForKey(key: string): Direction | undefined {
  return KEYS[key.toLowerCase()];
}

/** Seeds are Rust `u32`s, as in the tool (DR-0028 rule 5). */
export const MAX_SEED = 0xffff_ffff;

/** A fixed default, so the first world on load is the same every time. */
export const DEFAULT_SEED = 1;

/** The seed in the field, or the text explaining why it isn't one. */
export function parseSeed(text: string): { seed: number } | { problem: string } {
  const trimmed = text.trim();
  if (!/^\d+$/.test(trimmed)) return { problem: "A seed is a whole number." };
  const seed = Number(trimmed);
  if (seed > MAX_SEED) return { problem: `A seed is a whole number up to ${MAX_SEED}.` };
  return { seed };
}
