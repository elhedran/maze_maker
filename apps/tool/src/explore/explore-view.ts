import { LitElement, css, html, nothing, type PropertyValues } from "lit";
import { customElement, query, state } from "lit/decorators.js";
import { errorText } from "../ipc.ts";
import { reportToolError } from "../tool-error.ts";
import { glyphStyles, renderGlyph, shadeStyles, themeStyles } from "../table/draw-tile.ts";
import { DIRECTIONS, OPPOSITE, cellTitle, formatShare, headerTitle, shadeStep, sideName } from "../table/format.ts";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import type { Explained } from "../table/table-explanation.ts";
import "../table/table-explanation.ts";
import {
  loadExplore,
  loadWhy,
  sameTile,
  type ArmNeighbour,
  type Centre,
  type ExploreData,
  type MazeCell,
  type TileRef,
  type Why,
} from "./explore-data.ts";
import { BACK_TO_MAZE_EVENT, type ExploreTarget } from "./open-explore.ts";
import type { FactorKey } from "./why-band.ts";
import "./why-band.ts";
import { NOT_PLACED, RELOADED, afterReloadText, noMazeAfterReload } from "./why-text.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { TileView } from "../bindings/TileView.ts";

/** A neighbour picked in one arm. `from` marks the tile just stepped from (DR-0026 point 8). */
interface Selected {
  direction: Direction;
  neighbour: TileRef;
  from?: boolean;
}

/** One centre on the trail, with what was selected there, so Back restores both. */
interface Stop {
  centre: TileRef;
  selected?: Selected;
  /** On the first stop only, when the trail began at a maze cell (DR-0033 point 7). */
  maze?: MazeCell;
}

/** The "Why it's here" band: the cell it's about, and its trace or a note in its place. */
interface Band {
  maze: MazeCell;
  why?: Why;
  note?: string;
  /** After a reload: "after reload", or "after reload; was ═" (DR-0037 point 17). */
  suffix?: string;
}

/** Past centres shown before the trail is cut at its oldest end with "…" (DR-0026 point 9). */
export const TRAIL_SHOWN = 10;

const ref = (tile: TileRef): TileRef => ({ id: tile.id, orientation: tile.orientation });
const glyphOf = (tile: TileRef) => tileLook(tile.id, tile.orientation).glyph;

/**
 * Exploring one tile (DR-0026): the centre in the middle, an arm of
 * neighbours on each side sorted by share, the facing sides on the shared
 * edge, DR-0025's panel for the selected pair, [Go] to step and [< Back]
 * along the trail. Opened by the Centre picker or by `open`, the one entry
 * point (point 10). All data comes through `loadExplore`.
 */
@customElement("explore-view")
export class ExploreView extends LitElement {
  /** Centres visited since the last `open`, oldest first; the last is on screen. Never empty once opened. */
  @state() private trail: Stop[] = [];
  /** The last good response. Its centre can lag the trail while a step is in flight. */
  @state() private data?: ExploreData;
  /** Arms whose 0s are shown (point 6). Cleared when the centre changes. */
  @state() private openZeros = new Set<Direction>();
  /** The name of a centre the loaded tile set doesn't have (point 1). */
  @state() private missing?: string;
  @state() private busy = false;
  /** Set while the trail began at a maze cell; shown at the first stop only (DR-0033 point 9). */
  @state() private band?: Band;
  /** A factor selected in the band. It and an arm selection clear each other (point 13). */
  @state() private factor?: FactorKey;

  @query("#centre") private picker?: HTMLSelectElement;

  /** Ignores a response that arrives after a newer request was sent. */
  private requestSerial = 0;
  private whySerial = 0;

  static styles = [
    themeStyles,
    glyphStyles,
    shadeStyles,
    css`
      :host {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      .bar { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem 1rem; }
      .muted { color: var(--muted); }
      .hint { color: var(--muted); margin: 1rem 0; }
      .problem { color: var(--problem); margin: 0; }
      button, select {
        font: inherit;
        padding: 0.25rem 0.75rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      button:disabled { cursor: default; opacity: 0.6; }
      select { font-family: var(--mono), system-ui, sans-serif; padding: 0.2rem 0.4rem; }
      .trail { display: flex; flex-wrap: wrap; align-items: center; gap: 0.25rem; }
      .trail button { padding: 0; border: none; background: none; }
      .trail [aria-current="true"] .glyph { outline: 2px solid var(--text); outline-offset: 1px; }
      .trail .glyph { font-size: 16px; }

      .body { display: flex; gap: 1rem; align-items: flex-start; }
      /* The compass: North above, West left, South below, East right (point 3). */
      .compass {
        flex: 1 1 auto;
        min-width: 0;
        overflow-x: auto;
        display: grid;
        grid-template-columns: minmax(15rem, 24rem) auto minmax(15rem, 24rem);
        grid-template-areas:
          "north north north"
          "west centre east"
          "south south south";
        gap: 1rem 1.25rem;
        align-items: center;
      }
      .compass.busy { opacity: 0.6; }
      .arm { min-width: 15rem; max-width: 24rem; }
      .arm.north { grid-area: north; justify-self: center; align-self: end; }
      .arm.south { grid-area: south; justify-self: center; align-self: start; }
      .arm.west { grid-area: west; }
      .arm.east { grid-area: east; }
      .arm h3 { font-size: 13px; font-weight: 600; margin: 0 0 0.3rem; }
      .arm h3 .muted { font-weight: normal; }

      /* The centre box, with its side kind on each edge (point 4). */
      .centre {
        grid-area: centre;
        display: grid;
        grid-template-areas:
          ". n ."
          "w g e"
          ". s .";
        align-items: center;
        justify-items: center;
        gap: 0.25rem;
        padding: 0.5rem;
        border: 2px solid var(--frame);
      }
      .centre .glyph { grid-area: g; font-size: 32px; }
      .edge { font-size: 12px; max-width: 6rem; }
      .edge.n { grid-area: n; }
      .edge.s { grid-area: s; }
      .edge.w { grid-area: w; text-align: right; }
      .edge.e { grid-area: e; text-align: left; }

      ol { list-style: none; margin: 0; padding: 0; }
      .rows { display: grid; grid-template-columns: auto 1fr auto auto; }
      .row {
        display: grid;
        grid-column: 1 / -1;
        grid-template-columns: subgrid;
        align-items: center;
        column-gap: 0.5rem;
        padding: 2px 4px;
        cursor: pointer;
      }
      .row:hover { background: var(--highlight); }
      .row.selected { outline: 3px solid var(--selection); outline-offset: -2px; }
      .row .faces { display: block; color: var(--muted); font-size: 12px; }
      .from {
        font-size: 11px;
        padding: 0 0.3rem;
        border: 1px solid var(--muted);
        border-radius: 3px;
        color: var(--muted);
      }
      /* The share looks exactly like the 1.1 cell (point 5). */
      .share {
        position: relative;
        min-width: 2.6rem;
        padding: 0.3rem;
        text-align: right;
        border: 1px solid var(--frame);
        color: var(--shade-text-dark);
        font-variant-numeric: tabular-nums;
      }
      .go { padding: 0.1rem 0.5rem; }
      .zeros-toggle { margin-top: 0.3rem; padding: 0.1rem 0.5rem; color: var(--muted); }
      .zeros { border-top: 1px dashed var(--divider); margin-top: 0.3rem; padding-top: 0.3rem; }

      .main { flex: 1 1 auto; min-width: 0; display: flex; flex-direction: column; gap: 1rem; }
      .in-maze {
        font-size: 11px;
        padding: 0 0.3rem;
        border: 1px solid var(--text);
        border-radius: 3px;
      }
      table-explanation {
        flex: 0 0 20rem;
        border: 1px solid var(--frame);
        position: sticky;
        top: 0;
      }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    this.addEventListener("keydown", this.onKeydown);
    // The tile list for the Centre picker; nothing is centred until one is picked.
    void this.load();
  }

  disconnectedCallback(): void {
    this.removeEventListener("keydown", this.onKeydown);
    super.disconnectedCallback();
  }

  /**
   * The one entry point (DR-0026 point 10): centre on `target.centre` and,
   * if given, select that neighbour so the panel explains the pair. With
   * `target.maze`, the trail's first stop also shows why the maze has that
   * tile there (DR-0033 point 7). Starts a new trail.
   */
  open(target: ExploreTarget): void {
    const { centre, select, maze } = target;
    this.trail = [
      { centre: ref(centre), selected: select && { direction: select.direction, neighbour: ref(select.neighbour) }, maze },
    ];
    this.openZeros = new Set();
    this.factor = undefined;
    this.band = maze && { maze, note: `Replaying seed ${maze.seed}…` };
    ++this.whySerial;
    if (maze) void this.loadBand(maze);
    void this.load();
  }

  /**
   * Refetches after the app loads the tile set (Reload or a save). Centre,
   * trail and selection stay (point 1). With `revision`, the "why" band
   * refreshes in place for the same column and row of the regenerated maze
   * (DR-0037 point 17); without it, it gives way to a note.
   */
  refresh(revision?: number): void {
    const band = this.band;
    if (band) {
      ++this.whySerial;
      if (revision === undefined) {
        this.band = { maze: band.maze, note: RELOADED };
        this.factor = undefined;
      } else {
        void this.refreshBand(band, revision);
      }
    }
    void this.load();
  }

  /**
   * DR-0037 point 17. Same tile: the band updates, its heading adds "after
   * reload". Another tile: the view recentres on it with a new trail, and
   * the heading adds "was ═". A selected factor stays if its candidate and
   * column are still there. A maze that gave up keeps the old heading.
   */
  private async refreshBand(band: Band, revision: number): Promise<void> {
    const serial = this.whySerial;
    const maze = { ...band.maze, revision };
    let result: Awaited<ReturnType<typeof loadWhy>>;
    try {
      result = await loadWhy(maze);
    } catch (e) {
      if (serial !== this.whySerial) return;
      this.band = { maze, note: "Couldn't replay this cell; the error is shown above." };
      reportToolError(this, errorText(e));
      return;
    }
    if (serial !== this.whySerial) return;
    if ("stale" in result) {
      // Another load overtook this one; its refresh follows.
      return;
    }
    if ("notPlaced" in result) {
      this.band = { maze, why: band.why, note: noMazeAfterReload(maze.seed), suffix: band.suffix };
      this.factor = undefined;
      return;
    }
    const why = result.why;
    const was = band.why?.tile;
    const changed = !!was && !sameTile(was, why.tile);
    this.band = { maze, why, suffix: afterReloadText(changed && was ? glyphOf(was) : undefined) };
    const f = this.factor;
    if (f && !why.candidates.some((c) => sameTile(c.tile, f.candidate) && c.factors.some((x) => x.direction === f.direction))) {
      this.factor = undefined;
    }
    const first = this.trail[0];
    if (!first?.maze) return;
    if (!changed) {
      this.trail = [{ ...first, maze }, ...this.trail.slice(1)];
    } else if (this.trail.length === 1) {
      this.trail = [{ centre: ref(why.tile), maze }];
      this.openZeros = new Set();
      void this.load();
    } else {
      // Ian has walked on; the first stop now holds the new tile for when he comes back.
      this.trail = [{ centre: ref(why.tile), maze }, ...this.trail.slice(1)];
    }
  }

  /** The band shows at the trail's first stop, when it began at a maze cell. */
  private get bandShown(): Band | undefined {
    return this.trail.length === 1 && this.trail[0].maze ? this.band : undefined;
  }

  /** Asks Rust to replay the maze and report how the cell was drawn (DR-0034 rules 4 to 7). */
  private async loadBand(maze: MazeCell): Promise<void> {
    const serial = this.whySerial;
    try {
      const result = await loadWhy(maze, this.data);
      if (serial !== this.whySerial) return;
      this.band = "why" in result ? { maze, why: result.why } : { maze, note: "stale" in result ? RELOADED : NOT_PLACED };
    } catch (e) {
      if (serial !== this.whySerial) return;
      this.band = { maze, note: "Couldn't replay this cell; the error is shown above." };
      reportToolError(this, errorText(e));
    }
  }

  private get stop(): Stop | undefined {
    return this.trail[this.trail.length - 1];
  }

  /** The centre on screen, if it is the one the trail is on (not a step still in flight). */
  private get shown(): Centre | undefined {
    const centre = this.data?.centre;
    return centre && this.stop && sameTile(centre.tile, this.stop.centre) ? centre : undefined;
  }

  private async load(): Promise<void> {
    const serial = ++this.requestSerial;
    const centre = this.stop?.centre;
    const known = centre && this.data?.tiles.find((t) => sameTile(t, centre));
    this.busy = true;
    try {
      const data = await loadExplore(centre, this.data);
      if (serial !== this.requestSerial) return;
      this.data = data;
      this.missing =
        centre && !data.centre ? (known ? tileLabel(known) : `${centre.id}, orientation ${centre.orientation}`) : undefined;
    } catch (e) {
      // The banner shows Rust's error; the last good arms stay.
      if (serial !== this.requestSerial) return;
      reportToolError(this, errorText(e));
    } finally {
      if (serial === this.requestSerial) this.busy = false;
    }
  }

  protected updated(changed: PropertyValues): void {
    // Point 1: a centre that's gone opens the picker. showPicker needs a user
    // gesture, which an async reload may have outlived; focus is the fallback.
    if (changed.has("missing") && this.missing && this.picker) {
      this.picker.focus();
      try {
        (this.picker as { showPicker?: () => void }).showPicker?.();
      } catch {
        // Not allowed without a recent gesture; the picker has focus.
      }
    }
  }

  /** Replaces the current stop's selection; earlier stops keep theirs. Clears a selected factor. */
  private select(direction: Direction, neighbour: TileRef) {
    const stop = this.stop;
    if (!stop) return;
    this.trail = [...this.trail.slice(0, -1), { ...stop, selected: { direction, neighbour: ref(neighbour) } }];
    this.factor = undefined;
  }

  /** DR-0033 point 13: a factor's pair goes to the panel, and the arm selection is cleared. */
  private onFactorSelect(event: CustomEvent<FactorKey>) {
    this.factor = event.detail;
    const stop = this.stop;
    if (stop?.selected) this.trail = [...this.trail.slice(0, -1), { ...stop, selected: undefined }];
  }

  /**
   * Point 8: the neighbour becomes the centre, and the tile we came from is
   * selected in the opposite arm, so the panel shows the edge just crossed.
   */
  private go(direction: Direction, neighbour: TileRef) {
    const stop = this.stop;
    if (!stop) return;
    this.trail = [
      ...this.trail.slice(0, -1),
      { ...stop, selected: { direction, neighbour: ref(neighbour) } },
      { centre: ref(neighbour), selected: { direction: OPPOSITE[direction], neighbour: stop.centre, from: true } },
    ];
    this.openZeros = new Set();
    this.factor = undefined;
    void this.load();
  }

  /**
   * Point 9: back to the previous centre and what was selected there. At the
   * first stop of a trail that began at a maze cell, back to the Maze tab
   * (DR-0033 point 8), which still shows the maze and the selected cell.
   */
  private back() {
    if (this.trail.length < 2) {
      if (this.trail[0]?.maze) this.dispatchEvent(new CustomEvent(BACK_TO_MAZE_EVENT, { bubbles: true, composed: true }));
      return;
    }
    this.backTo(this.trail.length - 2);
  }

  /** Point 9: clicking a past centre goes there and drops later ones. */
  private backTo(index: number) {
    if (index >= this.trail.length - 1) return;
    this.trail = this.trail.slice(0, index + 1);
    this.openZeros = new Set();
    this.factor = undefined;
    void this.load();
  }

  private toggleZeros(direction: Direction) {
    const next = new Set(this.openZeros);
    if (!next.delete(direction)) next.add(direction);
    this.openZeros = next;
  }

  private onKeydown = (event: KeyboardEvent) => {
    if (event.altKey && event.key === "ArrowLeft") {
      event.preventDefault();
      this.back();
    }
  };

  private onPick(event: Event) {
    const tile = this.data?.tiles[Number((event.target as HTMLSelectElement).value)];
    if (tile) this.open({ centre: tile });
  }

  render() {
    return html`
      ${this.renderBar()}
      ${this.missing
        ? html`<p class="problem" role="status">${this.missing} isn't in the tile set any more. Pick a centre.</p>`
        : nothing}
      ${this.renderBody()}
    `;
  }

  private renderBar() {
    const tiles = this.data?.tiles ?? [];
    const stop = this.stop;
    return html`<div class="bar">
      <button
        id="back"
        title="Back (Alt+Left)"
        ?disabled=${this.trail.length < 2 && !this.trail[0]?.maze}
        @click=${() => this.back()}
      >
        ${this.trail.length < 2 && this.trail[0]?.maze ? "< Back to maze" : "< Back"}
      </button>
      ${this.trail.length > 0 ? this.renderTrail() : nothing}
      <label
        >Centre:
        <select id="centre" @change=${this.onPick}>
          <option value="" disabled .selected=${!stop}>Pick a tile…</option>
          ${tiles.map(
            (t, i) =>
              html`<option value=${i} .selected=${!!stop && sameTile(t, stop.centre)}>
                ${glyphOf(t)} ${tileLabel(t)}
              </option>`,
          )}
        </select>
      </label>
    </div>`;
  }

  private renderTrail() {
    const first = Math.max(0, this.trail.length - TRAIL_SHOWN);
    const last = this.trail.length - 1;
    const stops = this.trail.slice(first).map((stop, k) => {
      const i = first + k;
      const tile = this.data?.tiles.find((t) => sameTile(t, stop.centre));
      const label = tile ? tileLabel(tile) : stop.centre.id;
      const glyph = tile ? renderGlyph(tile, label) : html`<span class="glyph">${glyphOf(stop.centre)}</span>`;
      return html`${k > 0 ? html`<span class="muted">›</span>` : nothing}
        <button
          class="stop"
          data-index=${i}
          title=${i === last ? `${label} (centre)` : `Back to ${label}`}
          aria-current=${i === last ? "true" : "false"}
          @click=${() => this.backTo(i)}
        >
          ${glyph}
        </button>`;
    });
    return html`<span class="trail" aria-label="Trail"
      ><span class="muted">Trail:</span> ${first > 0 ? html`<span class="muted cut">…</span>` : nothing}${stops}</span
    >`;
  }

  private renderBody() {
    if (!this.stop) {
      return html`<p class="hint">Pick a centre tile to see what can go on each side of it, and why.</p>`;
    }
    const centre = this.shown;
    if (!centre) return this.busy ? html`<p class="hint">Loading…</p>` : nothing;
    const selected = this.stop.selected;
    const band = this.bandShown;
    return html`<div class="body">
      <div class="main">
        ${band
          ? html`<why-band
              .why=${band.why}
              note=${band.note ?? ""}
              suffix=${band.suffix ?? ""}
              .selected=${this.factor}
              @factor-select=${this.onFactorSelect}
            ></why-band>`
          : nothing}
        <div class="compass ${this.busy ? "busy" : ""}">
          ${DIRECTIONS.map((d) => this.renderArm(centre, d, selected))} ${this.renderCentre(centre.tile)}
        </div>
      </div>
      ${band?.why && this.factor ? this.renderFactorPanel(band.why, this.factor) : this.renderPanel(centre, selected)}
    </div>`;
  }

  private renderCentre(tile: TileView) {
    const [n, e, s, w] = tile.sides.map(sideName);
    return html`<div class="centre" aria-label="Centre: ${tileLabel(tile)}">
      <span class="edge n" data-edge="north">${n}</span>
      <span class="edge w" data-edge="west">${w}</span>
      ${renderGlyph(tile, headerTitle(tile))}
      <span class="edge e" data-edge="east">${e}</span>
      <span class="edge s" data-edge="south">${s}</span>
    </div>`;
  }

  private renderArm(centre: Centre, direction: Direction, selected?: Selected) {
    const arm = centre.arms[direction];
    const glyph = glyphOf(centre.tile);
    const selectedHere = selected?.direction === direction ? selected : undefined;
    // A selected 0 opens its arm's 0s, so the selection is always on screen.
    const zerosOpen =
      this.openZeros.has(direction) || (!!selectedHere && arm.zeros.some((n) => sameTile(n.tile, selectedHere.neighbour)));
    const row = (n: ArmNeighbour) => this.renderRow(centre.tile, direction, n, selectedHere);
    return html`<section class="arm ${direction}" data-direction=${direction}>
      <h3>
        ${direction.toUpperCase()} of ${glyph}
        <span class="muted">(% of ${glyph}'s ${direction} neighbours)</span>
      </h3>
      <ol class="rows">
        ${arm.neighbours.map(row)}
      </ol>
      ${arm.zeros.length > 0
        ? html`<button class="zeros-toggle" aria-expanded=${zerosOpen ? "true" : "false"} @click=${() => this.toggleZeros(direction)}>
            ${zerosOpen ? "−" : "+"} ${arm.zeros.length} can't go here
          </button>`
        : nothing}
      ${zerosOpen ? html`<ol class="rows zeros">${arm.zeros.map(row)}</ol>` : nothing}
    </section>`;
  }

  private renderRow(centre: TileView, direction: Direction, n: ArmNeighbour, selected?: Selected) {
    const { tile, entry } = n;
    const label = tileLabel(tile);
    const isSelected = !!selected && sameTile(selected.neighbour, tile);
    // DR-0033 point 14: the tile the maze has on this side, while the band shows.
    const inMaze = this.bandShown?.why?.neighbours.find((m) => m.direction === direction)?.tile;
    const isInMaze = !!inMaze && sameTile(inMaze, tile);
    const canGo = entry.probability > 0;
    const classes = [
      "share",
      `step-${shadeStep(entry.probability)}`,
      entry.exceptions.length > 0 ? "exception" : "",
    ].join(" ");
    return html`<li
      class="row ${isSelected ? "selected" : ""}"
      data-tile="${tile.id}#${tile.orientation}"
      aria-current=${isSelected ? "true" : "false"}
      tabindex="0"
      title=${cellTitle(centre, tile, direction, entry)}
      @click=${() => this.select(direction, tile)}
      @dblclick=${() => canGo && this.go(direction, tile)}
      @keydown=${(e: KeyboardEvent) => {
        if (e.key === "Enter" && e.target === e.currentTarget && canGo) this.go(direction, tile);
      }}
    >
      ${renderGlyph(tile, label)}
      <span class="name"
        >${label}${isSelected && selected?.from ? html` <span class="from">from</span>` : nothing}${isInMaze
          ? html` <span class="in-maze">in maze</span>`
          : nothing}
        <span class="faces">faces with ${sideName(entry.facing.column)}</span></span
      >
      <span class=${classes}>${formatShare(entry.probability)}</span>
      ${canGo
        ? html`<button
            class="go"
            title="Make ${label} the centre"
            @click=${(e: Event) => {
              e.stopPropagation();
              this.go(direction, tile);
            }}
          >
            Go
          </button>`
        : html`<span></span>`}
    </li>`;
  }

  /**
   * DR-0033 point 13: DR-0025's panel, unchanged, for a factor's pair: the
   * candidate as row tile, the counted neighbour as column, the direction
   * from the cell to the neighbour.
   */
  private renderFactorPanel(why: Why, key: FactorKey) {
    const candidate = why.candidates.find((c) => sameTile(c.tile, key.candidate));
    const factor = candidate?.factors.find((f) => f.direction === key.direction);
    const explained: Explained | undefined = candidate &&
      factor && {
        entry: factor.entry,
        rowTile: candidate.tile,
        columnTile: factor.neighbour,
        exceptions: why.exceptions,
        tiles: this.data?.tiles ?? [],
      };
    return html`<table-explanation .explained=${explained} direction=${key.direction}></table-explanation>`;
  }

  /** DR-0025's panel, unchanged, with the centre as row tile and the arm's direction. */
  private renderPanel(centre: Centre, selected?: Selected) {
    const arm = selected && centre.arms[selected.direction];
    const n = arm && [...arm.neighbours, ...arm.zeros].find((x) => sameTile(x.tile, selected.neighbour));
    const explained: Explained | undefined = n && {
      entry: n.entry,
      rowTile: centre.tile,
      columnTile: n.tile,
      exceptions: this.data?.exceptions ?? [],
      tiles: this.data?.tiles ?? [],
    };
    return html`<table-explanation .explained=${explained} direction=${selected?.direction ?? "east"}>
      ${n && selected && n.entry.probability > 0
        ? html`<button id="explore-selected" @click=${() => this.go(selected.direction, n.tile)}>
            Explore ${glyphOf(n.tile)}
          </button>`
        : nothing}
    </table-explanation>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "explore-view": ExploreView;
  }
}
