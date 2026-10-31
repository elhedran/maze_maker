import { LitElement, css, html, nothing, type PropertyValues } from "lit";
import { customElement, state } from "lit/decorators.js";
import { errorText, reloadTileSet, tableView, wholeTableRequest } from "../ipc.ts";
import {
  DIRECTIONS,
  DIRECTION_KEYS,
  DIRECTION_LABEL,
  SHADE_LEGEND,
  formatLoadTime,
  formatNumber,
  formatShare,
  shadeStep,
} from "./format.ts";
import { glyphStyles, renderGlyph, themeStyles } from "./draw-tile.ts";
import { tileLabel, tileLook } from "./tile-glyphs.ts";
import type { Explained } from "./table-explanation.ts";
import "./table-explanation.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { TableView } from "../bindings/TableView.ts";
import type { TileKind } from "../bindings/TileKind.ts";
import type { TileView } from "../bindings/TileView.ts";

const KIND_LABEL: Record<TileKind, string> = { corridor: "Corridor", room: "Room", hedge: "Hedge" };

/** A (tile, orientation) by identity, so a selection survives reloads and direction switches. */
type TileKey = string;
const keyOf = (tile: TileView): TileKey => `${tile.id}#${tile.orientation}`;

interface Selection {
  row: TileKey;
  column: TileKey;
}

/** One run of rows or columns of the same kind, for labels and dividers. */
interface KindGroup {
  kind: TileKind;
  start: number;
  count: number;
}

function groupsOf(tiles: TileView[]): KindGroup[] {
  const groups: KindGroup[] = [];
  tiles.forEach((tile, i) => {
    const last = groups[groups.length - 1];
    if (last && last.kind === tile.kind) last.count++;
    else groups.push({ kind: tile.kind, start: i, count: 1 });
  });
  return groups;
}

/**
 * The adjacency table for one direction at a time, with the explanation
 * panel for the selected entry (DR-0025). Rust sends the numbers and the
 * order; this component lays them out.
 */
@customElement("adjacency-table")
export class AdjacencyTable extends LitElement {
  @state() private direction: Direction = "east";
  @state() private view?: TableView;
  /**
   * The direction `view` was fetched for. It lags `direction` while a switch
   * is in flight, and the grid's labels follow the numbers on screen.
   */
  @state() private viewDirection: Direction = "east";
  /** Shown as a red banner; the last good table stays (DR-0025 point 12). */
  @state() private error?: string;
  @state() private selection?: Selection;
  @state() private hover?: { row: number; column: number };
  @state() private busy = false;

  // Derived from `view` in willUpdate, so render stays a plain read.
  private rowTiles: TileView[] = [];
  private columnTiles: TileView[] = [];
  private entries = new Map<string, EntryView>();

  /** Ignores a response that arrives after a newer request was sent. */
  private requestSerial = 0;

  static styles = [
    themeStyles,
    glyphStyles,
    css`
      :host {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      .bar {
        display: flex;
        flex-wrap: wrap;
        align-items: center;
        gap: 0.5rem 1rem;
      }
      .path { font-family: var(--mono); }
      .muted { color: var(--muted); }
      button {
        font: inherit;
        padding: 0.25rem 0.75rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      button[aria-pressed="true"] {
        background: var(--text);
        color: var(--page);
        font-weight: 600;
      }
      .legend { display: flex; gap: 0.25rem; align-items: center; }
      .swatch { padding: 0 0.4rem; border: 1px solid var(--frame); }
      .banner {
        color: var(--problem);
        background: var(--problem-bg);
        border: 1px solid var(--problem);
        padding: 0.5rem 0.75rem;
        white-space: pre-wrap;
        margin: 0;
      }
      .body { display: flex; gap: 1rem; align-items: flex-start; }
      .grid-wrap { overflow: auto; max-height: calc(100vh - 8rem); }
      table:focus { outline: 2px solid var(--frame); }
      table { border-collapse: collapse; font-variant-numeric: tabular-nums; }
      th { font-weight: normal; padding: 1px; }
      .corner { text-align: left; vertical-align: bottom; max-width: 10rem; color: var(--muted); }
      .kind { font-weight: 600; text-align: left; vertical-align: top; padding: 0.2rem 0.4rem; }
      th.lit .glyph { background-color: var(--highlight); }
      td {
        position: relative;
        min-width: 2.6rem;
        height: 2rem;
        text-align: right;
        padding: 0 0.3rem;
        border: 1px solid var(--frame);
        cursor: pointer;
        color: var(--shade-text-dark);
      }
      td.step-1 { background: var(--shade-1); }
      td.step-2 { background: var(--shade-2); }
      td.step-3 { background: var(--shade-3); }
      td.step-4 { background: var(--shade-4); color: var(--shade-text-light); }
      td.step-5 { background: var(--shade-5); color: var(--shade-text-light); }
      /* Exception mark: a small corner triangle (DR-0025 point 8). */
      td.exception::after,
      .mark::after {
        content: "";
        position: absolute;
        top: 0;
        left: 0;
        border-top: 7px solid var(--muted);
        border-right: 7px solid transparent;
      }
      .mark { position: relative; display: inline-block; width: 1rem; height: 1rem; border: 1px solid var(--frame); }
      td.selected { outline: 3px solid var(--selection); outline-offset: -2px; }
      .group-start-col { border-left: 3px solid var(--divider); }
      .group-start-row > * { border-top: 3px solid var(--divider); }
      table-explanation {
        flex: 0 0 22rem;
        border: 1px solid var(--frame);
        position: sticky;
        top: 0;
      }
    `,
  ];

  connectedCallback(): void {
    super.connectedCallback();
    this.addEventListener("keydown", this.onKeydown);
    void this.fetchView();
  }

  disconnectedCallback(): void {
    this.removeEventListener("keydown", this.onKeydown);
    super.disconnectedCallback();
  }

  protected willUpdate(changed: PropertyValues): void {
    if (changed.has("view")) {
      const v = this.view;
      this.rowTiles = v ? v.rows.map((i) => v.tiles[i]) : [];
      this.columnTiles = v ? v.columns.map((i) => v.tiles[i]) : [];
      this.entries = new Map(
        (v?.entries ?? [])
          .filter((e) => e.direction === this.viewDirection)
          .map((e) => [`${e.row},${e.column}`, e]),
      );
    }
  }

  /** Asks Rust for the whole table in the current direction. */
  private async fetchView(): Promise<boolean> {
    const serial = ++this.requestSerial;
    const direction = this.direction;
    this.busy = true;
    try {
      const view = await tableView(wholeTableRequest(direction));
      if (serial !== this.requestSerial) return false;
      this.viewDirection = direction;
      this.view = view;
      return true;
    } catch (e) {
      if (serial !== this.requestSerial) return false;
      this.error = errorText(e);
      return false;
    } finally {
      if (serial === this.requestSerial) this.busy = false;
    }
  }

  private async reload() {
    this.busy = true;
    try {
      await reloadTileSet();
    } catch (e) {
      this.error = errorText(e);
      this.busy = false;
      return;
    }
    if (await this.fetchView()) this.error = undefined;
  }

  private setDirection(direction: Direction) {
    if (direction === this.direction) return;
    this.direction = direction;
    void this.fetchView();
  }

  private onKeydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey) return;
    const direction = DIRECTION_KEYS[event.key.toLowerCase()];
    if (direction) {
      this.setDirection(direction);
      event.preventDefault();
    }
  };

  /** Arrow keys move the selection once the grid has focus (DR-0025 point 10). */
  private onGridKeydown(event: KeyboardEvent) {
    const moves: Record<string, [number, number]> = {
      ArrowUp: [-1, 0],
      ArrowDown: [1, 0],
      ArrowLeft: [0, -1],
      ArrowRight: [0, 1],
    };
    const move = moves[event.key];
    if (!move || this.rowTiles.length === 0) return;
    event.preventDefault();
    const at = this.selectedPosition() ?? { row: 0, column: 0 };
    const clamp = (n: number, max: number) => Math.min(Math.max(n, 0), max - 1);
    this.select(clamp(at.row + move[0], this.rowTiles.length), clamp(at.column + move[1], this.columnTiles.length));
  }

  private select(row: number, column: number) {
    this.selection = { row: keyOf(this.rowTiles[row]), column: keyOf(this.columnTiles[column]) };
  }

  /** Where the selection is in the current grid, if its tiles are still in it. */
  private selectedPosition(): { row: number; column: number } | undefined {
    if (!this.selection) return undefined;
    const row = this.rowTiles.findIndex((t) => keyOf(t) === this.selection!.row);
    const column = this.columnTiles.findIndex((t) => keyOf(t) === this.selection!.column);
    return row < 0 || column < 0 ? undefined : { row, column };
  }

  private entryAt(row: number, column: number): EntryView | undefined {
    const v = this.view!;
    return this.entries.get(`${v.rows[row]},${v.columns[column]}`);
  }

  private onGridHover(event: Event) {
    const cell = (event.target as HTMLElement).closest("td");
    if (!cell) return;
    const row = Number(cell.dataset.row);
    const column = Number(cell.dataset.column);
    if (this.hover?.row !== row || this.hover?.column !== column) this.hover = { row, column };
  }

  render() {
    return html`
      ${this.renderTopBar()}
      ${this.error ? html`<pre class="banner" role="alert">${this.error}</pre>` : nothing}
      ${this.view ? this.renderBody(this.view) : html`<p class="muted">${this.busy ? "Loading the table…" : ""}</p>`}
    `;
  }

  private renderTopBar() {
    const v = this.view;
    return html`
      <div class="bar">
        <span>Tile set: <span class="path">${v?.tileSetPath ?? "—"}</span></span>
        ${v ? html`<span class="muted">loaded ${formatLoadTime(v.loadedAt)}</span>` : nothing}
        <button id="reload" @click=${() => this.reload()} ?disabled=${this.busy}>Reload tile set</button>
      </div>
      <div class="bar">
        <span>Neighbour to the:</span>
        <span role="group" aria-label="Direction">
          ${DIRECTIONS.map(
            (d) => html`<button
              data-direction=${d}
              aria-pressed=${d === this.direction ? "true" : "false"}
              @click=${() => this.setDirection(d)}
            >
              ${DIRECTION_LABEL[d]}
            </button>`,
          )}
        </span>
        <span class="legend" aria-label="Shade legend">
          <span class="muted">shade:</span>
          ${SHADE_LEGEND.map((label, i) => html`<span class="swatch" style=${swatchStyle(i + 1)}>${label}</span>`)}
          <span class="mark"></span><span class="muted">exception applied</span>
        </span>
      </div>
    `;
  }

  private renderBody(v: TableView) {
    const columnGroups = groupsOf(this.columnTiles);
    const rowGroups = groupsOf(this.rowTiles);
    const columnStarts = new Set(columnGroups.slice(1).map((g) => g.start));
    const rowStarts = new Set(rowGroups.slice(1).map((g) => g.start));
    const selected = this.selectedPosition();
    const lit = { row: new Set<number>(), column: new Set<number>() };
    for (const p of [selected, this.hover]) {
      if (p) {
        lit.row.add(p.row);
        lit.column.add(p.column);
      }
    }
    const dirWord = this.viewDirection.toUpperCase();

    return html`<div class="body">
      <div class="grid-wrap">
        <table
          role="grid"
          tabindex="0"
          aria-label="Row tile, then its neighbour to the ${dirWord}"
          @keydown=${this.onGridKeydown}
          @mouseover=${this.onGridHover}
          @mouseleave=${() => (this.hover = undefined)}
        >
          <thead>
            <tr>
              <th class="corner" colspan="2" rowspan="2">
                Row tile, then its neighbour to the ${dirWord} (% of row)
              </th>
              ${columnGroups.map(
                (g, i) =>
                  html`<th class="kind ${i > 0 ? "group-start-col" : ""}" colspan=${g.count} scope="colgroup">
                    ${KIND_LABEL[g.kind]}
                  </th>`,
              )}
            </tr>
            <tr>
              ${this.columnTiles.map(
                (tile, c) =>
                  html`<th
                    scope="col"
                    class="${columnStarts.has(c) ? "group-start-col" : ""} ${lit.column.has(c) ? "lit" : ""}"
                    title=${headerTitle(tile)}
                  >
                    ${renderGlyph(tile, headerTitle(tile))}
                  </th>`,
              )}
            </tr>
          </thead>
          <tbody>
            ${this.rowTiles.map((tile, r) => {
              const group = rowGroups.find((g) => g.start === r);
              return html`<tr class=${rowStarts.has(r) ? "group-start-row" : ""}>
                ${group ? html`<th class="kind" rowspan=${group.count} scope="rowgroup">${KIND_LABEL[group.kind]}</th>` : nothing}
                <th scope="row" class=${lit.row.has(r) ? "lit" : ""} title=${headerTitle(tile)}>
                  ${renderGlyph(tile, headerTitle(tile))}
                </th>
                ${this.columnTiles.map((columnTile, c) =>
                  this.renderCell(r, c, tile, columnTile, columnStarts.has(c), selected),
                )}
              </tr>`;
            })}
          </tbody>
        </table>
      </div>
      <table-explanation .explained=${this.explained(v, selected)} direction=${this.viewDirection}></table-explanation>
    </div>`;
  }

  private renderCell(
    r: number,
    c: number,
    rowTile: TileView,
    columnTile: TileView,
    groupStart: boolean,
    selected?: { row: number; column: number },
  ) {
    const entry = this.entryAt(r, c);
    const probability = entry?.probability ?? 0;
    const classes = [
      `step-${shadeStep(probability)}`,
      groupStart ? "group-start-col" : "",
      entry && entry.exceptions.length > 0 ? "exception" : "",
      selected?.row === r && selected?.column === c ? "selected" : "",
    ].join(" ");
    return html`<td
      class=${classes}
      data-row=${r}
      data-column=${c}
      aria-selected=${selected?.row === r && selected?.column === c ? "true" : "false"}
      title=${cellTitle(rowTile, columnTile, this.viewDirection, entry)}
      @click=${() => this.select(r, c)}
    >
      ${formatShare(probability)}
    </td>`;
  }

  private explained(v: TableView, at?: { row: number; column: number }): Explained | undefined {
    if (!at) return undefined;
    const entry = this.entryAt(at.row, at.column);
    if (!entry) return undefined;
    return {
      entry,
      rowTile: this.rowTiles[at.row],
      columnTile: this.columnTiles[at.column],
      exceptions: v.exceptions,
    };
  }
}

/** Header hover and screen-reader label: "Corridor corner, open N and E · weight 3.0". */
function headerTitle(tile: TileView): string {
  return `${tileLabel(tile)} · weight ${formatNumber(tile.weight)}`;
}

/** "─ Corridor straight, E–W, then ┐ Corridor corner, open S and W to its east: 15.0%". */
function cellTitle(rowTile: TileView, columnTile: TileView, direction: Direction, entry?: EntryView): string {
  const a = `${tileLook(rowTile.id, rowTile.orientation).glyph} ${tileLabel(rowTile)}`;
  const b = `${tileLook(columnTile.id, columnTile.orientation).glyph} ${tileLabel(columnTile)}`;
  const share = entry && entry.probability > 0 ? `${formatShare(entry.probability)}%` : "can't happen";
  return `${a}, then ${b} to its ${direction}: ${share}`;
}

function swatchStyle(step: number): string {
  const text = step >= 4 ? "var(--shade-text-light)" : "var(--shade-text-dark)";
  return `background: var(--shade-${step}); color: ${text}`;
}

declare global {
  interface HTMLElementTagNameMap {
    "adjacency-table": AdjacencyTable;
  }
}
