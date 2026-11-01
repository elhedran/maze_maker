import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { errorText, startGame, step, type Direction, type GameView } from "./ipc.ts";
import { DEFAULT_SEED, directionForKey, parseSeed } from "./controls.ts";
import { cellLabel, layoutGrid, type ScreenCell } from "./world-grid.ts";

/** How long the "blocked" cue stays up after a move that went nowhere. */
export const BLOCKED_CUE_MS = 600;

/**
 * The game screen (G.1): a header with the seed and position, a seed field
 * with New world, and the world around the player drawn with DR-0017's
 * glyphs. Rust grows and stores the world (DR-0035, DR-0036); this only
 * sends moves and draws what comes back. What the player sees follows DR-0044; the look is a placeholder until DR-0045.
 */
@customElement("game-app")
export class GameApp extends LitElement {
  @state() private seedText = String(DEFAULT_SEED);
  /** The seed of the world on screen, which the field may no longer match. */
  @state() private worldSeed?: number;
  @state() private view?: GameView;
  @state() private error?: string;
  @state() private blocked = false;

  /**
   * One command at a time. Keys pressed while a step is in flight are
   * dropped rather than queued, so holding a key can't build a backlog that
   * keeps walking after it's released. Holding a key still walks, at the OS key-repeat rate, until DR-0045 sets a pace.
   */
  private busy = false;
  private blockedTimer?: ReturnType<typeof setTimeout>;

  static styles = css`
    /* DR-0017's colour tokens, copied from apps/tool/src/table/draw-tile.ts. */
    :host {
      --page: #ffffff;
      --text: #1b1d21;
      --muted: #5d636d;
      --corridor-line: #1d5fa6;
      --room-line: #8a4100;
      --room-tint: #f2d0a6;
      --hedge-bg: #eef2ec;
      --hedge-glyph: #6f8a6a;
      --frame: #c8ccd2;
      --problem: #b00020;
      --visited: rgba(27, 29, 33, 0.07);
      --mono: "Cascadia Mono", Consolas, monospace;
      color: var(--text);
      background: var(--page);
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      padding: 1rem 1.5rem;
      min-height: 100vh;
      box-sizing: border-box;
      font-family: system-ui, sans-serif;
      font-size: 14px;
    }
    @media (prefers-color-scheme: dark) {
      :host {
        --page: #15171c;
        --text: #e4e6ea;
        --muted: #9aa0aa;
        --corridor-line: #7cb8f2;
        --room-line: #f2a65a;
        --room-tint: #4a3420;
        --hedge-bg: #1c241d;
        --hedge-glyph: #5a785a;
        --frame: #3a3f48;
        --problem: #ff8a95;
        --visited: rgba(228, 230, 234, 0.08);
      }
    }
    header { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.5rem 1.5rem; }
    h1 { font-size: 1.25rem; margin: 0; }
    .muted { color: var(--muted); }
    .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
    .problem { color: var(--problem); margin: 0; white-space: pre-wrap; min-height: 1.2em; }
    input, button {
      font: inherit;
      padding: 0.2rem 0.5rem;
      border: 1px solid var(--frame);
      background: var(--page);
      color: var(--text);
    }
    input { font-family: var(--mono); width: 9rem; }
    button { border-radius: 3px; cursor: pointer; }
    button:disabled { cursor: default; opacity: 0.6; }
    /* The map: DR-0017 rules 2 and 5, as the tool's Maze view draws it. */
    .map {
      font-family: var(--mono);
      font-size: 18px;
      font-variant-ligatures: none;
      font-synthesis: none;
      white-space: pre;
      border: 1px solid var(--frame);
      width: max-content;
    }
    /* A blocked move flashes the frame; the text cue beside it says why. */
    .map.blocked { border-color: var(--problem); }
    .row { display: block; }
    .cell { background-repeat: no-repeat; }
    .cell.corridor { color: var(--corridor-line); }
    .cell.room { color: var(--room-line); }
    .cell.hedge { color: var(--hedge-glyph); background-color: var(--hedge-bg); }
    /* Visited: a faint wash, under the tint so rooms keep their floor. */
    .cell.visited { background-color: var(--visited); }
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
    /* The player: '@' in the text colour and bold, so it reads without colour. */
    .cell.player { color: var(--text); font-weight: bold; outline: 2px solid var(--text); outline-offset: -2px; }
    .cue { color: var(--problem); min-height: 1.4em; }
  `;

  connectedCallback(): void {
    super.connectedCallback();
    // On the window, not the element: keys work on load without a click.
    window.addEventListener("keydown", this.onKeydown);
    void this.start(DEFAULT_SEED);
  }

  disconnectedCallback(): void {
    window.removeEventListener("keydown", this.onKeydown);
    clearTimeout(this.blockedTimer);
    super.disconnectedCallback();
  }

  private async start(seed: number) {
    if (this.busy) return;
    this.busy = true;
    try {
      this.view = await startGame(seed);
      this.worldSeed = seed;
      this.error = undefined;
      this.clearBlocked();
    } catch (e) {
      this.error = errorText(e);
    } finally {
      this.busy = false;
    }
  }

  private async move(direction: Direction) {
    if (this.busy || !this.view) return;
    this.busy = true;
    try {
      const view = await step(direction);
      this.view = view;
      this.error = undefined;
      if (view.moved) this.clearBlocked();
      else this.showBlocked();
    } catch (e) {
      this.error = errorText(e);
    } finally {
      this.busy = false;
    }
  }

  private showBlocked() {
    clearTimeout(this.blockedTimer);
    this.blocked = true;
    this.blockedTimer = setTimeout(() => (this.blocked = false), BLOCKED_CUE_MS);
  }

  private clearBlocked() {
    clearTimeout(this.blockedTimer);
    this.blocked = false;
  }

  private onKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    // Typing a seed isn't walking.
    if (event.composedPath().some((t) => t instanceof HTMLInputElement)) return;
    const direction = directionForKey(event.key);
    if (!direction) return;
    event.preventDefault(); // Arrow keys would scroll the page.
    void this.move(direction);
  };

  private onNewWorld() {
    const parsed = parseSeed(this.seedText);
    if ("seed" in parsed) void this.start(parsed.seed);
  }

  private onSeedKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      this.onNewWorld();
    }
  }

  render() {
    const parsed = parseSeed(this.seedText);
    const problem = "problem" in parsed ? parsed.problem : undefined;
    const view = this.view;
    return html`
      <header>
        <h1>Maze Maker</h1>
        <span id="status" class="muted">
          ${view ? `Seed ${this.worldSeed} · x ${view.player.x}, y ${view.player.y}` : "Growing the world…"}
        </span>
      </header>
      <div class="bar">
        <label>Seed
          <input
            id="seed"
            inputmode="numeric"
            autocomplete="off"
            .value=${this.seedText}
            aria-invalid=${problem ? "true" : "false"}
            aria-describedby="seed-problem"
            @input=${(e: Event) => (this.seedText = (e.target as HTMLInputElement).value)}
            @keydown=${this.onSeedKeydown}
          />
        </label>
        <button id="new-world" @click=${this.onNewWorld} ?disabled=${!!problem}>New world</button>
        <span class="muted">Arrow keys or WASD to walk.</span>
      </div>
      <p id="seed-problem" class="problem">${problem ?? ""}</p>
      ${this.error ? html`<p class="problem" id="error" role="alert">${this.error}</p>` : nothing}
      ${view ? this.renderMap(view) : nothing}
      <span id="cue" class="cue" role="status">${this.blocked ? "Blocked: no way through there." : ""}</span>
    `;
  }

  private renderMap(view: GameView) {
    return html`<div
      class="map ${this.blocked ? "blocked" : ""}"
      role="img"
      aria-label="The world around you, seed ${this.worldSeed}"
    >${layoutGrid(view).map((row) => html`<span class="row">${row.map(renderCell)}</span>`)}</div>`;
  }
}

function renderCell(c: ScreenCell) {
  const classes = [
    "cell",
    c.cell?.kind ?? "ungrown",
    c.tint === "none" ? "" : `tint-${c.tint}`,
    c.cell?.visited ? "visited" : "",
    c.player ? "player" : "",
  ].join(" ");
  return html`<span class=${classes} data-x=${c.x} data-y=${c.y} aria-label=${cellLabel(c)}>${c.text}</span>`;
}

declare global {
  interface HTMLElementTagNameMap {
    "game-app": GameApp;
  }
}
