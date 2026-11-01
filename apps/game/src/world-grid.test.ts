import { describe, expect, test } from "vitest";
import { cellLabel, layoutGrid, tileName } from "./world-grid.ts";
import type { GameView, ViewCell } from "./ipc.ts";
import { cell } from "./test-cells.ts";


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
    expect(rows.map((r) => r.map((c) => c.glyph + c.filler).join(""))).toEqual(["░░░░░░", "──@─══", "      "]);
    expect(rows[0][0]).toMatchObject({ x: 9, y: -5 });
    expect(rows[2][2]).toMatchObject({ x: 11, y: -3 });
  });

  test("ungrown cells are blank and carry no tile", () => {
    const south = layoutGrid(VIEW)[2];
    expect(south.every((c) => c.cell === undefined && c.glyph + c.filler === "  ")).toBe(true);
  });

  test("the player keeps the cell's tint and east filler", () => {
    const rows = layoutGrid({ ...VIEW, player: { x: 11, y: -4 } });
    expect(rows[1][1]).toMatchObject({ glyph: "@", filler: "═", tint: "lower", player: true });
  });

  test("a room keeps its floor tint", () => {
    expect(layoutGrid(VIEW)[1][2].tint).toBe("lower");
  });

  test("the player shows even on an ungrown cell", () => {
    const rows = layoutGrid({ ...VIEW, cells: [] });
    expect(rows[1][1].glyph + rows[1][1].filler).toBe("@ ");
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

describe("sealed unknown pockets (Wren's G1)", () => {
  // A radius-2 view (5 × 5) around (0, 0), drawn as rows north to south:
  //   ┌─┬─┐      row -2
  //   │ ? │      row -1: (0,-1) unknown, walled in by hedge sides
  //   ├─@─┤      row  0
  //   │ ? │ ?    row  1: (0,1) unknown, hedge all round; (2,1) unknown on the edge
  //   └─┴─? ?    row  2: (2,2) unknown on the edge
  const C = (x: number, y: number, tile: string, o: number) => cell(x, y, tile, o, "corridor");
  const sealedView = (cells: ViewCell[]): GameView => ({ player: { x: 0, y: 0 }, moved: true, radius: 2, cells });
  const FRAME: ViewCell[] = [
    C(-2, -2, "corridor_corner", 1), // ┌
    C(-1, -2, "corridor_straight", 1), // ─
    C(0, -2, "corridor_t_intersection", 2), // ┬ opens S onto (0,-1)
    C(1, -2, "corridor_straight", 1),
    C(2, -2, "corridor_corner", 2), // ┐
    C(-2, -1, "corridor_straight", 0), // │
    C(-1, -1, "corridor_straight", 0), // │ closed E
    C(1, -1, "corridor_straight", 0), // │ closed W
    C(2, -1, "corridor_straight", 0),
    C(-2, 0, "corridor_t_intersection", 1), // ├
    C(-1, 0, "corridor_straight", 1), // ─ closed N
    C(0, 0, "corridor_straight", 1), // player, ─ closed N and S
    C(1, 0, "corridor_straight", 1),
    C(2, 0, "corridor_t_intersection", 3), // ┤
    C(-2, 1, "corridor_straight", 0),
    C(-1, 1, "corridor_straight", 0), // │ closed E
    C(1, 1, "corridor_straight", 0), // │ closed W
    C(-2, 2, "corridor_corner", 0), // └
    C(-1, 2, "corridor_straight", 1), // ─ closed N
    C(0, 2, "corridor_straight", 1), // ─ closed N
    C(1, 2, "corridor_straight", 1),
  ];
  const at = (rows: ReturnType<typeof layoutGrid>, x: number, y: number) => rows.flat().find((c) => c.x === x && c.y === y)!;

  test("an inner pocket that a stored cell opens onto stays haze", () => {
    const rows = layoutGrid(sealedView(FRAME));
    // ┬ at (0,-2) shows a passage south into (0,-1).
    expect(at(rows, 0, -1)).toMatchObject({ sealed: false, glyph: " " });
  });

  test("an inner pocket that every neighbour faces with hedge is drawn as hedge", () => {
    const closed = FRAME.map((c) => (c.x === 0 && c.y === -2 ? C(0, -2, "corridor_straight", 1) : c));
    const rows = layoutGrid(sealedView(closed));
    const pocket = at(rows, 0, -1);
    expect(pocket).toMatchObject({ sealed: true, glyph: "░", filler: "░" });
    expect(cellLabel(pocket)).toBe("Hedge");
  });

  test("a pocket that touches the view's edge stays haze, even with hedge all round inside", () => {
    // Close the two cells that opened onto the east pocket: ┤ becomes ┘, ─ becomes ┘.
    const edged = FRAME.map((c) =>
      (c.x === 2 && c.y === 0) || (c.x === 1 && c.y === 2) ? C(c.x, c.y, "corridor_corner", 3) : c,
    );
    const rows = layoutGrid(sealedView(edged));
    for (const [x, y] of [
      [2, 1],
      [2, 2],
    ]) {
      expect(at(rows, x, y)).toMatchObject({ sealed: false, glyph: " " });
    }
    expect(cellLabel(at(rows, 2, 1))).toBe("Not yet known");
    // (0,1) is its own pocket, hedge on all four sides and inside: sealed.
    expect(at(rows, 0, 1).sealed).toBe(true);
  });
});
