import { LitElement, css, html, nothing, type PropertyValues } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { OPPOSITE, clashLine, formatNumber, formatShare, sideName } from "./format.ts";
import { glyphStyles, renderGlyph, themeStyles } from "./draw-tile.ts";
import { tileLabel, tileLook } from "./tile-glyphs.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { ExceptionView } from "../bindings/ExceptionView.ts";
import type { TileView } from "../bindings/TileView.ts";
import type { NumberTarget } from "../pending-bindings.ts";
import { editLabel, reachText, savedText } from "../edit/edit-text.ts";
import { showCompare, type Saved } from "../edit/save-number.ts";
import "../edit/number-editor.ts";

/** What the panel explains: one entry and the tiles and exceptions it refers to. */
export interface Explained {
  entry: EntryView;
  rowTile: TileView;
  columnTile: TileView;
  exceptions: ExceptionView[];
  /** The whole tile list, so an edit can say how far a tile weight reaches (DR-0037 point 4). */
  tiles: TileView[];
}

/**
 * The `[bases]` key for the entry's pair (DR-0038, Ash's addition), or null.
 * Read loosely until the generated EntryView carries `basePair`.
 */
function basePairOf(entry: EntryView): string | null {
  return (entry as EntryView & { basePair?: string | null }).basePair ?? null;
}

/** The pair an explanation is about, so a new selection can be told from a refetch. */
function pairKey(x: Explained | undefined, direction: Direction): string {
  if (!x) return "";
  return `${direction}|${x.rowTile.id}#${x.rowTile.orientation}|${x.columnTile.id}#${x.columnTile.orientation}`;
}

/**
 * The explanation panel (DR-0025 point 11). Every number comes from Rust;
 * this only lays out the product it was built from.
 */
@customElement("table-explanation")
export class TableExplanation extends LitElement {
  @property({ attribute: false }) explained?: Explained;
  @property() direction: Direction = "east";

  /** Which line's editor is open (DR-0037 point 3: one at a time). */
  @state() private editing?: string;
  /** Point 7's saved line; stays until the next edit or a new selection. */
  @state() private saved?: { text: string; warnings: string[]; canCompare: boolean };
  private shownPair = "";

  static styles = [
    themeStyles,
    glyphStyles,
    css`
      :host {
        display: block;
        padding: 0.75rem 1rem;
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      .pair {
        display: flex;
        gap: 0;
        margin-bottom: 0.5rem;
      }
      .pair.north { flex-direction: column-reverse; align-items: flex-start; }
      .pair.south { flex-direction: column; align-items: flex-start; }
      .pair.west { flex-direction: row-reverse; justify-content: flex-end; }
      .names p { margin: 0.1rem 0; }
      h3 {
        font-size: 13px;
        margin: 0.9rem 0 0.3rem;
        color: var(--muted);
        text-transform: uppercase;
        letter-spacing: 0.04em;
      }
      .share { font-size: 16px; font-weight: 600; margin: 0.6rem 0 0; }
      .cant { color: var(--problem); }
      ol { list-style: none; margin: 0; padding: 0; font-variant-numeric: tabular-nums; }
      li { padding: 0.1rem 0; }
      .op { display: inline-block; width: 1.2em; color: var(--muted); }
      .hint { color: var(--muted); }
      .rule { margin-left: 1.2em; color: var(--muted); font-family: var(--mono); font-size: 12px; }
      ::slotted(*) { margin-top: 0.9rem; }
      /* A stored number: editable, shown as the number with a dotted underline (DR-0037 point 1). */
      button.number {
        font: inherit;
        color: inherit;
        background: none;
        border: none;
        padding: 0;
        cursor: pointer;
        text-decoration: underline dotted;
        text-underline-offset: 3px;
      }
      button.number:hover { background: var(--highlight); }
      .saved { margin: 0.6rem 0 0; }
      .saved button, .actions button { font: inherit; }
      .warning { color: var(--muted); margin: 0.2rem 0 0; }
    `,
  ];

  /**
   * The panel, then a slot for the host's action on the selected pair:
   * "Explore this tile" in the table, "Explore ┐" in the explore view
   * (DR-0025 point 13, DR-0026). The panel itself is the same in both.
   */
  render() {
    return html`${this.renderExplanation()}${this.renderSaved()}<slot class="actions"></slot>`;
  }

  protected willUpdate(changed: PropertyValues): void {
    if (changed.has("explained") || changed.has("direction")) {
      const key = pairKey(this.explained, this.direction);
      if (key !== this.shownPair) {
        // A new selection: drop an open editor and the saved line. A refetch of the same pair keeps both.
        this.shownPair = key;
        this.editing = undefined;
        this.saved = undefined;
      }
    }
  }

  private onSaved(event: Event, label: string) {
    const s = (event as CustomEvent<Saved>).detail;
    this.editing = undefined;
    this.saved = { text: savedText(s.record, label), warnings: s.warnings, canCompare: s.canCompare };
  }

  /** A stored number as a button that opens its editor under the line (DR-0037 points 1 and 3). */
  private editable(key: string, target: NumberTarget, value: number, x: Explained) {
    const names = { tiles: x.tiles, exceptions: x.exceptions };
    const label = editLabel(target, names);
    return html`<button
        class="number"
        data-edit=${key}
        aria-label="Edit ${label}, ${formatNumber(value)}"
        @click=${() => (this.editing = key)}
      >
        ${formatNumber(value)}</button
      >${this.editing === key
        ? html`<number-editor
            .target=${target}
            .value=${value}
            label=${label}
            reach=${reachText(target, names)}
            @number-saved=${(e: Event) => this.onSaved(e, label)}
            @editor-close=${() => (this.editing = undefined)}
          ></number-editor>`
        : nothing}`;
  }

  /** Point 7: "Saved 20:31 · Corridor corner weight 3.0 → 2.0 [See before and after]". */
  private renderSaved() {
    const s = this.saved;
    if (!s) return nothing;
    return html`<p class="saved" role="status">
        ${s.text}
        ${s.canCompare ? html`<button id="see-compare" @click=${() => showCompare(this)}>See before and after</button>` : nothing}
      </p>
      ${s.warnings.map((w) => html`<p class="warning">${w}</p>`)}`;
  }

  private renderExplanation() {
    const x = this.explained;
    if (!x) return html`<p class="hint">Select a cell to see where its number comes from.</p>`;
    const { entry, rowTile, columnTile } = x;
    const rowGlyph = tileLook(rowTile.id, rowTile.orientation).glyph;
    const rowName = tileLabel(rowTile);
    const columnName = tileLabel(columnTile);

    const header = html`
      <div class="pair ${this.direction}">
        ${renderGlyph(rowTile, rowName)} ${renderGlyph(columnTile, columnName)}
      </div>
      <div class="names">
        <p>${rowName}</p>
        <p>then to its ${this.direction}</p>
        <p>${columnName}</p>
      </div>
    `;
    const facing = html`
      <h3>Facing sides</h3>
      <ol>
        <li>${rowGlyph} ${this.direction} side: ${sideName(entry.facing.row)}</li>
        <li>
          ${tileLook(columnTile.id, columnTile.orientation).glyph} ${OPPOSITE[this.direction]} side:
          ${sideName(entry.facing.column)}
        </li>
      </ol>
    `;

    if (!entry.allowed) {
      return html`${header}
        <p class="share cant">Can't happen</p>
        ${facing}
        <p class="clash">${clashLine(entry.facing, entry.mayMeet)}</p>`;
    }

    const share = formatShare(entry.probability);
    const pair = basePairOf(entry);
    const exceptionLines =
      entry.exceptions.length === 0
        ? html`<li><span class="op">×</span> exceptions: none</li>`
        : entry.exceptions.map((i) => {
            const e = x.exceptions[i];
            return html`<li class="exception">
              <span class="op">×</span>
              ${e ? html`${e.name} ${this.editable(`exception:${i}`, { kind: "exception", index: i }, e.multiplier, x)}` : `exception ${i}`}
              ${e ? html`<div class="rule">${e.rule}</div>` : ""}
            </li>`;
          });

    return html`${header}
      <p class="share">${share}% of ${rowGlyph}'s ${this.direction} neighbours</p>
      ${facing}
      <h3>Where it comes from</h3>
      <ol class="sum">
        <li>
          <span class="op"></span> ${sideName(entry.facing.row)}–${sideName(entry.facing.column)} base
          ${pair ? this.editable("base", { kind: "base", pair }, entry.base, x) : formatNumber(entry.base)}
        </li>
        <li>
          <span class="op">×</span> ${rowTile.name}
          ${this.editable("row", { kind: "tileWeight", tileId: rowTile.id }, entry.rowWeight, x)}
        </li>
        <li>
          <span class="op">×</span> ${columnTile.name}
          ${this.editable("column", { kind: "tileWeight", tileId: columnTile.id }, entry.columnWeight, x)}
        </li>
        ${exceptionLines}
        <li class="noise"><span class="op">×</span> noise ${formatNumber(entry.noise)}</li>
        <li><span class="op">=</span> entry ${formatNumber(entry.weight)}</li>
        <li><span class="op">÷</span> row total ${formatNumber(entry.rowTotal)} = ${share}%</li>
      </ol>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "table-explanation": TableExplanation;
  }
}
