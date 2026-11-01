import { describe, expect, test } from "vitest";
import { eastFiller, failureText, hoverText, parseSeed, randomSeed, restartText, statusText } from "./maze-text.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";

const maze = (overrides: Partial<GeneratedMaze> = {}): GeneratedMaze => ({
  revision: 1,
  seed: 48213,
  width: 24,
  height: 16,
  cells: [],
  centre: { x: 12, y: 8 },
  restarts: 0,
  failed: false,
  measures: [],
  ...overrides,
});

describe("parseSeed", () => {
  test("takes whole numbers from 0 to the largest u32", () => {
    expect(parseSeed("48213")).toEqual({ seed: 48213 });
    expect(parseSeed(" 0 ")).toEqual({ seed: 0 });
    expect(parseSeed("4294967295")).toEqual({ seed: 4294967295 });
  });

  test("anything else explains what a seed is", () => {
    for (const text of ["", "abc", "-1", "1.5", "1e3", "12 34"]) {
      expect(parseSeed(text)).toEqual({ problem: "A seed is a whole number." });
    }
    expect(parseSeed("4294967296")).toEqual({ problem: "A seed is a whole number up to 4294967295." });
  });

  test("a random seed is always a valid one", () => {
    for (let i = 0; i < 100; i++) {
      const seed = randomSeed();
      expect(parseSeed(String(seed))).toEqual({ seed });
    }
  });
});

describe("status and failure text", () => {
  test("says how many restarts it took", () => {
    expect(restartText(0)).toBe("first try");
    expect(restartText(1)).toBe("restarted 1 time");
    expect(restartText(2)).toBe("restarted 2 times");
  });

  test("the status line describes the maze on screen", () => {
    expect(statusText(maze(), false)).toBe("Seed 48213 · 24×16 · first try");
    expect(statusText(maze({ restarts: 2 }), false)).toBe("Seed 48213 · 24×16 · restarted 2 times");
    expect(statusText(maze(), true)).toBe("Seed 48213 · regenerated after reload · 24×16 · first try");
  });

  test("a maze that gave up names the seed and the restarts", () => {
    expect(failureText(maze({ failed: true, restarts: 20 }))).toBe(
      "No maze for seed 48213: gave up after 20 restarts. Try another seed, or look in the table for tiles that can't meet.",
    );
  });

  test("the hover readout names the tile and its column and row", () => {
    expect(hoverText("Corridor corner, open N and E", 5, 2)).toBe("Corridor corner, open N and E · col 5, row 2");
  });
});

describe("eastFiller", () => {
  test("carries a single east arm as a single line", () => {
    for (const glyph of "─└┌├┴┬┼╶╟") expect(eastFiller(glyph)).toBe("─");
  });

  test("carries a double east arm as a double line", () => {
    for (const glyph of "═╔╚╧╤") expect(eastFiller(glyph)).toBe("═");
  });

  test("continues hedge, and leaves everything else open", () => {
    expect(eastFiller("░")).toBe("░");
    for (const glyph of "│┐┘┤╵╷╴·║╗╝╢?") expect(eastFiller(glyph)).toBe(" ");
  });
});
