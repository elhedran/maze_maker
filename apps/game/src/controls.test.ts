import { describe, expect, test } from "vitest";
import { directionForKey, parseSeed } from "./controls.ts";

describe("directionForKey", () => {
  test("arrows and WASD map to the four directions", () => {
    expect(["ArrowUp", "ArrowRight", "ArrowDown", "ArrowLeft"].map(directionForKey)).toEqual([
      "north",
      "east",
      "south",
      "west",
    ]);
    expect(["w", "d", "s", "a"].map(directionForKey)).toEqual(["north", "east", "south", "west"]);
    expect(directionForKey("W")).toBe("north");
  });

  test("other keys don't move", () => {
    expect(directionForKey("q")).toBeUndefined();
    expect(directionForKey("Enter")).toBeUndefined();
  });
});

describe("parseSeed", () => {
  test("takes whole numbers up to u32", () => {
    expect(parseSeed(" 42 ")).toEqual({ seed: 42 });
    expect(parseSeed("4294967295")).toEqual({ seed: 4294967295 });
  });

  test("explains anything else", () => {
    expect(parseSeed("")).toHaveProperty("problem");
    expect(parseSeed("-1")).toHaveProperty("problem");
    expect(parseSeed("1.5")).toHaveProperty("problem");
    expect(parseSeed("4294967296")).toHaveProperty("problem");
  });
});
