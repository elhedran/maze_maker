import { describe, expect, test } from "vitest";
import { MAX_SEED, directionForKey, randomSeed } from "./controls.ts";

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

describe("randomSeed", () => {
  test("is a whole number in u32 range", () => {
    for (let i = 0; i < 100; i++) {
      const seed = randomSeed();
      expect(Number.isInteger(seed) && seed >= 0 && seed <= MAX_SEED).toBe(true);
    }
  });
});
