// Key mapping and seeds for the game screen. Pure, tested without a DOM.

import type { Direction } from "./ipc.ts";

/** Arrow keys and WASD (DR-0045 point 8). Matched on `key`, lower-cased so Caps Lock doesn't matter. */
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

/** [New world]'s seed (DR-0045 point 1). */
export function randomSeed(): number {
  return Math.floor(Math.random() * (MAX_SEED + 1));
}
