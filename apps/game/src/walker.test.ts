import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { HOLD_DELAY_MS, HOLD_PACE_MS, Walker, type StepOutcome } from "./walker.ts";
import type { Direction } from "./ipc.ts";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

/** A walker whose steps answer at once with `outcome`, recording each direction. */
function walker(outcome: (d: Direction) => StepOutcome = () => "moved") {
  const steps: Direction[] = [];
  const w = new Walker(async (d) => {
    steps.push(d);
    return outcome(d);
  });
  return { w, steps };
}

const wait = (ms: number) => vi.advanceTimersByTimeAsync(ms);

describe("Walker", () => {
  test("one press is one step, at once", async () => {
    const { w, steps } = walker();
    w.press("north");
    w.release("north");
    await wait(1000);
    expect(steps).toEqual(["north"]);
  });

  test("holding walks after the delay, at the game's pace", async () => {
    const { w, steps } = walker();
    w.press("east");
    await wait(HOLD_DELAY_MS - 1);
    expect(steps.length).toBe(1);
    await wait(1 + HOLD_PACE_MS * 4);
    expect(steps.length).toBe(5);
    w.release("east");
    await wait(1000);
    expect(steps.length).toBe(5);
  });

  test("the newest held key wins, and the older one takes over when it's let go", async () => {
    const { w, steps } = walker();
    w.press("east");
    await wait(HOLD_DELAY_MS + HOLD_PACE_MS);
    w.press("north");
    await wait(HOLD_DELAY_MS + HOLD_PACE_MS);
    w.release("north");
    await wait(HOLD_PACE_MS);
    expect(steps).toEqual(["east", "east", "north", "north", "east"]);
    w.release("east");
  });

  test("holding into a hedge bumps once", async () => {
    const { w, steps } = walker((d) => (d === "north" ? "blocked" : "moved"));
    w.press("north");
    await wait(HOLD_DELAY_MS + HOLD_PACE_MS * 5);
    expect(steps).toEqual(["north"]);
    w.press("east");
    await wait(1);
    expect(steps).toEqual(["north", "east"]);
    w.releaseAll();
  });

  test("only one step is in flight; a press meanwhile waits in one slot, newest wins", async () => {
    const steps: Direction[] = [];
    const answers: (() => void)[] = [];
    const w = new Walker(
      (d) =>
        new Promise<StepOutcome>((resolve) => {
          steps.push(d);
          answers.push(() => resolve("moved"));
        }),
    );
    w.press("east");
    w.release("east");
    w.press("north");
    w.release("north");
    w.press("west");
    w.release("west");
    expect(steps).toEqual(["east"]);
    answers.shift()!();
    await wait(0);
    expect(steps).toEqual(["east", "west"]);
    answers.shift()!();
    await wait(1000);
    expect(steps).toEqual(["east", "west"]);
  });

  test("hold ticks skip while a step is in flight, so nothing queues", async () => {
    const steps: Direction[] = [];
    let answer: (() => void) | undefined;
    const w = new Walker(
      (d) =>
        new Promise<StepOutcome>((resolve) => {
          steps.push(d);
          answer = () => resolve("moved");
        }),
    );
    w.press("south");
    await wait(HOLD_DELAY_MS + HOLD_PACE_MS * 10);
    expect(steps.length).toBe(1);
    w.release("south");
    answer!();
    await wait(1000);
    expect(steps.length).toBe(1);
  });

  test("releaseAll stops a held key whose keyup never comes", async () => {
    const { w, steps } = walker();
    w.press("west");
    w.releaseAll();
    await wait(1000);
    expect(steps).toEqual(["west"]);
  });

  test("a failed step stops walking", async () => {
    const { w, steps } = walker(() => "failed");
    w.press("west");
    await wait(1000);
    expect(steps).toEqual(["west"]);
  });
});
