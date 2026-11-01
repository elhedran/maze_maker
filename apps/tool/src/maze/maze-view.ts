import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { errorText, generateMaze, tableView, tilesOnlyRequest } from "../ipc.ts";
import { themeStyles } from "../table/draw-tile.ts";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";
import {
  MAZE_HEIGHT,
  MAZE_WIDTH,
  eastFiller,
  failureText,
  hoverText,
  parseSeed,
  randomSeed,
  statusText,
} from "./maze-text.ts";

/**
 * The generation view (DR-0029): a seed, Generate, one maze drawn with
 * DR-0017's glyphs two characters per cell, a hover readout and Wren's
 * measures (DR-0030). Rust generates and measures; this only draws.
 */
@customElement("maze-view")
export class MazeView extends LitElement {
  /** What's typed in the seed field. It opens on a random seed so Generate works at once. */
  @state() private seedText = String(randomSeed());
  /** The maze on screen, or the failed attempt. Its seed is the one the status line shows. */
  @state() private maze?: GeneratedMaze;
  /** The tiles `maze.cells` index into, from the same tile set revision. */
  @state() private tiles?: TileView[];
  @state() private afterReload = false;
  @state() private generating = false;
  /** A command that failed outright (not a maze that gave up). The last maze stays. */
  @state() private error?: string;
  @state() private hover?: { x: number; y: number };

  private tilesRevision?: number;
  /** Ignores a response that arrives after a newer request was sent. */
  private requestSerial = 0;

  static styles = [
    themeStyles,
    css`
      :host {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
      .muted { color: var(--muted); }
      .problem { color: var(--problem); margin: 0; white-space: pre-wrap; }
      input {
        font: inherit;
        font-family: var(--mono);
        width: 9rem;
        padding: 0.2rem 0.4rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
      }
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
      .body { display: flex; gap: 1.5rem; align-items: flex-start; }
      .stage { display: flex; flex-direction: column; gap: 0.25rem; }
      /* The map: DR-0017 rules 2 and 5, natural line height and no frames. */
      .map {
        font-family: var(--mono);
        font-size: 18px;
        font-variant-ligatures: none;
        font-synthesis: none;
        white-space: pre;
        border: 1px solid var(--frame);
        width: max-content;
      }
      .map.dimmed { opacity: 0.4; }
      .map.empty {
        display: flex;
        align-items: center;
        justify-content: center;
        box-sizing: border-box;
      }
      .note {
        white-space: normal;
        text-align: center;
        font-family: system-ui, sans-serif;
        font-size: 14px;
        padding: 1rem;
        margin: 0;
      }
      .row { display: block; }
      .cell { background-repeat: no-repeat; }
      .cell.corridor { color: var(--corridor-line); }
      .cell.room { color: var(--room-line); }
      .cell.hedge { color: var(--hedge-glyph); background-color: var(--hedge-bg); }
      /* Floor tint on the floor side only (DR-0017 rule 4), over both characters. */
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
      /* The centre DR-0030 counts from: text colour, never red or magenta (DR-0029 point 9). */
      .cell.centre { outline: 2px solid var(--text); outline-offset: -2px; }
      .readout { min-height: 1.4em; }
      .measures { flex: 0 0 20rem; display: flex; flex-direction: column; gap: 0.75rem; }
      .measures p { margin: 0; }
      .centre-key {
        display: inline-block;
        width: 0.9em;
        height: 0.9em;
        vertical-align: middle;
        outline: 2px solid var(--text);
        outline-offset: -2px;
      }
    `,
  ];

  /**
   * Called by the app after a successful tile set reload (DR-0029 point 3):
   * if a maze is shown, make it again from the same seed and new data.
   */
  regenerateAfterReload(): void {
    if (this.maze) void this.generate(this.maze.seed, true);
  }

  private async generate(seed: number, afterReload = false) {
    const serial = ++this.requestSerial;
    this.generating = true;
    try {
      const maze = await generateMaze({ seed, width: MAZE_WIDTH, height: MAZE_HEIGHT });
      if (maze.revision !== this.tilesRevision) {
        const view = await tableView(tilesOnlyRequest());
        if (serial !== this.requestSerial) return;
        this.tiles = view.tiles;
        this.tilesRevision = view.revision;
      }
      if (serial !== this.requestSerial) return;
      this.maze = maze;
      this.afterReload = afterReload;
      this.error = undefined;
    } catch (e) {
      if (serial !== this.requestSerial) return;
      this.error = errorText(e);
    } finally {
      if (serial === this.requestSerial) this.generating = false;
    }
  }

  private onGenerate() {
    const parsed = parseSeed(this.seedText);
    if ("seed" in parsed) void this.generate(parsed.seed);
  }

  private onRandomSeed() {
    const seed = randomSeed();
    this.seedText = String(seed);
    void this.generate(seed);
  }

  private onSeedKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !this.generating) {
      event.preventDefault();
      this.onGenerate();
    }
  }

  private onMapHover(event: Event) {
    const cell = (event.target as HTMLElement).closest<HTMLElement>(".cell");
    if (!cell) return;
    const x = Number(cell.dataset.x);
    const y = Number(cell.dataset.y);
    if (this.hover?.x !== x || this.hover?.y !== y) this.hover = { x, y };
  }

  render() {
    const parsed = parseSeed(this.seedText);
    const problem = "problem" in parsed ? parsed.problem : undefined;
    return html`
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
        <button id="generate" @click=${this.onGenerate} ?disabled=${!!problem || this.generating}>
          ${this.generating ? "Generating…" : "Generate"}
        </button>
        <button id="random-seed" @click=${this.onRandomSeed} ?disabled=${this.generating}>Random seed</button>
      </div>
      <p id="seed-problem" class="problem">${problem ?? ""}</p>
      ${this.error ? html`<p class="problem error" role="alert">${this.error}</p>` : nothing}
      ${this.renderMaze()}
    `;
  }

  private renderMaze() {
    const maze = this.maze;
    if (!maze) {
      return html`<div class="map empty" style=${boxStyle(MAZE_WIDTH, MAZE_HEIGHT)}>
        <p class="note">Press Generate to make a maze from this seed.</p>
      </div>`;
    }
    if (maze.failed) {
      return html`<div class="map empty" style=${boxStyle(maze.width, maze.height)}>
        <p class="note problem failed" role="alert">${failureText(maze)}</p>
      </div>`;
    }
    return html`<div class="body">
      <div class="stage">
        <span id="status">${statusText(maze, this.afterReload)}</span>
        <div
          class="map ${this.generating ? "dimmed" : ""}"
          role="img"
          aria-label="Maze, seed ${maze.seed}"
          @mouseover=${this.onMapHover}
          @mouseleave=${() => (this.hover = undefined)}
        >${this.renderRows(maze)}</div>
        <span id="readout" class="readout">${this.readout(maze)}</span>
      </div>
      ${this.renderMeasures(maze)}
    </div>`;
  }

  private renderRows(maze: GeneratedMaze) {
    const rows = [];
    for (let y = 0; y < maze.height; y++) {
      const cells = [];
      for (let x = 0; x < maze.width; x++) {
        const centre = maze.centre.x === x && maze.centre.y === y;
        cells.push(this.renderCell(this.tileAt(maze, x, y), x, y, centre));
      }
      rows.push(html`<span class="row">${cells}</span>`);
    }
    return rows;
  }

  /** The tile at a cell, or undefined for an open cell (or an index outside the tile list). */
  private tileAt(maze: GeneratedMaze, x: number, y: number): TileView | undefined {
    const index = maze.cells[y * maze.width + x];
    return index === null || index === undefined ? undefined : this.tiles?.[index];
  }

  private renderCell(tile: TileView | undefined, x: number, y: number, centre: boolean) {
    const mark = centre ? "centre" : "";
    if (!tile) {
      // Blank means "not generated yet" (DR-0017 rule 6).
      return html`<span class="cell ${mark}" data-x=${x} data-y=${y} aria-label="Not generated">  </span>`;
    }
    const look = tileLook(tile.id, tile.orientation);
    const tint = look.tint === "none" ? "" : `tint-${look.tint}`;
    return html`<span class="cell ${tile.kind} ${tint} ${mark}" data-x=${x} data-y=${y} aria-label=${tileLabel(tile)}
      >${look.glyph}${eastFiller(look.glyph)}</span
    >`;
  }

  private readout(maze: GeneratedMaze): string {
    if (!this.hover) return "";
    const { x, y } = this.hover;
    const tile = this.tileAt(maze, x, y);
    return hoverText(tile ? tileLabel(tile) : "Not generated", x, y);
  }

  private renderMeasures(maze: GeneratedMaze) {
    return html`<div class="measures" aria-label="Measures">
      ${maze.measures.map(
        (m) => html`<div class="measure">
          <p><strong>${m.name}: ${m.value}</strong></p>
          <p class="muted">${m.explanation}</p>
        </div>`,
      )}
      <p class="muted"><span class="centre-key"></span> centre</p>
    </div>`;
  }
}

/**
 * A box about the size the maze will be: two map characters by one line per
 * cell. `ch` is the map's monospace width; 1.2em approximates its natural line height.
 */
function boxStyle(width: number, height: number): string {
  return `width: ${width * 2}ch; height: ${height * 1.2}em`;
}

declare global {
  interface HTMLElementTagNameMap {
    "maze-view": MazeView;
  }
}
