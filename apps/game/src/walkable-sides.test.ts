import { describe, expect, test } from "vitest";
import { walkableSides } from "./walkable-sides.ts";
import { tileLook } from "./tile-glyphs.ts";

// Each corridor glyph's arms, N, E, S, W: an independent copy of the same
// tile data (DR-0017), so the two tables check each other.
const ARMS: Record<string, [boolean, boolean, boolean, boolean]> = {
  "│": [true, false, true, false],
  "─": [false, true, false, true],
  "└": [true, true, false, false],
  "┌": [false, true, true, false],
  "┐": [false, false, true, true],
  "┘": [true, false, false, true],
  "┴": [true, true, false, true],
  "├": [true, true, true, false],
  "┬": [false, true, true, true],
  "┤": [true, false, true, true],
  "┼": [true, true, true, true],
  "╵": [true, false, false, false],
  "╶": [false, true, false, false],
  "╷": [false, false, true, false],
  "╴": [false, false, false, true],
};

describe("walkableSides", () => {
  test("matches every corridor glyph's arms", () => {
    const ids = ["corridor_straight", "corridor_corner", "corridor_t_intersection", "corridor_crossroads", "corridor_dead_end"];
    for (const id of ids) {
      for (let o = 0; o < 4; o++) {
        const { glyph } = tileLook(id, o);
        if (glyph === "?") continue; // Orientations a tile doesn't have.
        expect(walkableSides(id, o), `${id} ${o} ${glyph}`).toEqual(ARMS[glyph]);
      }
    }
  });

  test("rooms are closed only on their wall sides", () => {
    expect(walkableSides("room_wall", 0)).toEqual([false, true, true, true]); // ═ wall N
    expect(walkableSides("room_wall", 1)).toEqual([true, false, true, true]); // ║ wall E
    expect(walkableSides("room_corner", 0)).toEqual([false, false, true, true]); // ╗ NE
    expect(walkableSides("room_corner", 3)).toEqual([false, true, true, false]); // ╔ NW, walls N and W
    expect(walkableSides("room_doorway", 2)).toEqual([true, true, true, true]);
  });

  test("hedge is closed and an unknown tile is open everywhere", () => {
    expect(walkableSides("hedge", 0)).toEqual([false, false, false, false]);
    expect(walkableSides("no_such_tile", 0)).toEqual([true, true, true, true]);
  });
});
