// Text and drawing rules for the generation view (DR-0029). Pure functions,
// so the wording and the two-character cells are tested without a DOM.

import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";

/** The first size (DR-0028 rule 7). A tool setting, not a core rule. */
export const MAZE_WIDTH = 24;
export const MAZE_HEIGHT = 16;

/** Seeds are Rust `u32`s (DR-0028 rule 5). */
export const MAX_SEED = 0xffff_ffff;

export function randomSeed(): number {
  return Math.floor(Math.random() * (MAX_SEED + 1));
}

/** The seed in the field, or the text explaining why it isn't one (DR-0029 point 4). */
export function parseSeed(text: string): { seed: number } | { problem: string } {
  const trimmed = text.trim();
  if (!/^\d+$/.test(trimmed)) return { problem: "A seed is a whole number." };
  const seed = Number(trimmed);
  if (seed > MAX_SEED) return { problem: `A seed is a whole number up to ${MAX_SEED}.` };
  return { seed };
}

/** "first try", "restarted 1 time", "restarted 2 times". */
export function restartText(restarts: number): string {
  if (restarts === 0) return "first try";
  return `restarted ${restarts} ${restarts === 1 ? "time" : "times"}`;
}

/**
 * The status line above the maze, about the maze on screen (DR-0029 point 6):
 * "Seed 48213 · 24×16 · restarted 2 times". After a reload it also says
 * "regenerated after reload" (point 3).
 */
export function statusText(maze: GeneratedMaze, afterReload: boolean): string {
  const parts = [`Seed ${maze.seed}`];
  if (afterReload) parts.push("regenerated after reload");
  parts.push(`${maze.width}×${maze.height}`, restartText(maze.restarts));
  return parts.join(" · ");
}

/** DR-0029 point 11, "Gave up". */
export function failureText(maze: GeneratedMaze): string {
  return (
    `No maze for seed ${maze.seed}: gave up after ${maze.restarts} restarts. ` +
    "Try another seed, or look in the table for tiles that can't meet."
  );
}

// Glyphs whose east arm is a single or a double line (DR-0029 point 7).
const SINGLE_EAST = new Set([..."─└┌├┴┬┼╶╟"]);
const DOUBLE_EAST = new Set([..."═╔╚╧╤"]);

/**
 * The second character of a cell, carrying the glyph's east side so cells
 * come out about square and lines join across them (DR-0029 point 7).
 */
export function eastFiller(glyph: string): string {
  if (SINGLE_EAST.has(glyph)) return "─";
  if (DOUBLE_EAST.has(glyph)) return "═";
  if (glyph === "░") return "░";
  return " ";
}

/** The hover readout: "Corridor corner, open N and E · col 5, row 2" (0-based, as DR-0030 counts). */
export function hoverText(label: string, x: number, y: number): string {
  return `${label} · col ${x}, row ${y}`;
}
