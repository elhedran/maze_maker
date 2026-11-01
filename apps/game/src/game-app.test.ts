import { describe, expect, test, vi } from "vitest";
import { mockIPC } from "@tauri-apps/api/mocks";
import { COMMANDS, type Direction, type GameView, type ViewCell } from "./ipc.ts";
import { BLOCKED_CUE_MS } from "./game-app.ts";
import type { GameApp } from "./game-app.ts";

// Radius 1 around the origin: corridor E–W, hedge north, nothing grown south.
const cell = (x: number, y: number, tile: string, orientation: number, kind: string, visited = false): ViewCell => ({
  x,
  y,
  tile,
  orientation,
  kind,
  visited,
});
function viewAt(x: number, moved = true): GameView {
  return {
    player: { x, y: 0 },
    moved,
    radius: 1,
    cells: [
      cell(x - 1, -1, "hedge", 0, "hedge"),
      cell(x, -1, "hedge", 0, "hedge"),
      cell(x + 1, -1, "hedge", 0, "hedge"),
      cell(x - 1, 0, "corridor_straight", 1, "corridor", true),
      cell(x, 0, "corridor_straight", 1, "corridor", true),
      cell(x + 1, 0, "corridor_straight", 1, "corridor"),
    ],
  };
}

interface Backend {
  starts: number[];
  steps: Direction[];
  /** What the next step answers; walks east by default. */
  stepAnswer?: (direction: Direction, x: number) => GameView | Promise<GameView>;
  startFails?: string;
}

function backend(): Backend {
  const b: Backend = { starts: [], steps: [] };
  let x = 0;
  mockIPC((cmd, args) => {
    const a = args as Record<string, unknown>;
    if (cmd === COMMANDS.startGame) {
      if (b.startFails) return Promise.reject(b.startFails);
      b.starts.push(a.seed as number);
      x = 0;
      return viewAt(0);
    }
    if (cmd === COMMANDS.step) {
      const direction = a.direction as Direction;
      b.steps.push(direction);
      if (b.stepAnswer) return b.stepAnswer(direction, x);
      x += direction === "east" ? 1 : direction === "west" ? -1 : 0;
      return viewAt(x);
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return b;
}

async function mount(): Promise<GameApp> {
  await import("./game-app.ts");
  const el = document.createElement("game-app");
  document.body.append(el);
  await settle(el);
  return el;
}

/** Lets the mocked IPC promise resolve and Lit re-render. */
async function settle(el: GameApp) {
  for (let i = 0; i < 3; i++) {
    await new Promise((r) => setTimeout(r, 0));
    await el.updateComplete;
  }
}

const q = (el: GameApp, selector: string) => el.shadowRoot!.querySelector<HTMLElement>(selector);
const cells = (el: GameApp) => [...el.shadowRoot!.querySelectorAll<HTMLElement>(".map .cell")];
const press = (key: string, target: EventTarget = window) =>
  target.dispatchEvent(new KeyboardEvent("keydown", { key, bubbles: true, composed: true, cancelable: true }));

describe("game-app", () => {
  test("starts the default world on load and draws its cells", async () => {
    const b = backend();
    const el = await mount();
    expect(b.starts).toEqual([1]);
    const map = cells(el);
    expect(map.length).toBe(9);
    expect(map.map((c) => c.textContent).join("")).toBe("░░░░░░──@───      ");
    expect(map.filter((c) => c.classList.contains("hedge")).length).toBe(3);
    expect(map.filter((c) => c.classList.contains("ungrown")).length).toBe(3);
    expect(map[3].classList.contains("visited")).toBe(true);
    expect(map[5].classList.contains("visited")).toBe(false);
    expect(q(el, "#status")!.textContent).toContain("Seed 1 · x 0, y 0");
  });

  test("draws the player at the centre, as '@' and not by colour alone", async () => {
    backend();
    const el = await mount();
    const map = cells(el);
    const player = map.filter((c) => c.classList.contains("player"));
    expect(player).toEqual([map[4]]);
    expect(player[0].dataset.x).toBe("0");
    expect(player[0].textContent!.startsWith("@")).toBe(true);
    expect(player[0].getAttribute("aria-label")).toContain("you are here");
  });

  test("arrow keys and WASD call step with the right direction", async () => {
    const b = backend();
    const el = await mount();
    for (const key of ["ArrowUp", "ArrowRight", "ArrowDown", "ArrowLeft", "w", "d", "s", "a"]) {
      press(key);
      await settle(el);
    }
    expect(b.steps).toEqual(["north", "east", "south", "west", "north", "east", "south", "west"]);
  });

  test("a step redraws around the new position", async () => {
    backend();
    const el = await mount();
    press("ArrowRight");
    await settle(el);
    expect(q(el, "#status")!.textContent).toContain("x 1, y 0");
    expect(q(el, ".cell.player")!.dataset.x).toBe("1");
  });

  test("keys typed into the seed field don't walk", async () => {
    const b = backend();
    const el = await mount();
    press("a", q(el, "#seed")!);
    press("ArrowLeft", q(el, "#seed")!);
    await settle(el);
    expect(b.steps).toEqual([]);
  });

  test("a key pressed while a step is in flight is dropped", async () => {
    const b = backend();
    let release!: () => void;
    b.stepAnswer = () => new Promise((r) => (release = () => r(viewAt(1))));
    const el = await mount();
    press("ArrowRight");
    press("ArrowRight");
    release();
    await settle(el);
    expect(b.steps).toEqual(["east"]);
  });

  test("a blocked move shows the cue, which then fades", async () => {
    const b = backend();
    b.stepAnswer = () => viewAt(0, false);
    const el = await mount();
    expect(q(el, "#cue")!.textContent).toBe("");
    vi.useFakeTimers();
    try {
      press("ArrowUp");
      await vi.advanceTimersByTimeAsync(0);
      await el.updateComplete;
      expect(q(el, "#cue")!.textContent).toContain("Blocked");
      expect(q(el, ".map")!.classList.contains("blocked")).toBe(true);
      expect(q(el, ".cell.player")!.dataset.x).toBe("0");
      await vi.advanceTimersByTimeAsync(BLOCKED_CUE_MS);
      await el.updateComplete;
      expect(q(el, "#cue")!.textContent).toBe("");
    } finally {
      vi.useRealTimers();
    }
  });

  test("New world starts the seed in the field", async () => {
    const b = backend();
    const el = await mount();
    const input = q(el, "#seed") as HTMLInputElement;
    input.value = "48213";
    input.dispatchEvent(new Event("input"));
    await el.updateComplete;
    q(el, "#new-world")!.click();
    await settle(el);
    expect(b.starts).toEqual([1, 48213]);
    expect(q(el, "#status")!.textContent).toContain("Seed 48213");
  });

  test("a bad seed disables New world and says why", async () => {
    backend();
    const el = await mount();
    const input = q(el, "#seed") as HTMLInputElement;
    input.value = "abc";
    input.dispatchEvent(new Event("input"));
    await el.updateComplete;
    expect((q(el, "#new-world") as HTMLButtonElement).disabled).toBe(true);
    expect(q(el, "#seed-problem")!.textContent).toContain("whole number");
  });

  test("a failed start shows Rust's error", async () => {
    const b = backend();
    b.startFails = "tile set failed to load";
    const el = await mount();
    expect(q(el, "#error")!.textContent).toBe("tile set failed to load");
  });
});
