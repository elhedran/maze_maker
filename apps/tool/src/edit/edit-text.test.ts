import { describe, expect, test } from "vitest";
import {
  changeText,
  editLabel,
  exactText,
  notSavedText,
  parseNumber,
  pressureText,
  reachText,
  savedText,
} from "./edit-text.ts";
import { TILES } from "../explore/fixture.ts";

const NAMES = {
  tiles: TILES,
  exceptions: [
    {
      name: "Long straights are rarer than turns",
      multiplier: 0.5,
      rule: "tile corridor_straight (passage) beside tile corridor_straight (passage)",
    },
  ],
};

describe("edit-text", () => {
  test("parses plain numbers, signed or not, and nothing else", () => {
    expect(parseNumber(" 2.0 ")).toBe(2);
    expect(parseNumber(".5")).toBe(0.5);
    expect(parseNumber("-1")).toBe(-1);
    expect(parseNumber("1e-3")).toBe(0.001);
    for (const bad of ["", "abc", "1.2.3", "0x10", "Infinity", "1,5"]) expect(parseNumber(bad)).toBeUndefined();
  });

  test("shows stored numbers in full, never rounded", () => {
    expect(exactText(3)).toBe("3.0");
    expect(exactText(0.125)).toBe("0.125");
    expect(exactText(1.0749)).toBe("1.0749");
  });

  test("opening pressure reads (off) at 1", () => {
    expect(pressureText(1)).toBe("1.0 (off)");
    expect(pressureText(0.5)).toBe("0.5");
  });

  test("names each kind of number", () => {
    expect(editLabel({ kind: "tileWeight", tileId: "corridor_corner" }, NAMES)).toBe("Corridor corner weight");
    expect(editLabel({ kind: "base", pair: "passage_door" }, NAMES)).toBe("Passage–Door base");
    expect(editLabel({ kind: "exception", index: 0 }, NAMES)).toBe("Long straights are rarer than turns multiplier");
    expect(editLabel({ kind: "openingPressure" }, NAMES)).toBe("Opening pressure");
  });

  test("says what each number reaches (DR-0037 point 4)", () => {
    expect(reachText({ kind: "tileWeight", tileId: "corridor_corner" }, NAMES)).toBe(
      "Every Corridor corner, in all three orientations and every direction.",
    );
    expect(reachText({ kind: "tileWeight", tileId: "room_doorway" }, NAMES)).toBe(
      "Every Room doorway, in both orientations and every direction.",
    );
    expect(reachText({ kind: "tileWeight", tileId: "hedge" }, NAMES)).toBe(
      "Every Hedge, in its one orientation and every direction.",
    );
    expect(reachText({ kind: "base", pair: "passage_passage" }, NAMES)).toBe("Every Passage–Passage meeting.");
    expect(reachText({ kind: "exception", index: 0 }, NAMES)).toBe(
      "Every pair this rule matches: tile corridor_straight (passage) beside tile corridor_straight (passage)",
    );
    expect(reachText({ kind: "openingPressure" }, NAMES)).toBe("Every draw where a tile would add no new way onward.");
  });

  test("a refusal gives the loader's reason as given (DR-0037 point 16)", () => {
    expect(notSavedText(["weight is -1; it must be at least 0"])).toBe(
      "Not saved: weight is -1; it must be at least 0. core.toml is unchanged.",
    );
    expect(notSavedText(["First.", "Second."])).toBe("Not saved: First. Second. core.toml is unchanged.");
  });

  test("the saved line has the time, the number and from → to", () => {
    const record = {
      at: "2026-11-03T14:03:22Z",
      target: { kind: "tileWeight" as const, tileId: "corridor_corner" },
      number: "tile.corridor_corner.weight",
      from: 3,
      to: 2,
    };
    expect(changeText(record, "Corridor corner weight")).toBe("Corridor corner weight 3.0 → 2.0");
    expect(savedText(record, "Corridor corner weight")).toMatch(/^Saved \d\d:\d\d · Corridor corner weight 3\.0 → 2\.0$/);
    expect(changeText({ ...record, from: null, to: 0.5 }, "Opening pressure")).toBe("Opening pressure not set → 0.5");
  });
});
