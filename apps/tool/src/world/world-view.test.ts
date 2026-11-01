import { afterEach, describe, expect, test, vi } from "vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { WORLD_COMMANDS } from "./world-ipc.ts";
import "./world-view.ts";
import type { WorldView } from "./world-view.ts";
import { TOOL_ERROR_EVENT } from "../tool-error.ts";
import type { ToolError } from "../bindings/ToolError.ts";
import type {
  Endpoint,
  PatchDelta,
  ReplayCheck,
  ReplayStepRequest,
  ReplayUpdate,
  WalkRequest,
  WorldStartRequest,
  WorldStarted,
  WorldUpdate,
} from "./world-types.ts";
import { TILES, patch } from "./fixture.ts";

// A small fake of DR-0041's commands. New world grows three patches:
//
//        │        patch 1 at (0, -1)
//      ─ ┼ ─      patch 0: the origin crossroads with a corridor either side
//        │        patch 2 at (0, 1)
//
// Each random step moves the player one cell east and grows one patch with a
// ─ just ahead of it. A key step east moves; north is blocked.

interface Backend {
  starts: WorldStartRequest[];
  walks: WalkRequest[];
  replaySteps: ReplayStepRequest[];
  worldId: number;
  player: { x: number; y: number };
  log: PatchDelta[];
  steps: number;
  walkFails?: ToolError;
  /** The check the last replay step returns. */
  check?: ReplayCheck;
  replayed: number;
}

const FRONTIER: Endpoint[] = [
  { cell: { x: 1, y: 0 }, facing: "east", anchor: { x: 2, y: 0 }, outward: true },
  { cell: { x: 0, y: -1 }, facing: "north", anchor: { x: 0, y: -2 }, outward: false },
];

function update(b: Backend, patches: PatchDelta[], path: { x: number; y: number }[] = []): WorldUpdate {
  return {
    worldId: b.worldId,
    player: { ...b.player },
    path,
    patches,
    frontier: FRONTIER,
    bounds: { x: -1, y: -1, width: 3, height: 3 },
    steps: b.steps,
    patchCount: b.log.length,
    storedCount: b.log.reduce((n, p) => n + p.cells.length, 0),
  };
}

function backend(overrides: Partial<Backend> = {}): Backend {
  const b: Backend = { starts: [], walks: [], replaySteps: [], worldId: 0, player: { x: 0, y: 0 }, log: [], steps: 0, replayed: 0, ...overrides };
  mockIPC((cmd, args) => {
    const request = (args as { request: unknown }).request;
    switch (cmd) {
      case WORLD_COMMANDS.worldStart: {
        b.starts.push(request as WorldStartRequest);
        b.worldId += 1;
        b.player = { x: 0, y: 0 };
        b.steps = 0;
        b.log = [
          patch(0, [[0, 0, 2], [-1, 0, 0], [1, 0, 0]]),
          { ...patch(1, [[0, -1, 1]]), anchor: { x: 0, y: -1 }, from: "north" },
          { ...patch(2, [[0, 1, 1]]), anchor: { x: 0, y: 1 }, from: "south" },
        ];
        const started: WorldStarted = {
          worldId: b.worldId,
          seed: (request as WorldStartRequest).seed,
          revision: 1,
          loadedAt: "2026-11-01T20:14:03Z",
          tiles: TILES,
          update: update(b, b.log),
        };
        return started;
      }
      case WORLD_COMMANDS.worldWalk: {
        const walk = request as WalkRequest;
        b.walks.push(walk);
        if (b.walkFails) throw b.walkFails;
        if (walk.worldId !== b.worldId) throw { kind: "stale", messages: ["That world has been replaced. Press New world."] };
        if (walk.walk.kind === "step") {
          if (walk.walk.direction !== "east") return update(b, []);
          b.player.x += 1;
          b.steps += 1;
          return update(b, [], [{ ...b.player }]);
        }
        b.player.x += 1;
        b.steps += 1;
        const grown = { ...patch(b.log.length, [[b.player.x + 1, 0, 0]], b.steps), anchor: { x: b.player.x + 1, y: 0 }, from: "east" as const };
        b.log.push(grown);
        return update(b, [grown], [{ ...b.player }]);
      }
      case WORLD_COMMANDS.worldReplayStart: {
        b.replayed = 0;
        const start: ReplayUpdate = { worldId: b.worldId, patches: [], frontier: [], bounds: null, replayed: 0, total: b.log.length, check: null };
        return start;
      }
      case WORLD_COMMANDS.worldReplayStep: {
        const step = request as ReplayStepRequest;
        b.replaySteps.push(step);
        const patches = b.log.slice(b.replayed, b.replayed + step.patches);
        b.replayed += patches.length;
        const done = b.replayed === b.log.length || (b.check !== undefined && !b.check.same && b.replayed >= 2);
        const cells = b.log.slice(0, b.replayed).reduce((n, p) => n + p.cells.length, 0);
        const result: ReplayUpdate = {
          worldId: b.worldId,
          patches,
          frontier: FRONTIER,
          bounds: { x: -1, y: -1, width: 3, height: 3 },
          replayed: b.replayed,
          total: b.log.length,
          check: done ? (b.check ?? { same: true, cells, firstDifference: null }) : null,
        };
        return result;
      }
      default:
        throw new Error(`not under test: ${cmd}`);
    }
  });
  return b;
}

function mount(): WorldView {
  const el = document.createElement("world-view");
  document.body.append(el);
  return el;
}

const $ = <T extends Element = HTMLElement>(el: WorldView, selector: string) => el.shadowRoot!.querySelector<T>(selector);
const text = (el: WorldView, selector: string) => $(el, selector)?.textContent?.trim() ?? "";
const cell = (el: WorldView, x: number, y: number) => $(el, `.cell[data-x="${x}"][data-y="${y}"]`)!;
const click = (el: WorldView, selector: string) => $<HTMLButtonElement>(el, selector)!.click();
const key = (el: WorldView, k: string, shiftKey = false) =>
  $(el, ".map")!.dispatchEvent(new KeyboardEvent("keydown", { key: k, shiftKey, bubbles: true, composed: true }));

async function fast(el: WorldView) {
  await el.updateComplete;
  const speed = $<HTMLSelectElement>(el, "#speed")!;
  speed.value = "fast";
  speed.dispatchEvent(new Event("change"));
}

/** Mounts, sets Fast, presses New world and waits for all three patches. */
async function grown(): Promise<WorldView> {
  const el = mount();
  await fast(el);
  click(el, "#new-world");
  await vi.waitFor(() => expect(text(el, "#patch-panel")).toContain("PATCH 3 of 3"));
  return el;
}

afterEach(() => {
  document.body.innerHTML = "";
  clearMocks();
});

describe("world-view", () => {
  test("before New world it asks for one and calls nothing", async () => {
    const b = backend();
    const el = mount();
    await el.updateComplete;
    expect(el.shadowRoot!.textContent).toContain("Press New world to grow a world from this seed.");
    expect(b.starts).toEqual([]);
    expect($<HTMLButtonElement>(el, "#play")!.disabled).toBe(true);
    expect($<HTMLButtonElement>(el, "#replay")!.disabled).toBe(true);
  });

  test("New world sends the typed seed, and its patches arrive one per tick, in order", async () => {
    const b = backend();
    const el = mount();
    await el.updateComplete;
    const seed = $<HTMLInputElement>(el, "#world-seed")!;
    seed.value = "48213";
    seed.dispatchEvent(new Event("input"));
    click(el, "#new-world");
    await vi.waitFor(() => expect(text(el, "#patch-panel")).toContain("PATCH 1 of 3"));
    expect(b.starts).toEqual([{ seed: 48213 }]);
    // The origin patch is in; the later ones aren't yet.
    expect(cell(el, 0, 0).getAttribute("aria-label")).toBe("Corridor crossroads · x 0, y 0 · stored by patch 1");
    expect(cell(el, 0, 1).getAttribute("aria-label")).toBe("Unsettled · x 0, y 1");
    await vi.waitFor(() => expect(text(el, "#patch-panel")).toContain("PATCH 2 of 3"));
    expect(cell(el, 0, -1).textContent).toBe("│ ");
    expect(cell(el, 0, 1).getAttribute("aria-label")).toBe("Unsettled · x 0, y 1");
    await vi.waitFor(() => expect(text(el, "#patch-panel")).toContain("PATCH 3 of 3"));
    expect(cell(el, 0, 1).getAttribute("aria-label")).toBe("Corridor straight, N–S · x 0, y 1 · stored by patch 3");
    expect(text(el, "#status")).toBe("World 48213 · 0 steps · 3 patches · 5 cells");
  });

  test("draws the player, the origin, the frontier and the last patch", async () => {
    backend();
    const el = await grown();
    expect(cell(el, 0, 0).classList).toContain("player");
    expect(cell(el, 0, 0).classList).toContain("origin");
    expect(cell(el, 2, 0).textContent).toBe("→ ");
    expect(cell(el, 2, 0).getAttribute("aria-label")).toBe("Unsettled · x 2, y 0 · open endpoint, outward");
    expect(cell(el, 0, -2).textContent).toBe("○ ");
    expect(text(el, "#frontier-count")).toBe("Frontier: 2 open, 1 outward");
    // Last patch (patch 3) is full strength; earlier patches are dimmed.
    expect(cell(el, 0, 1).classList).not.toContain("older");
    expect(cell(el, 0, -1).classList).toContain("older");
    // Its window (5 × 5 from -2, -2) is outlined on its edges.
    expect(cell(el, -2, -2).className).toMatch(/win-n.*win-w/);
    expect(cell(el, 2, 2).className).toMatch(/win-e.*win-s/);
    expect(cell(el, 0, 0).className).not.toMatch(/win-/);
  });

  test("overlays toggle by button and by key, with aria-pressed", async () => {
    backend();
    const el = await grown();
    // Stored on: unsettled is blank. Off: solid hedge.
    expect(cell(el, 5, 5).textContent).toBe("  ");
    key(el, "s");
    await el.updateComplete;
    expect($(el, "#overlay-stored")!.getAttribute("aria-pressed")).toBe("false");
    expect(cell(el, 5, 5).textContent).toBe("░░");
    expect(cell(el, 5, 5).classList).toContain("hedge");
    click(el, "#overlay-frontier");
    await el.updateComplete;
    expect(cell(el, 2, 0).textContent).toBe("░░");
    key(el, "l");
    await el.updateComplete;
    expect(cell(el, 0, -1).classList).not.toContain("older");
    expect(el.shadowRoot!.querySelector('[class*="win-"]')).toBeNull();
  });

  test("Patch debug adds the patch in words", async () => {
    backend();
    const el = await grown();
    expect(text(el, "#patch-panel")).not.toContain("window");
    key(el, "d");
    await el.updateComplete;
    const panel = text(el, "#patch-panel");
    expect(panel).toContain("anchor x 0, y 1, grown S");
    expect(panel).toContain("window x -2..2, y -2..2");
    expect(panel).toContain("spine: no");
    expect(panel).toContain("fallback: no");
    expect(panel).toContain("stored 1");
  });

  test("Step walks one seeded step, and the step's patch arrives", async () => {
    const b = backend();
    const el = await grown();
    const walkSeed = $<HTMLInputElement>(el, "#walk-seed")!;
    walkSeed.value = "907";
    walkSeed.dispatchEvent(new Event("input"));
    await el.updateComplete;
    click(el, "#step");
    await vi.waitFor(() => expect(text(el, "#status")).toMatch(/^World \d+ · 1 step · 4 patches · 6 cells$/));
    expect(b.walks).toEqual([{ worldId: 1, walk: { kind: "random", seed: 907, steps: 1 } }]);
    expect(cell(el, 1, 0).classList).toContain("player");
    expect(cell(el, 2, 0).getAttribute("aria-label")).toBe("Corridor straight, E–W · x 2, y 0 · stored by patch 4");
  });

  test("a blocked arrow key moves nothing and says what blocked it", async () => {
    const b = backend();
    const el = await grown();
    click(el, "#step");
    await vi.waitFor(() => expect(cell(el, 1, 0).classList).toContain("player"));
    key(el, "ArrowUp");
    await vi.waitFor(() => expect(text(el, "#readout")).toBe("Hedge to the north"));
    expect(b.walks[1]).toEqual({ worldId: 1, walk: { kind: "step", direction: "north" } });
    expect(cell(el, 1, 0).classList).toContain("player");
    key(el, "ArrowRight");
    await vi.waitFor(() => expect(cell(el, 2, 0).classList).toContain("player"));
    expect(text(el, "#readout")).toBe("");
  });

  test("the readout, Centre and legend sit in the side column, readout first, above the patch", async () => {
    backend();
    const el = await grown();
    // At the default 1024×768 window anything under the map is below the fold (Damson, fix 2).
    const side = $(el, ".side")!;
    const order = [...side.children].map((c) => c.id || c.className);
    expect(order).toEqual(["readout", "bar", "patch-panel", "legend muted"]);
    expect($(el, ".stage #readout, .stage #centre, .stage .legend")).toBeNull();
    expect($(el, ".side #centre")).not.toBeNull();
  });

  test("Play walks Steps steps, one call per step, then stops", async () => {
    const b = backend();
    const el = await grown();
    const steps = $<HTMLInputElement>(el, "#steps")!;
    steps.value = "3";
    steps.dispatchEvent(new Event("input"));
    await el.updateComplete;
    click(el, "#play");
    await vi.waitFor(() => expect(text(el, "#status")).toContain("3 steps · 6 patches"));
    await vi.waitFor(() => expect($<HTMLButtonElement>(el, "#play")!.disabled).toBe(false));
    const seed = Number($<HTMLInputElement>(el, "#walk-seed")!.value);
    expect(b.walks.map((w) => w.walk)).toEqual([1, 2, 3].map(() => ({ kind: "random", seed, steps: 1 })));
  });

  test("a bad field disables what needs it and says why", async () => {
    backend();
    const el = await grown();
    const steps = $<HTMLInputElement>(el, "#steps")!;
    steps.value = "0";
    steps.dispatchEvent(new Event("input"));
    await el.updateComplete;
    expect($<HTMLButtonElement>(el, "#play")!.disabled).toBe(true);
    expect(text(el, "#field-problem")).toContain("Steps is a whole number from 1 to 1,000,000.");
    const seed = $<HTMLInputElement>(el, "#world-seed")!;
    seed.value = "x";
    seed.dispatchEvent(new Event("input"));
    await el.updateComplete;
    expect($<HTMLButtonElement>(el, "#new-world")!.disabled).toBe(true);
  });

  test("Replay regrows patch by patch and ends with Rust's same-world answer", async () => {
    const b = backend();
    const el = await grown();
    expect($<HTMLButtonElement>(el, "#replay")!.disabled).toBe(true);
    click(el, "#step");
    await vi.waitFor(() => expect($<HTMLButtonElement>(el, "#replay")!.disabled).toBe(false));
    click(el, "#replay");
    await vi.waitFor(() => expect(text(el, "#replay-progress")).toBe("Replay · patch 0 of 4"));
    // Walking is off during replay, and the map starts empty.
    expect($<HTMLButtonElement>(el, "#step")!.disabled).toBe(true);
    expect(cell(el, 0, 0).getAttribute("aria-label")).toBe("Unsettled · x 0, y 0");
    click(el, "#replay-step");
    await vi.waitFor(() => expect(text(el, "#replay-progress")).toBe("Replay · patch 1 of 4"));
    click(el, "#replay-play");
    await vi.waitFor(() => expect(text(el, "#status")).toBe("Replayed 4 patches · same world as walked: yes (6 cells match)"));
    expect(b.replaySteps.every((s) => s.patches === 1 && s.worldId === 1)).toBe(true);
    expect($(el, "#status")!.classList).not.toContain("problem");
    expect($<HTMLButtonElement>(el, "#replay-play")!.disabled).toBe(true);
    click(el, "#back-to-walk");
    await el.updateComplete;
    expect($(el, "#timeline")).toBeNull();
    expect(cell(el, 1, 0).classList).toContain("player");
    expect(text(el, "#status")).toContain("1 step · 4 patches · 6 cells");
  });

  test("Replay keeps the frame the walk ended on", async () => {
    backend();
    const el = await grown();
    click(el, "#step");
    await vi.waitFor(() => expect($<HTMLButtonElement>(el, "#replay")!.disabled).toBe(false));
    key(el, "ArrowRight", true);
    await el.updateComplete;
    const corner = () => {
      const first = $(el, ".map .cell")!;
      return { x: first.dataset.x, y: first.dataset.y };
    };
    const before = corner();
    click(el, "#replay");
    await vi.waitFor(() => expect($(el, "#replay-play")).not.toBeNull());
    click(el, "#replay-play");
    await vi.waitFor(() => expect(text(el, "#status")).toContain("same world as walked: yes"));
    expect(corner()).toEqual(before);
  });

  test("a replay that differs ends at Rust's check, early, in the problem colour", async () => {
    backend({ check: { same: false, cells: 4, firstDifference: { x: 0, y: -1 } } });
    const el = await grown();
    click(el, "#step");
    await vi.waitFor(() => expect($<HTMLButtonElement>(el, "#replay")!.disabled).toBe(false));
    click(el, "#replay");
    await vi.waitFor(() => expect($(el, "#replay-play")).not.toBeNull());
    click(el, "#replay-play");
    await vi.waitFor(() => expect(text(el, "#status")).toBe("Replayed 2 patches · same world as walked: no · first difference at x 0, y -1"));
    expect($(el, "#status")!.classList).toContain("problem");
    expect(text(el, "#replay-progress")).toBe("Replay · patch 2 of 4");
  });

  test("a failed walk goes to the shared banner and the world stays", async () => {
    backend({ walkFails: { kind: "stale", messages: ["That world has been replaced. Press New world."] } });
    const el = await grown();
    const errors: string[] = [];
    el.addEventListener(TOOL_ERROR_EVENT, (e) => errors.push((e as CustomEvent<string>).detail));
    click(el, "#step");
    await vi.waitFor(() => expect(errors).toEqual(["That world has been replaced. Press New world."]));
    expect(cell(el, 0, 0).classList).toContain("player");
    expect(text(el, "#status")).toContain("3 patches · 5 cells");
    expect($<HTMLButtonElement>(el, "#step")!.disabled).toBe(false);
  });

  test("after a Reload the world keeps its set and says so", async () => {
    backend();
    const el = await grown();
    el.tileSetReloaded();
    await el.updateComplete;
    expect(text(el, "#status")).toMatch(/· grown with the tile set loaded .+ · New world uses the reloaded one$/);
    click(el, "#new-world");
    await vi.waitFor(() => expect(text(el, "#status")).not.toContain("grown with"));
  });

  test("Shift+arrows pan and stop following; C centres again", async () => {
    backend();
    const el = await grown();
    expect($<HTMLButtonElement>(el, "#centre")!.disabled).toBe(true);
    key(el, "ArrowRight", true);
    await el.updateComplete;
    // The origin cell is now 8 cells further left on screen, so (−20, 0) has scrolled off.
    expect($(el, '.cell[data-x="-20"][data-y="0"]')).toBeNull();
    expect($(el, '.cell[data-x="27"][data-y="0"]')).not.toBeNull();
    expect($<HTMLButtonElement>(el, "#centre")!.disabled).toBe(false);
    key(el, "c");
    await el.updateComplete;
    expect($(el, '.cell[data-x="-20"][data-y="0"]')).not.toBeNull();
    expect($<HTMLButtonElement>(el, "#centre")!.disabled).toBe(true);
  });
});
