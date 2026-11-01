import { describe, expect, test } from "vitest";
import { positionText, statusText, stepsText } from "./status-text.ts";

const START = { x: 0, y: 0 };

describe("positionText", () => {
  test("says the start at the start", () => {
    expect(positionText(START, START)).toBe("at the start");
  });

  test("gives east or west first, then north or south, with y growing south", () => {
    expect(positionText(START, { x: 3, y: -5 })).toBe("3 east, 5 north of the start");
    expect(positionText(START, { x: -2, y: 0 })).toBe("2 west of the start");
    expect(positionText(START, { x: 0, y: 4 })).toBe("4 south of the start");
  });

  test("counts from wherever the start is", () => {
    expect(positionText({ x: 10, y: 10 }, { x: 11, y: 9 })).toBe("1 east, 1 north of the start");
  });
});

describe("statusText", () => {
  test("seed, position and steps", () => {
    expect(statusText(48213, START, { x: 3, y: -5 }, 42)).toBe("Seed 48213 · 3 east, 5 north of the start · 42 steps");
    expect(stepsText(1)).toBe("1 step");
    expect(stepsText(0)).toBe("0 steps");
  });
});
