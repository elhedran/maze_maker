import { describe, expect, test } from "vitest";
import { clashLine, fileName, formatNumber, formatShare, shadeStep, sideName } from "./format.ts";
import { tileLabel, tileLook } from "./tile-glyphs.ts";

describe("formatShare", () => {
  test("one decimal, no % sign", () => {
    expect(formatShare(0.15)).toBe("15.0");
    expect(formatShare(0.301)).toBe("30.1");
    expect(formatShare(1)).toBe("100.0");
  });

  test("0 is blank", () => {
    expect(formatShare(0)).toBe("");
  });

  test("a non-zero share below 0.05% shows <0.1, never 0.0", () => {
    expect(formatShare(0.0004)).toBe("<0.1");
    expect(formatShare(1e-9)).toBe("<0.1");
  });
});

describe("shadeStep", () => {
  test("0 has no shade", () => {
    expect(shadeStep(0)).toBe(0);
  });

  test("five fixed steps, a boundary takes the upper step", () => {
    expect(shadeStep(0.001)).toBe(1);
    expect(shadeStep(0.0499)).toBe(1);
    expect(shadeStep(0.05)).toBe(2);
    expect(shadeStep(0.1)).toBe(3);
    expect(shadeStep(0.2)).toBe(4);
    expect(shadeStep(0.4)).toBe(5);
    expect(shadeStep(1)).toBe(5);
  });
});

describe("words", () => {
  test("side names follow DR-0015", () => {
    expect(sideName("passage")).toBe("Passage");
    expect(sideName("floor_wall_n")).toBe("Floor, wall at N end");
  });

  test("the clash line names both sides and what the row's side may meet", () => {
    expect(clashLine({ row: "passage", column: "floor_wall_n" }, ["passage", "door"])).toBe(
      "Passage meets Floor, wall at N end. A passage only meets a passage or a door.",
    );
  });

  test("explanation numbers keep one or two decimals", () => {
    expect(formatNumber(1)).toBe("1.0");
    expect(formatNumber(1.07)).toBe("1.07");
    expect(formatNumber(0.1)).toBe("0.1");
    expect(formatNumber(40)).toBe("40.0");
  });
});

describe("tile glyphs", () => {
  test("orientation k picks DR-0017's glyph", () => {
    expect(tileLook("corridor_corner", 0).glyph).toBe("└");
    expect(tileLook("corridor_corner", 2).glyph).toBe("┐");
    expect(tileLook("room_wall", 2)).toMatchObject({ glyph: "═", tint: "upper" });
  });

  test("an unknown tile shows ? rather than borrowing a glyph", () => {
    expect(tileLook("alcove", 0).glyph).toBe("?");
  });

  test("labels add the orientation, except for one-orientation tiles", () => {
    expect(tileLabel({ id: "corridor_corner", name: "Corridor corner", orientation: 0 })).toBe(
      "Corridor corner, open N and E",
    );
    expect(tileLabel({ id: "hedge", name: "Hedge", orientation: 0 })).toBe("Hedge");
  });
});

describe("fileName", () => {
  test("the last part of a path, with either separator", () => {
    expect(fileName("data/tile-sets/core.toml")).toBe("core.toml");
    expect(fileName("C:\\Users\\ian\\maze_maker\\data\\tile-sets\\core.toml")).toBe("core.toml");
    expect(fileName("core.toml")).toBe("core.toml");
    expect(fileName("")).toBe("");
  });
});
