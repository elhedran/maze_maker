// The filter bar above the grid (DR-0027): one group for the tile (rows),
// one for its neighbour (columns), and the count line with Clear all.
// It holds no state of its own: the table owns the filters, because they go
// into its request and outlive a direction switch or a reload.

import { LitElement, css, html, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";
import { themeStyles } from "./draw-tile.ts";
import { DIRECTION_LABEL } from "./format.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { SideFilter } from "../bindings/SideFilter.ts";
import type { TileFilter } from "../bindings/TileFilter.ts";
import type { TileKind } from "../bindings/TileKind.ts";
import type { TileView } from "../bindings/TileView.ts";


export const KIND_LABEL: Record<TileKind, string> = { corridor: "Corridor", room: "Room", hedge: "Hedge" };
const KINDS: TileKind[] = ["corridor", "room", "hedge"];

/** The side kinds a filter can name; Floor+wall is any wall end (DR-0027 point 2). */
const SIDE_LABEL: Record<SideFilter, string> = {
  hedge: "Hedge",
  passage: "Passage",
  door: "Door",
  floor: "Floor",
  floor_wall: "Floor+wall",
};
const SIDES = Object.keys(SIDE_LABEL) as SideFilter[];

const OPPOSITE: Record<Direction, Direction> = { north: "south", east: "west", south: "north", west: "east" };

/** "room_edge" reads as "room edge". */
const tagLabel = (tag: string) => tag.replaceAll("_", " ");

export const noFilter = (): TileFilter => ({ kinds: [], tileIds: [], tags: [], sides: [] });

export const isFiltered = (f: TileFilter) =>
  f.kinds.length + f.tileIds.length + f.tags.length + f.sides.length > 0;

/** Which group a change came from: the tile picks rows, the neighbour picks columns. */
export type FilterGroup = "row" | "column";

export interface FilterChange {
  rowFilter: TileFilter;
  columnFilter: TileFilter;
}

/** One tile type for the multi-select: DR-0015's name, not an orientation. */
interface TileType {
  id: string;
  name: string;
  kind: TileKind;
}

/** Tile types and tags in table order, taken from the response rather than hard-coded. */
function optionsOf(tiles: TileView[]): { types: TileType[]; tags: string[] } {
  const types = new Map<string, TileType>();
  const tags = new Set<string>();
  for (const t of tiles) {
    if (!types.has(t.id)) types.set(t.id, { id: t.id, name: t.name, kind: t.kind });
    t.tags.forEach((tag) => tags.add(tag));
  }
  return { types: [...types.values()], tags: [...tags] };
}

const toggled = <T>(values: T[], value: T): T[] =>
  values.includes(value) ? values.filter((v) => v !== value) : [...values, value];

/**
 * One group's filter in words, values of a facet joined by "or", facets by
 * commas: "Room, West side Passage". Empty when nothing is set.
 */
export function describeFilter(f: TileFilter, sideDirection: Direction, tiles: TileView[]): string {
  const names = new Map(tiles.map((t) => [t.id, t.name]));
  const or = (values: string[]) => values.join(" or ");
  const parts: string[] = [];
  if (f.kinds.length) parts.push(or(f.kinds.map((k) => KIND_LABEL[k])));
  if (f.tileIds.length) parts.push(or(f.tileIds.map((id) => names.get(id) ?? id)));
  if (f.tags.length) parts.push(or(f.tags.map(tagLabel)));
  if (f.sides.length) parts.push(`${DIRECTION_LABEL[sideDirection]} side ${or(f.sides.map((s) => SIDE_LABEL[s]))}`);
  return parts.join(", ");
}

/**
 * The count line (DR-0027 point 5): "Showing all 29 rows and 29 columns." or
 * "Showing 13 of 29 rows, 15 of 29 columns." The set size is every tile in
 * the response, so it follows the tile set rather than being fixed at 29.
 */
export function countLine(setSize: number, rows: number, columns: number): string {
  if (rows === setSize && columns === setSize) return `Showing all ${setSize} rows and ${setSize} columns.`;
  return `Showing ${rows} of ${setSize} rows, ${columns} of ${setSize} columns.`;
}

@customElement("filter-bar")
export class FilterBar extends LitElement {
  /** Every (tile, orientation) in the set: the source of tile types and tags. */
  @property({ attribute: false }) tiles: TileView[] = [];
  @property({ attribute: false }) rowFilter: TileFilter = noFilter();
  @property({ attribute: false }) columnFilter: TileFilter = noFilter();
  /** The switcher's direction; the side toggles are labelled by it. */
  @property() direction: Direction = "east";
  @property({ type: Number }) totalRows = 0;
  @property({ type: Number }) totalColumns = 0;

  static styles = [
    themeStyles,
    css`
      :host {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 0.35rem 1rem;
        align-items: start;
      }
      .group-label { font-weight: 600; padding-top: 0.25rem; }
      .group-label small { display: block; font-weight: normal; color: var(--muted); }
      .facets { display: flex; flex-direction: column; gap: 0.3rem; }
      .facet-row { display: flex; flex-wrap: wrap; gap: 0.3rem 1rem; align-items: center; }
      .facet { display: flex; flex-wrap: wrap; gap: 0.25rem; align-items: center; }
      .facet-name { color: var(--muted); margin-right: 0.25rem; }
      .count { grid-column: 1 / -1; display: flex; flex-wrap: wrap; gap: 0.5rem 1rem; align-items: center; }
      .words { color: var(--muted); }
      /* Same as the direction switcher: pressed is bold and filled (DR-0017, no new colours). */
      button {
        font: inherit;
        padding: 0.15rem 0.6rem;
        border: 1px solid var(--frame);
        background: var(--page);
        color: var(--text);
        border-radius: 3px;
        cursor: pointer;
      }
      button[aria-pressed="true"] { background: var(--text); color: var(--page); font-weight: 600; }
      button:disabled { cursor: default; color: var(--muted); }
      details { position: relative; }
      summary {
        list-style: none;
        cursor: pointer;
        border: 1px solid var(--frame);
        border-radius: 3px;
        padding: 0.15rem 0.6rem;
        min-width: 10rem;
      }
      summary::after { content: " ▾"; color: var(--muted); }
      details[open] .menu {
        position: absolute;
        z-index: 3;
        top: 100%;
        left: 0;
        background: var(--page);
        border: 1px solid var(--frame);
        padding: 0.4rem 0.6rem;
        min-width: 14rem;
      }
      fieldset { border: none; margin: 0 0 0.3rem; padding: 0; }
      legend { font-weight: 600; padding: 0; }
      label { display: block; white-space: nowrap; }
    `,
  ];

  render() {
    const { types, tags } = optionsOf(this.tiles);
    const filtered = isFiltered(this.rowFilter) || isFiltered(this.columnFilter);
    const rowWords = describeFilter(this.rowFilter, this.direction, this.tiles);
    const columnWords = describeFilter(this.columnFilter, OPPOSITE[this.direction], this.tiles);
    const words = [rowWords && `Rows: ${rowWords}.`, columnWords && `Columns: ${columnWords}.`].filter(Boolean).join(" ");
    return html`
      ${this.renderGroup("row", "Tile", "rows", this.rowFilter, this.direction, types, tags)}
      ${this.renderGroup("column", "Neighbour", "columns", this.columnFilter, OPPOSITE[this.direction], types, tags)}
      <div class="count" role="status">
        <span class="count-line">${countLine(this.tiles.length, this.totalRows, this.totalColumns)}</span>
        ${words ? html`<span class="words">${words}</span>` : nothing}
        <button id="clear-all" ?disabled=${!filtered} @click=${this.clearAll}>Clear all</button>
      </div>
    `;
  }

  private renderGroup(
    group: FilterGroup,
    name: string,
    axis: string,
    filter: TileFilter,
    sideDirection: Direction,
    types: TileType[],
    tags: string[],
  ) {
    const set = (next: Partial<TileFilter>) => this.change(group, { ...filter, ...next });
    const toggle = <T extends string>(label: string, values: T[], value: T, field: keyof TileFilter) =>
      html`<button
        data-value=${value}
        aria-pressed=${values.includes(value) ? "true" : "false"}
        @click=${() => set({ [field]: toggled(values, value) })}
      >
        ${label}
      </button>`;
    const typeSummary = filter.tileIds.length
      ? filter.tileIds.map((id) => types.find((t) => t.id === id)?.name ?? id).join(", ")
      : "any";

    return html`
      <div class="group-label">${name}<small>(${axis})</small></div>
      <div class="facets" role="group" aria-label="${name} filters" data-group=${group}>
        <div class="facet-row">
          <span class="facet" data-facet="kind">
            <span class="facet-name">Kind</span>
            ${KINDS.map((k) => toggle(KIND_LABEL[k], filter.kinds, k, "kinds"))}
          </span>
          <span class="facet" data-facet="type">
            <span class="facet-name">Type</span>
            <details>
              <summary>${typeSummary}</summary>
              <div class="menu">
                ${KINDS.filter((k) => types.some((t) => t.kind === k)).map(
                  (k) => html`<fieldset>
                    <legend>${KIND_LABEL[k]}</legend>
                    ${types
                      .filter((t) => t.kind === k)
                      .map(
                        (t) => html`<label
                          ><input
                            type="checkbox"
                            data-value=${t.id}
                            .checked=${filter.tileIds.includes(t.id)}
                            @change=${() => set({ tileIds: toggled(filter.tileIds, t.id) })}
                          />
                          ${t.name}</label
                        >`,
                      )}
                  </fieldset>`,
                )}
              </div>
            </details>
          </span>
        </div>
        ${tags.length
          ? html`<span class="facet" data-facet="tag">
              <span class="facet-name">Tag</span>
              ${tags.map((tag) => toggle(tagLabel(tag), filter.tags, tag, "tags"))}
            </span>`
          : nothing}
        <span class="facet" data-facet="side">
          <span class="facet-name">${DIRECTION_LABEL[sideDirection]} side</span>
          ${SIDES.map((s) => toggle(SIDE_LABEL[s], filter.sides, s, "sides"))}
        </span>
      </div>
    `;
  }

  private change(group: FilterGroup, filter: TileFilter) {
    const detail: FilterChange = {
      rowFilter: group === "row" ? filter : this.rowFilter,
      columnFilter: group === "column" ? filter : this.columnFilter,
    };
    this.dispatchEvent(new CustomEvent<FilterChange>("filter-change", { detail }));
  }

  private clearAll = () => {
    this.dispatchEvent(
      new CustomEvent<FilterChange>("filter-change", { detail: { rowFilter: noFilter(), columnFilter: noFilter() } }),
    );
  };
}

declare global {
  interface HTMLElementTagNameMap {
    "filter-bar": FilterBar;
  }
  interface HTMLElementEventMap {
    "filter-change": CustomEvent<FilterChange>;
  }
}
