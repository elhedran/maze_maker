import { LitElement, css, html, nothing, type PropertyValues } from "lit";
import { customElement, state } from "lit/decorators.js";
import { errorText, tableView, wholeTableRequest } from "../ipc.ts";
import { reportToolError } from "../tool-error.ts";
import { openExplore } from "../explore/open-explore.ts";
import {
  DIRECTIONS,
  DIRECTION_KEYS,
  DIRECTION_LABEL,
  SHADE_LEGEND,
  cellTitle,
  formatShare,
  headerTitle,
  shadeStep,
} from "./format.ts";
import { glyphStyles, renderGlyph, shadeStyles, themeStyles } from "./draw-tile.ts";
import type { Explained } from "./table-explanation.ts";
import "./table-explanation.ts";
import { KIND_LABEL, applyFilterEdit, noFilter, type FilterEdit, type Filters } from "./filter-bar.ts";
import "./filter-bar.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { TableView } from "../bindings/TableView.ts";
import type { TileFilter } from "../bindings/TileFilter.ts";
import type { TileKind } from "../bindings/TileKind.ts";
import type { TileView } from "../bindings/TileView.ts";

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
  /**
   * The tile filter picks rows, the neighbour filter picks columns (DR-0027).
   * Kept across a direction switch and a reload; not saved between runs.
   */
  @state() private rowFilter: TileFilter = noFilter();
  @state() private columnFilter: TileFilter = noFilter();
  /** The filters `view` was fetched with, so the corner sentence matches the grid. */
  private viewFilters: Filters = { rowFilter: noFilter(), columnFilter: noFilter() };
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
    shadeStyles,
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
      .muted { color: var(--muted); }
      .empty { color: var(--muted); padding: 2rem 0; margin: 0; }
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
      .body { display: flex; gap: 1rem; align-items: flex-start; }
      .grid-wrap { overflow: auto; max-height: calc(100vh - 8rem); }
      table:focus { outline: 2px solid var(--frame); }
      table { border-collapse: collapse; font-variant-numeric: tabular-nums; }
      th { font-weight: normal; padding: 1px; }
      .corner { text-align: left; vertical-align: bottom; max-width: 10rem; color: var(--muted); }
      .kind { font-weight: 600; text-align: left; vertical-align: top; padding: 0.2rem 0.4rem; }
      th.lit .glyph { background-color: var(--highlight); }
      /* Tile headers stay in view while the grid scrolls, so a cell's row and column are always readable. */
      th[scope="col"] { position: sticky; top: 0; z-index: 1; background: var(--page); }
      th[scope="row"] { position: sticky; left: 0; z-index: 1; background: var(--page); }
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

  /**
   * Refetches after the app reloads the tile set (DR-0029 point 3). The
   * selection and direction stay, since they're kept by tile identity.
   */
  refresh(): void {
    void this.fetchView();
  }

  /**
   * Asks Rust for the table in the current direction only, filtered. One
   * direction per request: the side facet needs it, and at 29 × 29 a request
   * per switch is cheap (DR-0027). A failure goes to the app's banner and the
   * last good table stays (DR-0025 point 12).
   */
  private async fetchView(): Promise<void> {
    const serial = ++this.requestSerial;
    const direction = this.direction;
    const filters = { rowFilter: this.rowFilter, columnFilter: this.columnFilter };
    this.busy = true;
    try {
      const view = await tableView({ ...wholeTableRequest(direction), ...filters });
      if (serial !== this.requestSerial) return;
      this.viewDirection = direction;
      this.viewFilters = filters;
      this.view = view;
      this.dropSelectionIfHidden(view);
    } catch (e) {
      if (serial !== this.requestSerial) return;
      reportToolError(this, errorText(e));
    } finally {
      if (serial === this.requestSerial) this.busy = false;
    }
  }

  /** DR-0027 point 8: a selected cell that is filtered out is no longer selected. */
  private dropSelectionIfHidden(view: TableView) {
    if (!this.selection) return;
    const shown = (indices: number[], key: TileKey) => indices.some((i) => keyOf(view.tiles[i]) === key);
    if (!shown(view.rows, this.selection.row) || !shown(view.columns, this.selection.column)) {
      this.selection = undefined;
    }
  }

  private onFilterEdit(event: CustomEvent<FilterEdit>) {
    const next = applyFilterEdit({ rowFilter: this.rowFilter, columnFilter: this.columnFilter }, event.detail);
    this.rowFilter = next.rowFilter;
    this.columnFilter = next.columnFilter;
    this.hover = undefined;
    void this.fetchView();
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
      ${this.view ? this.renderBody(this.view) : html`<p class="muted">${this.busy ? "Loading the table…" : ""}</p>`}
    `;
  }

  private renderTopBar() {
    const v = this.view;
    return html`
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
      ${v
        ? html`<filter-bar
            .tiles=${v.tiles}
            .rowFilter=${this.rowFilter}
            .columnFilter=${this.columnFilter}
            direction=${this.direction}
            .totalRows=${v.totalRows}
            .totalColumns=${v.totalColumns}
            @filter-edit=${this.onFilterEdit}
          ></filter-bar>`
        : nothing}
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
    if (this.rowTiles.length === 0 || this.columnTiles.length === 0) {
      // DR-0027 point 7: the grid gives way; the count line and Clear all stay in the bar.
      return html`<p class="empty">No tiles match these filters</p>`;
    }
    const kinds = (f: TileFilter) => f.kinds.map((k) => KIND_LABEL[k]).join(" or ");
    const rowWord = kinds(this.viewFilters.rowFilter) || "Row";
    const columnWord = kinds(this.viewFilters.columnFilter);
    const corner = `${rowWord} tile, then its ${columnWord ? `${columnWord} ` : ""}neighbour to the ${dirWord}`;

    return html`<div class="body">
      <div class="grid-wrap">
        <table
          role="grid"
          tabindex="0"
          aria-label=${corner}
          @keydown=${this.onGridKeydown}
          @mouseover=${this.onGridHover}
          @mouseleave=${() => (this.hover = undefined)}
        >
          <thead>
            <tr>
              <th class="corner" colspan="2" rowspan="2">
                ${corner} (% of row)
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
      ${this.renderPanel(v, selected)}
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

  /**
   * DR-0025's panel, with "Explore this tile" (its point 13): the explore
   * view opens on the row tile with this pair selected (DR-0026 point 10).
   */
  private renderPanel(v: TableView, selected?: { row: number; column: number }) {
    const explained = this.explained(v, selected);
    const explore = () => {
      if (!explained) return;
      openExplore(this, {
        centre: explained.rowTile,
        select: { direction: this.viewDirection, neighbour: explained.columnTile },
      });
    };
    return html`<table-explanation .explained=${explained} direction=${this.viewDirection}>
      ${explained ? html`<button id="explore-this" @click=${explore}>Explore this tile</button>` : nothing}
    </table-explanation>`;
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

function swatchStyle(step: number): string {
  const text = step >= 4 ? "var(--shade-text-light)" : "var(--shade-text-dark)";
  return `background: var(--shade-${step}); color: ${text}`;
}

declare global {
  interface HTMLElementTagNameMap {
    "adjacency-table": AdjacencyTable;
  }
}
