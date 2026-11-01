import { describe, expect, test } from "vitest";
import { cellLabel, layoutGrid, tileName } from "./world-grid.ts";
import type { GameView, ViewCell } from "./ipc.ts";

const cell = (x: number, y: number, tile: string, orientation: number, kind: string, visited = false): ViewCell => ({
  x,
  y,
  tile,
  orientation,
  kind,
  visited,
});

// Radius 1 around (10, -4): a corridor running E–W through the player,
// hedge to the north, nothing grown to the south.
const VIEW: GameView = {
  player: { x: 10, y: -4 },
  moved: true,
  radius: 1,
  cells: [
    cell(9, -5, "hedge", 0, "hedge"),
    cell(10, -5, "hedge", 0, "hedge"),
    cell(11, -5, "hedge", 0, "hedge"),
    cell(9, -4, "corridor_straight", 1, "corridor", true),
    cell(10, -4, "corridor_straight", 1, "corridor", true),
    cell(11, -4, "room_wall", 0, "room"),
  ],
};

describe("layoutGrid", () => {
  test("is a (2r+1) square with the player in the middle", () => {
    const rows = layoutGrid(VIEW);
    expect(rows.length).toBe(3);
    expect(rows.every((r) => r.length === 3)).toBe(true);
    expect(rows[1][1]).toMatchObject({ x: 10, y: -4, player: true });
    expect(rows.flat().filter((c) => c.player).length).toBe(1);
  });

  test("rows run north to south and columns west to east", () => {
    const rows = layoutGrid(VIEW);
    expect(rows.map((r) => r.map((c) => c.text).join(""))).toEqual(["░░░░░░", "──@─══", "      "]);
    expect(rows[0][0]).toMatchObject({ x: 9, y: -5 });
    expect(rows[2][2]).toMatchObject({ x: 11, y: -3 });
  });

  test("ungrown cells are blank and carry no tile", () => {
    const south = layoutGrid(VIEW)[2];
    expect(south.every((c) => c.cell === undefined && c.text === "  ")).toBe(true);
  });

  test("a room keeps its floor tint", () => {
    expect(layoutGrid(VIEW)[1][2].tint).toBe("lower");
  });

  test("the player shows even on an ungrown cell", () => {
    const rows = layoutGrid({ ...VIEW, cells: [] });
    expect(rows[1][1].text).toBe("@ ");
  });
});

describe("labels", () => {
  test("say the tile and the marks", () => {
    const rows = layoutGrid(VIEW);
    expect(cellLabel(rows[1][1])).toBe("Corridor straight, you are here");
    expect(cellLabel(rows[1][0])).toBe("Corridor straight, visited");
    expect(cellLabel(rows[2][0])).toBe("Not yet known");
  });

  test("tileName reads the id", () => {
    expect(tileName("corridor_t_intersection")).toBe("Corridor t intersection");
  });
});
