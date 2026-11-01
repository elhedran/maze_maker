// Turns key presses and releases into steps at the game's own pace
// (DR-0045 point 8). No DOM: the component feeds it keydown and keyup.

import type { Direction } from "./ipc.ts";

/** How long a key is held before it starts walking on its own. A setting. */
export const HOLD_DELAY_MS = 250;
/** Time between steps while a key is held, about 8 a second. A setting. */
export const HOLD_PACE_MS = 125;

/** What one step did. "failed" means the command failed; walking stops. */
export type StepOutcome = "moved" | "blocked" | "failed";

/**
 * One press is one step, sent at once. Holding walks every HOLD_PACE_MS
 * after HOLD_DELAY_MS, ignoring the OS's key repeat. The newest held key
 * wins. Only one step is in flight: a press made meanwhile waits in a single
 * slot (newest wins), and a hold tick that finds a step in flight is skipped,
 * so nothing queues up and letting go stops at once. Holding into a hedge
 * bumps once, then waits for a release or a new key.
 */
export class Walker {
  /** Keys held down, oldest first. */
  private held: Direction[] = [];
  private busy = false;
  private waiting?: Direction;
  /** The held direction that last bumped; hold ticks skip it. */
  private bumped?: Direction;
  private delayTimer?: ReturnType<typeof setTimeout>;
  private paceTimer?: ReturnType<typeof setInterval>;

  constructor(private readonly step: (direction: Direction) => Promise<StepOutcome>) {}

  /** A keydown that isn't an OS repeat. */
  press(direction: Direction): void {
    this.held = this.held.filter((d) => d !== direction);
    this.held.push(direction);
    this.bumped = undefined;
    this.request(direction);
    this.stopTimers();
    this.delayTimer = setTimeout(() => {
      this.paceTimer = setInterval(() => this.tick(), HOLD_PACE_MS);
    }, HOLD_DELAY_MS);
  }

  release(direction: Direction): void {
    this.held = this.held.filter((d) => d !== direction);
    if (this.bumped === direction) this.bumped = undefined;
    if (this.held.length === 0) this.stopTimers();
  }

  /**
   * Forget every held key and any waiting press: on window blur, where
   * keyups never arrive, and when a new world starts.
   */
  releaseAll(): void {
    this.held = [];
    this.waiting = undefined;
    this.bumped = undefined;
    this.stopTimers();
  }

  private tick() {
    const direction = this.held[this.held.length - 1];
    if (!direction || this.busy || direction === this.bumped) return;
    void this.run(direction);
  }

  private request(direction: Direction) {
    if (this.busy) this.waiting = direction;
    else void this.run(direction);
  }

  private async run(direction: Direction): Promise<void> {
    this.busy = true;
    let outcome: StepOutcome;
    try {
      outcome = await this.step(direction);
    } catch {
      outcome = "failed";
    }
    this.busy = false;
    if (outcome === "failed") {
      this.releaseAll();
      return;
    }
    if (outcome === "blocked") this.bumped = direction;
    const next = this.waiting;
    this.waiting = undefined;
    if (next) void this.run(next);
  }

  private stopTimers() {
    clearTimeout(this.delayTimer);
    clearInterval(this.paceTimer);
    this.delayTimer = this.paceTimer = undefined;
  }
}
