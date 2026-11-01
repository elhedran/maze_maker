import { LitElement, css, html, nothing } from "lit";
import { customElement, state } from "lit/decorators.js";
import { compareMazes, errorText, generateMaze, tableView, tilesOnlyRequest } from "../ipc.ts";
import { themeStyles } from "../table/draw-tile.ts";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import { openExplore } from "../explore/open-explore.ts";
import { NOT_PLACED } from "../explore/why-text.ts";
import type { ExceptionView } from "../bindings/ExceptionView.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";
import type { CompareResult, TableView } from "../pending-bindings.ts";
import { MAZE_HEIGHT, MAZE_WIDTH, failureText, hoverText, parseSeed, randomSeed, statusText } from "./maze-text.ts";
import { boxStyle, cellAt, mapStyles, renderRows, tileAt } from "./maze-cells.ts";
import {
  BEFORE_FIXED,
  COMPARE_STATUS,
  SPREAD,
  afterLabel,
  beforeLabel,
  changedText,
  measureRows,
} from "./compare-text.ts";
import { PRESSURE_HINT, PRESSURE_MEANING, editLabel, pressureText, reachText } from "../edit/edit-text.ts";
import "../edit/number-editor.ts";

/** The loaded tile set, as far as this view needs it. */
interface Loaded {
  revision: number;
  loadedAt: string;
  tiles: TileView[];
  exceptions: ExceptionView[];
  /** DR-0038: 1 when the key is absent. */
  openingPressure: number;
}

/** DR-0037 points 8 to 13: the same seed before and after the last load that changed the tile set. */
interface Compare {
  result: CompareResult & { before: NonNullable<CompareResult["before"]> };
  changed: Set<number>;
  afterLabel: string;
}

/**
 * The generation view (DR-0029): a seed, Generate, one maze drawn with
 * DR-0017's glyphs two characters per cell, a hover readout and Wren's
 * measures (DR-0030). After a change to the tile set it shows the same seed
 * before and after, side by side (DR-0037). Opening pressure sits under the
 * seed row (DR-0037 point 14). Rust generates, compares and measures; this
 * only draws.
 */
@customElement("maze-view")
export class MazeView extends LitElement {
  /** What's typed in the seed field. It opens on a random seed so Generate works at once. */
  @state() private seedText = String(randomSeed());
  /** The maze on screen, or the failed attempt; in compare, the after maze. */
  @state() private maze?: GeneratedMaze;
  /** The loaded tile set; `maze.cells` index into its tiles when the revisions match. */
  @state() private loaded?: Loaded;
  @state() private afterReload = false;
  @state() private generating = false;
  /** A command that failed outright (not a maze that gave up). The last maze stays. */
  @state() private error?: string;
  @state() private hover?: { x: number; y: number; before?: boolean };
  /**
   * The cell picked for "Why is this here?" (DR-0033 points 1 to 4), by
   * column and row: kept across tabs and Reload's regeneration, whatever
   * tile is there now; cleared by Generate and Random seed.
   */
  @state() private selected?: { x: number; y: number };
  @state() private compare?: Compare;
  /** DR-0037 point 11: fade cells that are the same in both mazes. */
  @state() private fade = true;
  @state() private editingPressure = false;

  /** Ignores a response that arrives after a newer request was sent. */
  private requestSerial = 0;

  static styles = [
    themeStyles,
    mapStyles,
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
      button.number {
        padding: 0;
        border: none;
        background: none;
        text-decoration: underline dotted;
        text-underline-offset: 3px;
      }
      .pressure { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem; }
      .pressure number-editor { flex-basis: 100%; max-width: 32rem; }
      .body { display: flex; gap: 1.5rem; align-items: flex-start; }
      .stage { display: flex; flex-direction: column; gap: 0.25rem; }
      .map .cell { cursor: pointer; }
      .selected-line { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; min-height: 1.8em; }
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

      /* Compare, DR-0037 point 10: side by side at DR-0017's 16px floor. */
      .status-line { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
      .sides { display: flex; gap: 1rem; align-items: flex-start; }
      .side { display: flex; flex-direction: column; gap: 0.25rem; }
      .side-label { font-size: 12px; font-weight: 600; letter-spacing: 0.04em; }
      .sides .map { font-size: 16px; }
      .side.before .map .cell { cursor: default; }
      /* Point 11: what didn't change drops back, so what did stands at full strength. */
      .cell.same { opacity: 0.4; }
      /* Point 10: the cell hovered in the other maze. */
      .cell.twin { outline: 1px solid var(--text); outline-offset: -1px; }
      .compare-measures { border-collapse: collapse; font-variant-numeric: tabular-nums; }
      .compare-measures th { font-weight: normal; color: var(--muted); text-align: left; padding: 0.1rem 0.75rem 0.1rem 0; }
      .compare-measures td { padding: 0.1rem 0.75rem 0.1rem 0; }
      .compare-measures .arrow { color: var(--muted); }
      p.count, p.spread { margin: 0; }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    // The opening pressure shows before any maze does.
    void this.loadTileSet().catch(() => {
      // The app's banner already reports a tile set that won't load.
    });
  }

  /** True when a maze is on screen, so a change has a before and after (DR-0037 point 7). */
  get hasMaze(): boolean {
    return !!this.maze;
  }

  /**
   * Called by the app after any successful load, Reload or a save (DR-0029
   * point 3, DR-0037 point 8). With a maze shown, Rust makes the same seed
   * on the old and the new tile set, and the two show side by side. With no
   * previous load there's no before, and the maze just regenerates.
   */
  showChange(): void {
    const maze = this.maze;
    if (!maze) {
      void this.loadTileSet().catch((e: unknown) => (this.error = errorText(e)));
      return;
    }
    void this.runCompare(maze.seed, maze.width, maze.height);
  }

  private async loadTileSet(): Promise<Loaded> {
    const view = (await tableView(tilesOnlyRequest())) as TableView;
    const loaded: Loaded = {
      revision: view.revision,
      loadedAt: view.loadedAt,
      tiles: view.tiles,
      exceptions: view.exceptions,
      openingPressure: view.openingPressure ?? 1,
    };
    this.loaded = loaded;
    return loaded;
  }

  /** The loaded tile set at `revision`, fetching it if this view holds another. */
  private async loadedAt(revision: number): Promise<Loaded> {
    return this.loaded?.revision === revision ? this.loaded : this.loadTileSet();
  }

  private async runCompare(seed: number, width: number, height: number) {
    const serial = ++this.requestSerial;
    this.generating = true;
    try {
      const result = await compareMazes({ seed, width, height });
      const loaded = await this.loadedAt(result.after.revision);
      if (serial !== this.requestSerial) return;
      this.maze = result.after;
      this.error = undefined;
      const before = result.before;
      this.compare = before
        ? {
            result: { ...result, before },
            changed: new Set(result.changed),
            afterLabel: afterLabel(result.edits, loaded, loaded.loadedAt),
          }
        : undefined;
      this.afterReload = !before;
    } catch (e) {
      if (serial !== this.requestSerial) return;
      this.error = errorText(e);
    } finally {
      if (serial === this.requestSerial) this.generating = false;
    }
  }

  private async generate(seed: number) {
    const serial = ++this.requestSerial;
    this.generating = true;
    try {
      const maze = await generateMaze({ seed, width: MAZE_WIDTH, height: MAZE_HEIGHT });
      await this.loadedAt(maze.revision);
      if (serial !== this.requestSerial) return;
      this.maze = maze;
      this.compare = undefined;
      this.afterReload = false;
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
    if (!("seed" in parsed)) return;
    this.selected = undefined;
    void this.generate(parsed.seed);
  }

  private onRandomSeed() {
    const seed = randomSeed();
    this.seedText = String(seed);
    this.selected = undefined;
    void this.generate(seed);
  }

  /** Point 8: back to DR-0029's single maze, the after one. */
  private closeCompare() {
    this.compare = undefined;
    this.hover = undefined;
  }

  private onMapClick(event: Event) {
    const at = cellAt(event);
    if (at) this.selected = at;
  }

  /** DR-0033 point 3: double-click opens, as [Why is this here?] does. */
  private onMapDoubleClick(event: Event) {
    const at = cellAt(event);
    if (!at) return;
    this.selected = at;
    this.openWhy();
  }

  /** The tile at a cell of the shown (after) maze. */
  private tileAt(maze: GeneratedMaze, x: number, y: number): TileView | undefined {
    return tileAt(maze, this.loaded?.revision === maze.revision ? this.loaded.tiles : undefined, x, y);
  }

  /**
   * Opens the Explore tab on the selected cell's tile, through DR-0026's
   * entry point, with the maze cell so the view can say why it's there.
   */
  private openWhy() {
    const maze = this.maze;
    const at = this.selected;
    const tile = maze && at ? this.tileAt(maze, at.x, at.y) : undefined;
    if (!maze || !at || !tile || this.generating) return;
    openExplore(this, {
      centre: tile,
      maze: { revision: maze.revision, seed: maze.seed, width: maze.width, height: maze.height, cell: at },
    });
  }

  private onSeedKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !this.generating) {
      event.preventDefault();
      this.onGenerate();
    }
  }

  private onMapHover(event: Event, before = false) {
    const at = cellAt(event);
    if (!at) return;
    const h = this.hover;
    if (h?.x !== at.x || h?.y !== at.y || !!h?.before !== before) this.hover = { ...at, before };
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
      ${this.renderPressure()}
      <p id="seed-problem" class="problem">${problem ?? ""}</p>
      ${this.error ? html`<p class="problem error" role="alert">${this.error}</p>` : nothing}
      ${this.compare && this.maze ? this.renderCompare(this.compare, this.maze) : this.renderMaze()}
    `;
  }

  /** DR-0037 point 14: "Opening pressure 0.5", editable like the panel's numbers. */
  private renderPressure() {
    const loaded = this.loaded;
    if (!loaded) return nothing;
    const target = { kind: "openingPressure" as const };
    const label = editLabel(target, loaded);
    return html`<div class="pressure">
      <span
        >${label}
        <button
          id="pressure"
          class="number"
          aria-label="Edit ${label}, ${pressureText(loaded.openingPressure)}"
          @click=${() => (this.editingPressure = true)}
        >
          ${pressureText(loaded.openingPressure)}
        </button></span
      >
      <span class="muted">${PRESSURE_MEANING}</span>
      ${this.editingPressure
        ? html`<number-editor
            .target=${target}
            .value=${loaded.openingPressure}
            label=${label}
            reach=${reachText(target, loaded)}
            hint=${PRESSURE_HINT}
            @number-saved=${() => (this.editingPressure = false)}
            @editor-close=${() => (this.editingPressure = false)}
          ></number-editor>`
        : nothing}
    </div>`;
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
        ${this.renderAfterMap(maze, "")}
        <span id="readout" class="readout">${this.readout(maze)}</span>
        ${this.renderSelectedLine(maze)}
      </div>
      ${this.renderMeasures(maze)}
    </div>`;
  }

  /** The current maze's map: clickable, selectable, the one "why" works on. */
  private renderAfterMap(maze: GeneratedMaze, extraMarks: string, same?: (i: number) => boolean) {
    return html`<div
      class="map ${this.generating ? "dimmed" : ""}"
      role="img"
      aria-label="Maze, seed ${maze.seed}"
      @mouseover=${(e: Event) => this.onMapHover(e)}
      @mouseleave=${() => (this.hover = undefined)}
      @click=${this.onMapClick}
      @dblclick=${this.onMapDoubleClick}
    >${renderRows(maze, this.loaded?.revision === maze.revision ? this.loaded.tiles : undefined, (x, y) =>
      [
        maze.centre.x === x && maze.centre.y === y ? "centre" : "",
        this.selected?.x === x && this.selected?.y === y ? "selected" : "",
        same?.(y * maze.width + x) ? "same" : "",
        extraMarks && this.hover?.before && this.hover.x === x && this.hover.y === y ? "twin" : "",
      ].join(" "),
    )}</div>`;
  }

  /** DR-0037 points 8 to 13: before on the left, after on the right, the measures under both. */
  private renderCompare(c: Compare, after: GeneratedMaze) {
    const before = c.result.before;
    const total = after.width * after.height;
    const fading = this.fade && c.changed.size > 0;
    const same = (i: number) => fading && !c.changed.has(i);
    return html`<div class="stage compare">
      <span class="status-line">
        <span id="status">Seed ${after.seed} · ${after.width}×${after.height} · ${COMPARE_STATUS}</span>
        <button id="close-compare" @click=${() => this.closeCompare()}>Close compare</button>
      </span>
      <div class="sides">
        <div class="side before">
          <span class="side-label" id="before-label">${beforeLabel(before.loadedAt)}</span>
          ${before.maze.failed
            ? html`<div class="map empty" style=${boxStyle(before.maze.width, before.maze.height)}>
                <p class="note problem failed">${failureText(before.maze)}</p>
              </div>`
            : html`<div
                class="map"
                role="img"
                aria-label="Before, seed ${before.maze.seed}"
                @mouseover=${(e: Event) => this.onMapHover(e, true)}
                @mouseleave=${() => (this.hover = undefined)}
              >${renderRows(before.maze, before.tiles, (x, y) =>
                [
                  before.maze.centre.x === x && before.maze.centre.y === y ? "centre" : "",
                  same(y * before.maze.width + x) ? "same" : "",
                  this.hover && !this.hover.before && this.hover.x === x && this.hover.y === y ? "twin" : "",
                ].join(" "),
              )}</div>`}
        </div>
        <div class="side after">
          <span class="side-label" id="after-label">${c.afterLabel}</span>
          ${after.failed
            ? html`<div class="map empty" style=${boxStyle(after.width, after.height)}>
                <p class="note problem failed">${failureText(after)}</p>
              </div>`
            : this.renderAfterMap(after, "twin", same)}
        </div>
      </div>
      <span id="readout" class="readout">${this.compareReadout(c, after)}</span>
      <p class="count">
        <span id="changed-count">${changedText(c.changed.size, total)}</span>
        ${c.changed.size > 0
          ? html`<label
              ><input
                type="checkbox"
                id="fade"
                .checked=${this.fade}
                @change=${(e: Event) => (this.fade = (e.target as HTMLInputElement).checked)}
              />
              Fade unchanged cells</label
            >`
          : nothing}
      </p>
      ${c.changed.size > 0 ? html`<p class="spread muted">${SPREAD}</p>` : nothing}
      <table class="compare-measures" aria-label="Measures before and after">
        <thead>
          <tr><th></th><th>BEFORE</th><th></th><th>AFTER</th></tr>
        </thead>
        <tbody>
          ${measureRows(before.maze, after).map(
            (m) => html`<tr>
              <th scope="row">${m.name}</th>
              <td>${m.before}</td>
              <td class="arrow">→</td>
              <td>${m.after}</td>
            </tr>`,
          )}
        </tbody>
      </table>
      ${after.failed ? nothing : this.renderSelectedLine(after, "Selected (after)")}
    </div>`;
  }

  private readout(maze: GeneratedMaze): string {
    if (!this.hover) return "";
    const { x, y } = this.hover;
    const tile = this.tileAt(maze, x, y);
    return hoverText(tile ? tileLabel(tile) : "Not generated", x, y);
  }

  /** Point 10: "col 7, row 1 · before: └ Corridor corner, … · after: ┌ Corridor corner, …". */
  private compareReadout(c: Compare, after: GeneratedMaze): string {
    const h = this.hover;
    if (!h) return "";
    const name = (tile: TileView | undefined) =>
      tile ? `${tileLook(tile.id, tile.orientation).glyph} ${tileLabel(tile)}` : "not generated";
    const before = tileAt(c.result.before.maze, c.result.before.tiles, h.x, h.y);
    const text = `col ${h.x}, row ${h.y} · before: ${name(before)} · after: ${name(this.tileAt(after, h.x, h.y))}`;
    return h.before ? `${text}. ${BEFORE_FIXED}` : text;
  }

  /**
   * DR-0033 point 2: "Selected: ═ Room wall, wall S · col 14, row 2
   * [Why is this here?]", or a prompt and a disabled button.
   */
  private renderSelectedLine(maze: GeneratedMaze, prefix = "Selected") {
    const at = this.selected;
    const tile = at && this.tileAt(maze, at.x, at.y);
    let text = "Click a cell, then ask why it's there.";
    if (at && tile) text = `${prefix}: ${tileLook(tile.id, tile.orientation).glyph} ${hoverText(tileLabel(tile), at.x, at.y)}`;
    else if (at) text = `${prefix}: col ${at.x}, row ${at.y}. ${NOT_PLACED}`;
    return html`<span class="selected-line">
      <span id="selected">${text}</span>
      <button id="why" ?disabled=${!tile || this.generating} @click=${() => this.openWhy()}>Why is this here?</button>
    </span>`;
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

declare global {
  interface HTMLElementTagNameMap {
    "maze-view": MazeView;
  }
}
