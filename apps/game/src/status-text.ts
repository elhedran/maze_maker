// The status line's words (DR-0045 point 1). Pure, tested without a DOM.
// TypeScript only formats the position; Rust moves the player.

import type { Pos } from "./ipc.ts";

/**
 * "3 east, 5 north of the start", "2 west of the start", or "at the start".
 * World y grows to the south, as rows do in the tool.
 */
export function positionText(start: Pos, at: Pos): string {
  const east = at.x - start.x;
  const south = at.y - start.y;
  const parts: string[] = [];
  if (east !== 0) parts.push(`${Math.abs(east)} ${east > 0 ? "east" : "west"}`);
  if (south !== 0) parts.push(`${Math.abs(south)} ${south > 0 ? "south" : "north"}`);
  return parts.length ? `${parts.join(", ")} of the start` : "at the start";
}

/** "1 step", "42 steps". */
export function stepsText(steps: number): string {
  return `${steps} ${steps === 1 ? "step" : "steps"}`;
}

export function statusText(seed: number, start: Pos, at: Pos, steps: number): string {
  return `Seed ${seed} · ${positionText(start, at)} · ${stepsText(steps)}`;
}
