import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { errorText } from "../ipc.ts";
import { themeStyles } from "../table/draw-tile.ts";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import { formatLoadTime } from "../table/format.ts";
import { reportToolError } from "../tool-error.ts";
import { eastFiller, parseSeed, randomSeed } from "../maze/maze-text.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { TileView } from "../bindings/TileView.ts";
import { worldReplayStart, worldReplayStep, worldStart, worldWalk } from "./world-ipc.ts";
import type { PatchDelta, ReplayCheck, ReplayUpdate, WorldPos, WorldUpdate } from "./world-types.ts";
import { PAN_STEP, VIEW_HEIGHT, VIEW_WIDTH, WorldModel, centredOn, followed } from "./world-model.ts";
import {
  DEFAULT_STEPS,
  SPEEDS,
  type Speed,
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

type Overlay = "stored" | "frontier" | "last" | "debug";
const OVERLAYS: { id: Overlay; label: string; key: string }[] = [
  { id: "stored", label: "Stored", key: "S" },
  { id: "frontier", label: "Frontier", key: "F" },
  { id: "last", label: "Last patch", key: "L" },
  { id: "debug", label: "Patch debug", key: "D" },
];

const ARROW_KEYS: Record<string, Direction> = {
  ArrowUp: "north",
  ArrowRight: "east",
  ArrowDown: "south",
  ArrowLeft: "west",
};
const PAN: Record<Direction, WorldPos> = {
  north: { x: 0, y: -PAN_STEP },
  east: { x: PAN_STEP, y: 0 },
  south: { x: 0, y: PAN_STEP },
  west: { x: -PAN_STEP, y: 0 },
};

/** The world as Rust started it: the id requests name, and the tile list its cells index. */
interface WorldInfo {
  id: number;
  seed: number;
  loadedAt: string;
  tiles: TileView[];
}

interface Totals {
  steps: number;
  patchCount: number;
  storedCount: number;
}

/** Replay's own world, beside the walked one (DR-0041 rule 11). */
interface Replay {
  model: WorldModel;
  total: number;
  replayed: number;
  /** Rust's same-world answer; non-null ends replay, even early (Rowan, DR-0041 Concerns). */
  check: ReplayCheck | null;
}

/**
 * The world view (DR-0039, with Marram's cuts): grow a world from a seed,
 * walk it by seeded random walk or arrow keys, watch patches arrive one per
 * tick with the frontier and last patch overlays, then replay the growth log
 * and read Rust's same-world answer. Rust grows and checks; this only draws.
 */
@customElement("world-view")
export class WorldView extends LitElement {
  @state() private worldSeedText = String(randomSeed());
  @state() private walkSeedText = String(randomSeed());
  @state() private stepsText = String(DEFAULT_STEPS);
  @state() private speed: Speed = "normal";
  @state() private overlays: Record<Overlay, boolean> = { stored: true, frontier: true, last: true, debug: false };

  @state() private world?: WorldInfo;
  @state() private totals: Totals = { steps: 0, patchCount: 0, storedCount: 0 };
  @state() private replay?: Replay;
  @state() private playing = false;
  @state() private starting = false;
  @state() private busy = false;
  /** The view's north-west corner, in world cells. */
  @state() private corner: WorldPos = centredOn({ x: 0, y: 0 });
  @state() private follow = true;
  @state() private hover?: WorldPos;
  /** Why the last key step didn't move. Cleared by the next move. */
  @state() private blocked?: string;
  @state() private reloadedNote?: string;

  /** The walked world as revealed so far. Mutated in place; changes call requestUpdate. */
  private walk = new WorldModel();
  /** Patches received but not yet shown: one is revealed per tick (DR-0039 rule 4). */
  private queue: PatchDelta[] = [];
  /** What to apply once the queue empties: the response's frontier and totals. */
  private afterQueue?: () => void;
  private stepsLeft = 0;
  /** The walked log's length as Rust last reported it, for "PATCH k of n" while patches arrive. */
  private patchTotal = 0;
  private timer?: ReturnType<typeof setTimeout>;
  /** Bumped by New world and by entering or leaving replay, so a late response is dropped. */
  private generation = 0;
  /** Ignores a world_start response when a newer New world was pressed. */
  private startSerial = 0;

  static styles = [
    themeStyles,
    css`
      :host { display: flex; flex-direction: column; gap: 0.5rem; font-family: system-ui, sans-serif; font-size: 14px; }
      .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
      .muted { color: var(--muted); }
      .problem { color: var(--problem); margin: 0; white-space: pre-wrap; }
      p { margin: 0; }
      input, select {
        font: inherit;
        font-family: var(--mono);
        padding: 0.2rem 0.4rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
      }
      input { width: 9rem; }
      input#steps { width: 6rem; }
      button {
        font: inherit;
        padding: 0.25rem 0.75rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      button:disabled { cursor: default; opacity: 0.6; }
      button[aria-pressed="true"] { background: var(--text); color: var(--page); }
      .body { display: flex; gap: 1.5rem; align-items: flex-start; }
      .stage { display: flex; flex-direction: column; gap: 0.25rem; }
      .map {
        font-family: var(--mono);
        font-size: 18px;
        font-variant-ligatures: none;
        font-synthesis: none;
        white-space: pre;
        border: 1px solid var(--frame);
        width: max-content;
      }
      .map:focus-visible { outline: 2px solid var(--text); outline-offset: 2px; }
      .map.empty { display: flex; align-items: center; justify-content: center; box-sizing: border-box; }
      .note { white-space: normal; text-align: center; font-family: system-ui, sans-serif; font-size: 14px; padding: 1rem; }
      .row { display: block; }
      .cell { position: relative; background-repeat: no-repeat; }
      .cell.corridor { color: var(--corridor-line); }
      .cell.room { color: var(--room-line); }
      .cell.hedge { color: var(--hedge-glyph); background-color: var(--hedge-bg); }
      .cell[class*="tint-"] { background-image: linear-gradient(var(--room-tint), var(--room-tint)); }
      .tint-full { background-size: 100% 100%; }
      .tint-lower { background-size: 100% 50%; background-position: bottom; }
      .tint-upper { background-size: 100% 50%; background-position: top; }
      .tint-left { background-size: 50% 100%; background-position: left; }
      .tint-right { background-size: 50% 100%; background-position: right; }
      .tint-lower-left { background-size: 50% 50%; background-position: left bottom; }
      .tint-upper-left { background-size: 50% 50%; background-position: left top; }
      .tint-upper-right { background-size: 50% 50%; background-position: right top; }
      .tint-lower-right { background-size: 50% 50%; background-position: right bottom; }
      /* Frontier marks: text colour and bold, never a tint (DR-0039 rule 7). */
      .cell.mark { color: var(--text); font-weight: 700; }
      /* Older patches dimmed under Last patch (rule 8): brightness, not hue. */
      .cell.older { opacity: 0.5; }
      /* The last patch's window: a dashed 2px outline drawn on its edge cells. */
      .cell[class*="win-"]::before {
        content: "";
        position: absolute;
        inset: 0;
        pointer-events: none;
        border: 0 dashed var(--text);
      }
      .cell.win-n::before { border-top-width: 2px; }
      .cell.win-e::before { border-right-width: 2px; }
      .cell.win-s::before { border-bottom-width: 2px; }
      .cell.win-w::before { border-left-width: 2px; }
      .cell.origin { outline: 1px dotted var(--text); outline-offset: -1px; }
      /* The player: inverted, with an outline; never red or magenta (rule 6). */
      .cell.player {
        background: var(--text);
        color: var(--page);
        opacity: 1;
        outline: 2px solid var(--text);
        outline-offset: 1px;
      }
      /* Two lines: in the side column a cell's readout often wraps, and a
         reserved second line stops the column jumping as the pointer moves. */
      .readout { min-height: 2.8em; }
      .side { flex: 0 0 18rem; display: flex; flex-direction: column; gap: 0.5rem; }
      .patch-lines { display: flex; flex-direction: column; gap: 0.25rem; font-family: var(--mono); }
      .patch-lines .gap { height: 0.5rem; }
      .legend { margin: 0; font-size: 0.85rem; }
    `,
  ];

  disconnectedCallback(): void {
    this.stopTimer();
    super.disconnectedCallback();
  }

  /**
   * Called by the app after a successful Reload (DR-0039 rule 12): the world
   * stays on the set it was grown with, and the status line says so.
   */
  tileSetReloaded(): void {
    if (this.world) this.reloadedNote = reloadNote(formatLoadTime(this.world.loadedAt));
  }

  // ---- The model on screen ----

  private get model(): WorldModel {
    return this.replay?.model ?? this.walk;
  }

  private tileOf(tile: number): TileView | undefined {
    return this.world?.tiles[tile];
  }

  // ---- Ticks: one patch per tick, then the next walk or replay step ----

  private stopTimer() {
    if (this.timer !== undefined) clearTimeout(this.timer);
    this.timer = undefined;
  }

  private schedule() {
    if (this.timer !== undefined) return;
    if (this.queue.length === 0 && !this.playing) return;
    this.timer = setTimeout(() => this.tick(), SPEEDS[this.speed]);
  }

  private tick() {
    this.timer = undefined;
    if (this.queue.length > 0) {
      this.revealOne();
    } else if (this.playing && !this.busy) {
      if (this.replay) void this.replayStep();
      else if (this.stepsLeft > 0) void this.walkRandom();
      else this.playing = false;
    }
    this.schedule();
  }

  private revealOne() {
    const next = this.queue.shift();
    if (next) this.model.addPatch(next);
    if (this.queue.length === 0) this.drainEnd();
    this.requestUpdate();
  }

  private drainEnd() {
    const end = this.afterQueue;
    this.afterQueue = undefined;
    end?.();
  }

  /** Shows everything still queued at once, e.g. before replay starts. */
  private flushQueue() {
    for (const p of this.queue) this.model.addPatch(p);
    this.queue = [];
    this.drainEnd();
  }

  /** Queues a response's patches; its frontier (and `end`) apply once they are all shown. */
  private enqueue(patches: PatchDelta[], end: () => void) {
    this.queue.push(...patches);
    const previous = this.afterQueue;
    this.afterQueue = () => {
      previous?.();
      end();
    };
    if (this.queue.length === 0) this.drainEnd();
    this.requestUpdate();
    this.schedule();
  }

  private fail(e: unknown) {
    this.playing = false;
    reportToolError(this, errorText(e));
  }

  // ---- New world ----

  private async newWorld(seed: number) {
    const serial = ++this.startSerial;
    this.playing = false;
    this.starting = true;
    try {
      const started = await worldStart({ seed });
      if (serial !== this.startSerial) return;
      // Only now does the old world go: a failed start keeps it, walk and all (DR-0039 rule 13).
      this.generation++;
      this.busy = false;
      this.stopTimer();
      this.world = { id: started.worldId, seed: started.seed, loadedAt: started.loadedAt, tiles: started.tiles };
      this.walk = new WorldModel();
      this.queue = [];
      this.afterQueue = undefined;
      this.replay = undefined;
      this.stepsLeft = 0;
      this.blocked = undefined;
      this.reloadedNote = undefined;
      this.totals = { steps: 0, patchCount: 0, storedCount: 0 };
      this.patchTotal = 0;
      this.follow = true;
      this.corner = centredOn(started.update.player);
      this.applyWalk(started.update);
    } catch (e) {
      if (serial === this.startSerial) this.fail(e);
    } finally {
      if (serial === this.startSerial) this.starting = false;
    }
  }

  private onNewWorld() {
    const parsed = parseSeed(this.worldSeedText);
    if ("seed" in parsed) void this.newWorld(parsed.seed);
  }

  private onRandomWorld() {
    const seed = randomSeed();
    this.worldSeedText = String(seed);
    void this.newWorld(seed);
  }

  // ---- Walking ----

  /** The player moves now; the patches arrive one per tick, then the frontier and totals. */
  private applyWalk(update: WorldUpdate) {
    this.walk.player = update.player;
    this.patchTotal = update.patchCount;
    if (this.follow) this.corner = followed(this.corner, update.player);
    this.enqueue(update.patches, () => {
      this.walk.setFrontier(update.frontier);
      this.totals = { steps: update.steps, patchCount: update.patchCount, storedCount: update.storedCount };
    });
  }

  private async walkRandom() {
    const world = this.world;
    const seed = parseSeed(this.walkSeedText);
    if (!world || !("seed" in seed) || this.busy) return;
    const generation = this.generation;
    this.busy = true;
    try {
      const update = await worldWalk({ worldId: world.id, walk: { kind: "random", seed: seed.seed, steps: 1 } });
      if (generation !== this.generation) return;
      this.stepsLeft = Math.max(0, this.stepsLeft - 1);
      this.blocked = undefined;
      this.applyWalk(update);
    } catch (e) {
      if (generation === this.generation) this.fail(e);
    } finally {
      if (generation === this.generation) this.busy = false;
      this.schedule();
    }
  }

  private async walkKey(direction: Direction) {
    const world = this.world;
    if (!world || this.replay || this.busy) return;
    this.playing = false;
    const generation = this.generation;
    const from = this.walk.player;
    this.busy = true;
    try {
      const update = await worldWalk({ worldId: world.id, walk: { kind: "step", direction } });
      if (generation !== this.generation) return;
      if (update.path.length === 0) {
        const here = from && this.walk.cellAt(from.x, from.y);
        this.blocked = blockedText(here ? this.tileOf(here.tile) : undefined, direction);
      } else {
        this.blocked = undefined;
      }
      this.applyWalk(update);
    } catch (e) {
      if (generation === this.generation) this.fail(e);
    } finally {
      if (generation === this.generation) this.busy = false;
    }
  }

  private onPlay() {
    if (this.replay) {
      if (!this.replay.check) this.playing = true;
    } else {
      const steps = parseSteps(this.stepsText);
      if (!("steps" in steps)) return;
      if (this.stepsLeft <= 0) this.stepsLeft = steps.steps;
      this.playing = true;
    }
    this.schedule();
  }

  private onPause() {
    this.playing = false;
  }

  private onStep() {
    this.playing = false;
    if (this.replay) void this.replayStep();
    else void this.walkRandom();
  }

  // ---- Replay ----

  private get canReplay(): boolean {
    return !!this.world && !this.replay && !this.busy && this.walk.patches.some((p) => p.step > 0);
  }

  private async startReplay() {
    const world = this.world;
    if (!world || !this.canReplay) return;
    this.playing = false;
    this.stopTimer();
    this.flushQueue();
    const generation = ++this.generation;
    this.busy = true;
    try {
      const update = await worldReplayStart({ worldId: world.id });
      if (generation !== this.generation) return;
      this.replay = { model: new WorldModel(), total: update.total, replayed: 0, check: null };
      this.corner = centredOn({ x: 0, y: 0 });
      this.applyReplay(update);
    } catch (e) {
      if (generation === this.generation) this.fail(e);
    } finally {
      if (generation === this.generation) this.busy = false;
    }
  }

  private async replayStep() {
    const world = this.world;
    const replay = this.replay;
    if (!world || !replay || replay.check || this.busy) return;
    const generation = this.generation;
    this.busy = true;
    try {
      const update = await worldReplayStep({ worldId: world.id, patches: 1 });
      if (generation !== this.generation) return;
      this.applyReplay(update);
    } catch (e) {
      if (generation === this.generation) this.fail(e);
    } finally {
      if (generation === this.generation) this.busy = false;
      this.schedule();
    }
  }

  private applyReplay(update: ReplayUpdate) {
    const replay = this.replay;
    if (!replay) return;
    replay.total = update.total;
    if (update.check) this.playing = false;
    this.enqueue(update.patches, () => {
      replay.model.setFrontier(update.frontier);
      replay.replayed = update.replayed;
      replay.check = update.check;
    });
  }

  private backToWalk() {
    this.generation++;
    this.stopTimer();
    this.queue = [];
    this.afterQueue = undefined;
    this.playing = false;
    this.busy = false;
    this.replay = undefined;
    this.follow = true;
    if (this.walk.player) this.corner = centredOn(this.walk.player);
  }

  // ---- Panning and keys ----

  private centre() {
    this.follow = true;
    this.corner = centredOn(this.replay ? { x: 0, y: 0 } : (this.walk.player ?? { x: 0, y: 0 }));
  }

  private toggle(overlay: Overlay) {
    this.overlays = { ...this.overlays, [overlay]: !this.overlays[overlay] };
  }

  /**
   * Arrows walk (Shift+arrows pan), C centres, Space plays or pauses, "."
   * steps, S F L D toggle the overlays. WASD is left out: S and D would both
   * walk and toggle (DR-0039 Concerns, Ash on build).
   */
  private onMapKeydown(event: KeyboardEvent) {
    const direction = ARROW_KEYS[event.key];
    const key = event.key.toLowerCase();
    if (direction && event.shiftKey) {
      this.follow = false;
      this.corner = { x: this.corner.x + PAN[direction].x, y: this.corner.y + PAN[direction].y };
    } else if (direction) {
      void this.walkKey(direction);
    } else if (key === " ") {
      if (this.playing) this.onPause();
      else this.onPlay();
    } else if (key === ".") {
      this.onStep();
    } else if (key === "c") {
      this.centre();
    } else {
      const overlay = OVERLAYS.find((o) => o.key.toLowerCase() === key);
      if (!overlay || event.ctrlKey || event.metaKey || event.altKey) return;
      this.toggle(overlay.id);
    }
    event.preventDefault();
  }

  private onMapHover(event: Event) {
    const cell = (event.target as HTMLElement).closest<HTMLElement>(".cell");
    if (!cell) return;
    const x = Number(cell.dataset.x);
    const y = Number(cell.dataset.y);
    if (this.hover?.x !== x || this.hover?.y !== y) this.hover = { x, y };
  }

  // ---- Drawing ----

  private cellText(x: number, y: number): string {
    const stored = this.model.cellAt(x, y);
    if (stored) {
      const tile = this.tileOf(stored.tile);
      return storedCellText(tile ? tileLabel(tile) : "Unknown tile", x, y, stored.patch);
    }
    return unsettledCellText(x, y, this.model.endpointAt(x, y));
  }

  render() {
    const worldSeed = parseSeed(this.worldSeedText);
    const walkSeed = parseSeed(this.walkSeedText);
    const steps = parseSteps(this.stepsText);
    const replaying = !!this.replay;
    const canWalk = !!this.world && !replaying;
    const problems = [
      "problem" in worldSeed ? `World seed: ${worldSeed.problem}` : "",
      "problem" in walkSeed ? `Walk seed: ${walkSeed.problem}` : "",
      "problem" in steps ? steps.problem : "",
    ].filter((p) => p);
    return html`
      <div class="bar">
        <label>World seed
          <input id="world-seed" inputmode="numeric" autocomplete="off" .value=${this.worldSeedText}
            aria-invalid=${"problem" in worldSeed ? "true" : "false"}
            @input=${(e: Event) => (this.worldSeedText = (e.target as HTMLInputElement).value)}
            @keydown=${(e: KeyboardEvent) => {
              if (e.key === "Enter" && !this.starting) {
                e.preventDefault();
                this.onNewWorld();
              }
            }} />
        </label>
        <button id="new-world" @click=${this.onNewWorld} ?disabled=${"problem" in worldSeed || this.starting}>
          ${this.starting ? "Growing…" : "New world"}
        </button>
        <button id="random-world" @click=${this.onRandomWorld} ?disabled=${this.starting}>Random seed</button>
      </div>
      <div class="bar">
        <label>Walk seed
          <input id="walk-seed" inputmode="numeric" autocomplete="off" .value=${this.walkSeedText}
            aria-invalid=${"problem" in walkSeed ? "true" : "false"}
            @input=${(e: Event) => (this.walkSeedText = (e.target as HTMLInputElement).value)} />
        </label>
        <label>Steps
          <input id="steps" inputmode="numeric" autocomplete="off" .value=${this.stepsText}
            aria-invalid=${"problem" in steps ? "true" : "false"}
            @input=${(e: Event) => {
              this.stepsText = (e.target as HTMLInputElement).value;
              this.stepsLeft = 0;
            }} />
        </label>
        <button id="play" @click=${this.onPlay} ?disabled=${!canWalk || this.playing || "problem" in walkSeed || "problem" in steps}>&gt; Play</button>
        <button id="pause" @click=${this.onPause} ?disabled=${!canWalk || !this.playing}>|| Pause</button>
        <button id="step" @click=${this.onStep} ?disabled=${!canWalk || this.busy || "problem" in walkSeed}>Step</button>
        <label>Speed
          <select id="speed" @change=${(e: Event) => (this.speed = (e.target as HTMLSelectElement).value as Speed)}>
            ${(Object.keys(SPEEDS) as Speed[]).map(
              (s) => html`<option value=${s} ?selected=${s === this.speed}>${s[0].toUpperCase() + s.slice(1)}</option>`,
            )}
          </select>
        </label>
        <button id="replay" @click=${() => this.startReplay()} ?disabled=${!this.canReplay}>Replay walk</button>
      </div>
      <div class="bar">
        Overlays:
        ${OVERLAYS.map(
          (o) => html`<button id="overlay-${o.id}" aria-pressed=${this.overlays[o.id] ? "true" : "false"} @click=${() => this.toggle(o.id)}>
            ${o.label} (${o.key})
          </button>`,
        )}
      </div>
      ${problems.length ? html`<p id="field-problem" class="problem">${problems.join(" ")}</p>` : nothing}
      ${this.renderReplayBar()}
      ${this.renderWorld()}
    `;
  }

  private renderReplayBar() {
    const replay = this.replay;
    if (!replay) return nothing;
    const ended = !!replay.check;
    return html`<div class="bar" id="timeline">
      <span id="replay-progress">${replayProgress(replay.model.patches.length, replay.total)}</span>
      <button id="replay-play" @click=${this.onPlay} ?disabled=${ended || this.playing}>&gt; Play</button>
      <button id="replay-pause" @click=${this.onPause} ?disabled=${!this.playing}>|| Pause</button>
      <button id="replay-step" @click=${this.onStep} ?disabled=${ended || this.busy}>Step forward</button>
      <button id="back-to-walk" @click=${() => this.backToWalk()}>Back to walk</button>
    </div>`;
  }

  private renderWorld() {
    const world = this.world;
    if (!world) {
      return html`<div class="map empty" style="width: ${VIEW_WIDTH * 2}ch; height: ${VIEW_HEIGHT * 1.2}em">
        <p class="note">Press New world to grow a world from this seed.</p>
      </div>`;
    }
    return html`<div class="body">
      <div class="stage">
        ${this.renderStatus(world)}
        <div class="map" tabindex="0" role="grid" aria-label="World ${world.seed}. Arrow keys walk, Shift+arrows pan."
          @keydown=${this.onMapKeydown}
          @mouseover=${this.onMapHover}
          @mouseleave=${() => (this.hover = undefined)}
          @click=${() => (this.playing = false)}
        >${this.renderRows()}</div>
      </div>
      ${this.renderSide()}
    </div>`;
  }

  private renderStatus(world: WorldInfo) {
    const replay = this.replay;
    if (replay?.check && this.queue.length === 0) {
      return html`<span id="status" class=${replay.check.same ? "" : "problem"} role="status">${checkText(replay.replayed, replay.check)}</span>`;
    }
    const parts = [worldStatus(world.seed, this.totals.steps, this.totals.patchCount, this.totals.storedCount)];
    if (this.reloadedNote) parts.push(this.reloadedNote);
    return html`<span id="status">${parts.join(" · ")}</span>`;
  }

  /**
   * The column beside the map: readout, Centre, the last patch, then the legend.
   * They sat under the map until Damson moved them here (2.6 step 3 review, fix 2),
   * because at the default 1024×768 window "Hedge to the north" fell below the fold.
   */
  private renderSide() {
    const model = this.model;
    const last = model.lastPatch;
    const of = this.replay ? this.replay.total : this.patchTotal;
    return html`<div class="side">
      <span id="readout" class="readout">${this.blocked ?? (this.hover ? this.cellText(this.hover.x, this.hover.y) : "")}</span>
      <span class="bar">
        <button id="centre" @click=${() => this.centre()} ?disabled=${this.follow}>Centre on player (C)</button>
        ${this.follow ? html`<span class="muted">Following the player</span>` : nothing}
      </span>
      <div class="patch-lines" id="patch-panel" aria-label="Last patch">
        ${last ? patchPanel(last, of, this.overlays.debug).map((line) => html`<span>${line}</span>`) : nothing}
        <span class="gap"></span>
        <span id="frontier-count">${frontierText(model.frontier.length, model.outwardCount)}</span>
      </div>
      <p class="legend muted">
        Legend: inverted cell = player · → outward endpoint · ○ open, not outward · dashed = last patch window ·
        dimmed = older patches · ${this.overlays.stored ? "blank" : "░"} = unsettled · dotted = origin
      </p>
    </div>`;
  }

  private renderRows() {
    const rows = [];
    for (let j = 0; j < VIEW_HEIGHT; j++) {
      const cells = [];
      for (let i = 0; i < VIEW_WIDTH; i++) cells.push(this.renderCell(this.corner.x + i, this.corner.y + j));
      rows.push(html`<span class="row" role="row">${cells}</span>`);
    }
    return rows;
  }

  private renderCell(x: number, y: number) {
    const model = this.model;
    const marks: string[] = [];
    const last = this.overlays.last ? model.lastPatch : undefined;
    if (last) {
      const w = last.window;
      const inside = x >= w.x && x < w.x + w.width && y >= w.y && y < w.y + w.height;
      if (inside) {
        if (y === w.y) marks.push("win-n");
        if (x === w.x + w.width - 1) marks.push("win-e");
        if (y === w.y + w.height - 1) marks.push("win-s");
        if (x === w.x) marks.push("win-w");
      }
    }
    if (x === 0 && y === 0) marks.push("origin");
    const player = !this.replay && model.player?.x === x && model.player?.y === y;
    if (player) marks.push("player");

    const label = this.cellText(x, y);
    const stored = model.cellAt(x, y);
    if (stored) {
      const tile = this.tileOf(stored.tile);
      const look = tile ? tileLook(tile.id, tile.orientation) : { glyph: "?", tint: "none" };
      const tint = look.tint === "none" || player ? "" : `tint-${look.tint}`;
      if (last && stored.patch < last.index) marks.push("older");
      return html`<span class="cell ${tile?.kind ?? ""} ${tint} ${marks.join(" ")}" role="gridcell" data-x=${x} data-y=${y} aria-label=${label}
        >${look.glyph}${eastFiller(look.glyph)}</span
      >`;
    }
    const endpoint = this.overlays.frontier ? model.endpointAt(x, y) : undefined;
    if (endpoint) {
      marks.push("mark");
      if (!this.overlays.stored) marks.push("hedge");
      return html`<span class="cell ${marks.join(" ")}" role="gridcell" data-x=${x} data-y=${y} aria-label=${label}
        >${endpointMark(endpoint.facing, endpoint.outward)} </span
      >`;
    }
    // Unsettled: solid hedge, what a player would see; blank under Stored (DR-0017 rule 6).
    if (this.overlays.stored) {
      return html`<span class="cell ${marks.join(" ")}" role="gridcell" data-x=${x} data-y=${y} aria-label=${label}>  </span>`;
    }
    return html`<span class="cell hedge ${marks.join(" ")}" role="gridcell" data-x=${x} data-y=${y} aria-label=${label}>░░</span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "world-view": WorldView;
  }
}
