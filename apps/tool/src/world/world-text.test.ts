import { describe, expect, test } from "vitest";
import {
  blockedText,
  checkText,
  endpointMark,
  frontierText,
  parseSteps,
  patchPanel,
  reloadNote,
  replayProgress,
  storedCellText,
  unsettledCellText,
  worldStatus,
} from "./world-text.ts";
import { TILES, patch } from "./fixture.ts";

describe("world text", () => {
  test("status line counts steps, patches and cells, with separators", () => {
    expect(worldStatus(48213, 112, 37, 1912)).toBe("World 48213 · 112 steps · 37 patches · 1,912 cells");
    expect(worldStatus(1, 1, 1, 9)).toBe("World 1 · 1 step · 1 patch · 9 cells");
  });

  test("Steps is 1 to 1,000,000", () => {
    expect(parseSteps(" 200 ")).toEqual({ steps: 200 });
    expect(parseSteps("1000000")).toEqual({ steps: 1_000_000 });
    for (const bad of ["0", "-3", "2.5", "", "abc", "1000001"]) {
      expect(parseSteps(bad)).toEqual({ problem: "Steps is a whole number from 1 to 1,000,000." });
    }
  });

  test("outward endpoints are arrows pointing away, others a hollow circle", () => {
    expect(endpointMark("north", true)).toBe("↑");
    expect(endpointMark("east", true)).toBe("→");
    expect(endpointMark("south", true)).toBe("↓");
    expect(endpointMark("west", true)).toBe("←");
    expect(endpointMark("west", false)).toBe("○");
  });

  test("hover text for stored and unsettled cells", () => {
    expect(storedCellText("Corridor corner, open N and E", 12, -4, 20)).toBe(
      "Corridor corner, open N and E · x 12, y -4 · stored by patch 21",
    );
    expect(unsettledCellText(3, 1)).toBe("Unsettled · x 3, y 1");
    expect(unsettledCellText(3, 1, { outward: true })).toBe("Unsettled · x 3, y 1 · open endpoint, outward");
    expect(unsettledCellText(3, 1, { outward: false })).toBe("Unsettled · x 3, y 1 · open endpoint");
  });

  test("a blocked step says hedge when the tile has hedge that way", () => {
    // TILES[0] is a corridor open E and W.
    expect(blockedText(TILES[0], "north")).toBe("Hedge to the north");
    expect(blockedText(TILES[0], "east")).toBe("Nothing grown to the east yet");
    expect(blockedText(undefined, "south")).toBe("Hedge to the south");
  });

  test("the patch panel is short with D off and in words with D on", () => {
    const p = { ...patch(36, [[14, -3, 0]]), anchor: { x: 14, y: -3 }, from: "east" as const, attempts: 3, spine: true };
    p.window = { x: 14, y: -11, width: 16, height: 16 };
    expect(patchPanel(p, 37, false)).toEqual(["PATCH 37 of 37", "attempts 3 (1 = clean run)"]);
    expect(patchPanel(p, 37, true)).toEqual([
      "PATCH 37 of 37",
      "anchor x 14, y -3, grown E",
      "window x 14..29, y -11..4",
      "attempts 3 (1 = clean run)",
      "spine: yes   fallback: no",
      "stored 1",
    ]);
    expect(patchPanel(patch(0, []), 1, true)[1]).toBe("anchor x 0, y 0, origin patch");
  });

  test("frontier, replay and reload wording", () => {
    expect(frontierText(14, 9)).toBe("Frontier: 14 open, 9 outward");
    expect(replayProgress(12, 37)).toBe("Replay · patch 12 of 37");
    expect(reloadNote("20:14:03")).toBe("grown with the tile set loaded 20:14:03 · New world uses the reloaded one");
  });

  test("Rust's check, in words", () => {
    expect(checkText(37, { same: true, cells: 1912, firstDifference: null })).toBe(
      "Replayed 37 patches · same world as walked: yes (1,912 cells match)",
    );
    expect(checkText(5, { same: false, cells: 80, firstDifference: { x: 3, y: -2 } })).toBe(
      "Replayed 5 patches · same world as walked: no · first difference at x 3, y -2",
    );
  });
});
