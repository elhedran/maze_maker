import { describe, expect, test } from "vitest";
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";
import { COMMANDS, type Direction, type GameView, type TileKind, type ViewCell } from "./ipc.ts";
import "./game-app.ts";
import type { GameApp } from "./game-app.ts";

// Radius 1 around (x, 0): corridor E–W with hedge north, nothing stored south.
const cell = (x: number, y: number, tile: string, orientation: number, kind: TileKind, visited = false): ViewCell => ({
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
  focused: number;
  launchSeed: number;
  /** Holds start_game until released, to see "Growing the world…". */
  holdStart?: Promise<void>;
  startFails?: unknown;
}

function backend(overrides: Partial<Backend> = {}): Backend {
  const b: Backend = { starts: [], steps: [], focused: 0, launchSeed: 48213, ...overrides };
  let x = 0;
  mockWindows("main");
  mockIPC(async (cmd, args) => {
    const a = args as Record<string, unknown>;
    if (cmd === "plugin:window|set_focus") return void b.focused++;
    if (cmd === COMMANDS.initialSeed) return b.launchSeed;
    if (cmd === COMMANDS.startGame) {
      b.starts.push(a.seed as number);
      if (b.holdStart) await b.holdStart;
      if (b.startFails) throw b.startFails;
      x = 0;
      return viewAt(0);
    }
    if (cmd === COMMANDS.step) {
      const direction = a.direction as Direction;
      b.steps.push(direction);
      if (direction === "east" || direction === "west") {
        x += direction === "east" ? 1 : -1;
        return viewAt(x);
      }
      return viewAt(x, false);
    }
    throw new Error(`unexpected command ${cmd}`);
  });
  return b;
}

async function mount(): Promise<GameApp> {
  const el = document.createElement("game-app");
  document.body.append(el);
  await settle(el);
  return el;
}

/** Lets the mocked IPC promises resolve and Lit re-render. */
async function settle(el: GameApp) {
  for (let i = 0; i < 5; i++) {
    await new Promise((r) => setTimeout(r, 0));
    await el.updateComplete;
  }
}

const q = (el: GameApp, selector: string) => el.shadowRoot!.querySelector<HTMLElement>(selector);
const cells = (el: GameApp) => [...el.shadowRoot!.querySelectorAll<HTMLElement>(".map .cell")];
const key = (type: "keydown" | "keyup", k: string, target: EventTarget = window, repeat = false) =>
  target.dispatchEvent(new KeyboardEvent(type, { key: k, repeat, bubbles: true, composed: true, cancelable: true }));
/** A press and release, as a tap. */
async function tap(el: GameApp, k: string) {
  key("keydown", k);
  key("keyup", k);
  await settle(el);
}

describe("game-app", () => {
  test("starts the launch seed and draws the cells it gets back", async () => {
    const b = backend();
    const el = await mount();
    expect(b.starts).toEqual([48213]);
    const map = cells(el);
    expect(map.length).toBe(9);
    expect(map.map((c) => c.textContent).join("")).toBe("░░░░░░──@───      ");
    expect(map.filter((c) => c.classList.contains("hedge")).length).toBe(3);
    expect(q(el, ".map")!.getAttribute("aria-label")).toBe("World around you");
  });

  test("cells not yet known are the haze, never hedge", async () => {
    backend();
    const el = await mount();
    const south = cells(el).slice(6);
    expect(south.every((c) => c.classList.contains("unknown") && !c.classList.contains("hedge"))).toBe(true);
    expect(south[0].getAttribute("aria-label")).toBe("Not yet known");
  });

  test("visited cells carry the trail", async () => {
    backend();
    const el = await mount();
    const map = cells(el);
    expect(map[3].classList.contains("visited")).toBe(true);
    expect(map[5].classList.contains("visited")).toBe(false);
  });

  test("draws the player at the centre as a bold @, not by colour alone", async () => {
    backend();
    const el = await mount();
    const map = cells(el);
    expect(map.filter((c) => c.classList.contains("player"))).toEqual([map[4]]);
    expect(map[4].querySelector(".at")!.textContent).toBe("@");
    expect(map[4].classList.contains("corridor")).toBe(true);
    expect(map[4].getAttribute("aria-label")).toContain("you are here");
  });

  test("shows the seed, the position in words and the steps", async () => {
    backend();
    const el = await mount();
    expect(q(el, "#status")!.textContent!.trim()).toBe("Seed 48213 · at the start · 0 steps");
    await tap(el, "ArrowRight");
    await tap(el, "d");
    expect(q(el, "#status")!.textContent!.trim()).toBe("Seed 48213 · 2 east of the start · 2 steps");
    expect(q(el, ".cell.player")!.dataset.x).toBe("2");
  });

  test("arrow keys and WASD call step with the right direction", async () => {
    const b = backend();
    const el = await mount();
    for (const k of ["ArrowUp", "ArrowRight", "ArrowDown", "ArrowLeft", "w", "d", "s", "a"]) await tap(el, k);
    expect(b.steps).toEqual(["north", "east", "south", "west", "north", "east", "south", "west"]);
  });

  test("OS key repeats don't step; the game sets the pace", async () => {
    const b = backend();
    const el = await mount();
    key("keydown", "ArrowRight");
    await settle(el);
    key("keydown", "ArrowRight", window, true);
    key("keydown", "ArrowRight", window, true);
    key("keyup", "ArrowRight");
    await settle(el);
    expect(b.steps).toEqual(["east"]);
  });

  test("a blocked move says so in muted words, nudges the @ and doesn't count", async () => {
    backend();
    const el = await mount();
    await tap(el, "ArrowUp");
    const message = q(el, "#message")!;
    expect(message.textContent).toBe("Can't go north.");
    expect(message.classList.contains("muted")).toBe(true);
    expect(message.getAttribute("aria-live")).toBe("polite");
    expect(q(el, ".at")!.classList.contains("nudge-north")).toBe(true);
    expect(q(el, "#status")!.textContent).toContain("0 steps");
    expect(q(el, "#error")).toBeNull();
    // The next move that happens clears it.
    await tap(el, "ArrowRight");
    expect(q(el, "#message")!.textContent).toBe("");
    expect(q(el, ".at")!.className).not.toContain("nudge");
  });

  test("keys in a text field don't walk", async () => {
    const b = backend();
    const el = await mount();
    const input = document.createElement("input");
    document.body.append(input);
    key("keydown", "a", input);
    key("keydown", "ArrowLeft", input);
    await settle(el);
    expect(b.steps).toEqual([]);
  });

  test("focuses the window on load", async () => {
    const b = backend();
    await mount();
    expect(b.focused).toBe(1);
  });

  test('says "Growing the world…" until start_game answers', async () => {
    let release!: () => void;
    backend({ holdStart: new Promise<void>((r) => (release = r)) });
    const el = await mount();
    expect(q(el, "#status")!.textContent!.trim()).toBe("Growing the world…");
    expect(q(el, ".map")).toBeNull();
    release();
    await settle(el);
    expect(q(el, ".map")).not.toBeNull();
  });

  test("New world starts a new seed, resets the count and gives up focus", async () => {
    const b = backend();
    const el = await mount();
    await tap(el, "ArrowRight");
    const button = q(el, "#new-world") as HTMLButtonElement;
    button.focus();
    button.click();
    await settle(el);
    expect(b.starts.length).toBe(2);
    expect(q(el, "#status")!.textContent!.trim()).toBe(`Seed ${b.starts[1]} · at the start · 0 steps`);
    expect(el.shadowRoot!.activeElement).not.toBe(button);
  });

  test("a failed start shows Rust's error, with New world to try again", async () => {
    const b = backend({ startFails: { message: "tile set: line 3: unknown side" } });
    const el = await mount();
    expect(q(el, "#error")!.textContent).toBe("tile set: line 3: unknown side");
    expect(q(el, "#new-world")).not.toBeNull();
    b.startFails = undefined;
    q(el, "#new-world")!.click();
    await settle(el);
    expect(q(el, "#error")).toBeNull();
    expect(q(el, ".map")).not.toBeNull();
  });
});
