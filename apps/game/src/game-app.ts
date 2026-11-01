import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { keyed } from "lit/directives/keyed.js";
import { errorText, focusWindow, initialSeed, startGame, step, type Direction, type GameView, type Pos } from "./ipc.ts";
import { directionForKey, randomSeed } from "./controls.ts";
import { statusText } from "./status-text.ts";
import { themeStyles } from "./theme.ts";
import { Walker, type StepOutcome } from "./walker.ts";
import { cellLabel, layoutGrid, type ScreenCell } from "./world-grid.ts";

/** The world on screen and where it started, for the status line. */
interface World {
  seed: number;
  start: Pos;
  view: GameView;
  /** Moves that happened; a blocked move doesn't count (DR-0045 point 1). */
  steps: number;
}

/**
 * The game screen (DR-0045): a status line, the world around the player
 * drawn with DR-0017's glyphs, and a message line. Rust grows and stores
 * the world (DR-0035, DR-0036) and decides every move (DR-0044); this sends
 * steps and draws what comes back.
 */
@customElement("game-app")
export class GameApp extends LitElement {
  @state() private world?: World;
  @state() private starting = true;
  @state() private error?: string;
  /** "Can't go north." after a bump; cleared by the next move that happens. */
  @state() private message = "";
  /** The last bump, so the `@` nudges that way; `bumps` replays it each time. */
  @state() private nudge?: Direction;
  @state() private bumps = 0;

  /** Bumped by every new world, so a step answered after it is ignored. */
  private worldSerial = 0;
  private walker = new Walker((d) => this.walk(d));

  static styles = [
    themeStyles,
    css`
      :host {
        /* DR-0045's own tokens: the unknown haze (point 7) and the trail (point 6). */
        --unknown: #d5d9df;
        --trail: rgba(160, 120, 40, 0.1);
        display: flex;
        flex-direction: column;
        gap: 1rem;
        padding: 1rem 1.5rem;
        min-height: 100vh;
        box-sizing: border-box;
        font-family: system-ui, sans-serif;
        font-size: 16px;
      }
      @media (prefers-color-scheme: dark) {
        :host {
          --unknown: #0b0c0f;
          --trail: rgba(242, 166, 90, 0.08);
        }
      }
      .status { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.5rem 1rem; }
      .muted { color: var(--muted); }
      .problem { color: var(--problem); margin: 0; white-space: pre-wrap; }
      button {
        font: inherit;
        padding: 0.2rem 0.75rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      .stage { flex: 1; display: flex; align-items: center; justify-content: center; }
      /* The map: DR-0017 rules 2 and 5 at 32px (DR-0045 point 2); the frame is the edge of sight. */
      .map {
        font-family: var(--mono);
        font-size: 32px;
        font-variant-ligatures: none;
        font-synthesis: none;
        white-space: pre;
        border: 1px solid var(--frame);
        width: max-content;
      }
      .row { display: block; }
      .cell { background-repeat: no-repeat; }
      .cell.corridor { color: var(--corridor-line); }
      .cell.room { color: var(--room-line); }
      .cell.hedge { color: var(--hedge-glyph); background-color: var(--hedge-bg); }
      .cell.unknown { background-color: var(--unknown); }
      /*
       * Floor tint (DR-0017 rule 4). A cell is two characters and a wall glyph
       * sits on the first, so the tint covers the glyph's floor side of it:
       * a quarter of the cell west of a ║, three quarters east of one. The
       * game draws this differently from the tool (Damson, DR-0045 review).
       */
      .cell[class*="tint-"] { background-image: linear-gradient(var(--room-tint), var(--room-tint)); }
      .tint-full { background-size: 100% 100%; }
      .tint-lower { background-size: 100% 50%; background-position: bottom; }
      .tint-upper { background-size: 100% 50%; background-position: top; }
      .tint-left { background-size: 25% 100%; background-position: left; }
      .tint-right { background-size: 75% 100%; background-position: right; }
      .tint-lower-left { background-size: 25% 50%; background-position: left bottom; }
      .tint-upper-left { background-size: 25% 50%; background-position: left top; }
      .tint-upper-right { background-size: 75% 50%; background-position: right top; }
      .tint-lower-right { background-size: 75% 50%; background-position: right bottom; }
      /* The trail: an inset shadow paints over the background and tint, under the glyph. */
      .cell.visited { box-shadow: inset 0 0 0 100vmax var(--trail); }
      /* Rooms keep their floor tint clean: the trail is for corridors. */
      .cell.room.visited { box-shadow: none; }
      /* The player: a bold @ in the text colour (DR-0045 point 4). */
      .at { display: inline-block; color: var(--text); font-weight: bold; }
      .at.nudge-north { animation: nudge-north 120ms ease-out; }
      .at.nudge-east { animation: nudge-east 120ms ease-out; }
      .at.nudge-south { animation: nudge-south 120ms ease-out; }
      .at.nudge-west { animation: nudge-west 120ms ease-out; }
      @keyframes nudge-north { 50% { transform: translateY(-3px); } }
      @keyframes nudge-east { 50% { transform: translateX(3px); } }
      @keyframes nudge-south { 50% { transform: translateY(3px); } }
      @keyframes nudge-west { 50% { transform: translateX(-3px); } }
      @media (prefers-reduced-motion: reduce) {
        .at[class*="nudge-"] { animation: none; }
      }
      .messages { display: flex; justify-content: space-between; gap: 1rem; min-height: 1.4em; }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    // On the window, not the element: keys work from launch with no click.
    window.addEventListener("keydown", this.onKeydown);
    window.addEventListener("keyup", this.onKeyup);
    // A key let go in another window never sends its keyup here.
    window.addEventListener("blur", this.onBlur);
    void focusWindow();
    void this.launch();
  }

  disconnectedCallback(): void {
    window.removeEventListener("keydown", this.onKeydown);
    window.removeEventListener("keyup", this.onKeyup);
    window.removeEventListener("blur", this.onBlur);
    this.walker.releaseAll();
    super.disconnectedCallback();
  }

  private async launch() {
    try {
      await this.start(await initialSeed());
    } catch (e) {
      this.error = errorText(e);
      this.starting = false;
    }
  }

  private async start(seed: number) {
    const serial = ++this.worldSerial;
    this.walker.releaseAll();
    this.starting = true;
    try {
      const view = await startGame(seed);
      if (serial !== this.worldSerial) return;
      this.world = { seed, start: view.player, view, steps: 0 };
      this.error = undefined;
      this.message = "";
      this.nudge = undefined;
    } catch (e) {
      if (serial !== this.worldSerial) return;
      this.error = errorText(e);
    } finally {
      if (serial === this.worldSerial) this.starting = false;
    }
  }

  /** One step, for the walker. Rust decides; this only records the answer. */
  private async walk(direction: Direction): Promise<StepOutcome> {
    if (!this.world || this.starting) return "failed";
    const serial = this.worldSerial;
    try {
      const view = await step(direction);
      if (serial !== this.worldSerial || !this.world) return "failed";
      this.error = undefined;
      if (view.moved) {
        this.world = { ...this.world, view, steps: this.world.steps + 1 };
        this.message = "";
        this.nudge = undefined;
        return "moved";
      }
      this.world = { ...this.world, view };
      this.message = `Can't go ${direction}.`;
      this.nudge = direction;
      this.bumps++;
      return "blocked";
    } catch (e) {
      if (serial === this.worldSerial) this.error = errorText(e);
      return "failed";
    }
  }

  private onKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    // Keys in a text field are typing, if one is ever added.
    if (event.composedPath().some((t) => t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement)) return;
    const direction = directionForKey(event.key);
    if (!direction) return;
    event.preventDefault(); // Arrow keys would scroll the page.
    // The game sets the pace of a held key, not the OS repeat.
    if (!event.repeat && this.world && !this.starting) this.walker.press(direction);
  };

  private onKeyup = (event: KeyboardEvent) => {
    const direction = directionForKey(event.key);
    if (direction) this.walker.release(direction);
  };

  private onBlur = () => this.walker.releaseAll();

  private onNewWorld(event: Event) {
    // Give up focus so Space and Enter can't press it again (DR-0045 point 1).
    (event.currentTarget as HTMLElement).blur();
    void this.start(randomSeed());
  }

  render() {
    const world = this.world;
    return html`
      <div class="status">
        <span id="status">
          ${world && !this.starting
            ? statusText(world.seed, world.start, world.view.player, world.steps)
            : "Growing the world…"}
        </span>
        <button id="new-world" @click=${this.onNewWorld}>New world</button>
      </div>
      ${this.error ? html`<p class="problem" id="error" role="alert">${this.error}</p>` : nothing}
      <div class="stage">${world && !this.starting ? this.renderMap(world.view) : nothing}</div>
      <div class="messages">
        <span id="message" class="muted" aria-live="polite">${this.message}</span>
        <span class="muted">Arrows or WASD to walk</span>
      </div>
    `;
  }

  private renderMap(view: GameView) {
    // Flush against the tags: the map is white-space: pre, so any newline
    // or indent here would draw as a blank band (Damson's DR-0045 review).
    const rows = layoutGrid(view).map((row) => html`<span class="row">${row.map((c) => this.renderCell(c))}</span>`);
    return html`<div class="map" role="img" aria-label="World around you">${rows}</div>`;
  }

  private renderCell(c: ScreenCell) {
    const classes = [
      "cell",
      c.cell?.kind ?? (c.sealed ? "hedge sealed" : "unknown"),
      c.tint === "none" ? "" : `tint-${c.tint}`,
      c.cell?.visited ? "visited" : "",
      c.player ? "player" : "",
    ].join(" ");
    // `keyed` remakes the @ on each bump, so its nudge animation plays again.
    const glyph = c.player
      ? keyed(this.bumps, html`<span class="at ${this.nudge ? `nudge-${this.nudge}` : ""}">@</span>`)
      : c.glyph;
    return html`<span class=${classes} data-x=${c.x} data-y=${c.y} aria-label=${cellLabel(c)}>${glyph}${c.filler}</span>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "game-app": GameApp;
  }
}
