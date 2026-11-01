import { LitElement, css, html } from "lit";
import { customElement, property } from "lit/decorators.js";
import { OPPOSITE, clashLine, formatNumber, formatShare, sideName } from "./format.ts";
import { glyphStyles, renderGlyph, themeStyles } from "./draw-tile.ts";
import { tileLabel, tileLook } from "./tile-glyphs.ts";
import type { Direction } from "../bindings/Direction.ts";
import type { EntryView } from "../bindings/EntryView.ts";
import type { ExceptionView } from "../bindings/ExceptionView.ts";
import type { TileView } from "../bindings/TileView.ts";

/** What the panel explains: one entry and the tiles and exceptions it refers to. */
export interface Explained {
  entry: EntryView;
  rowTile: TileView;
  columnTile: TileView;
  exceptions: ExceptionView[];
}

/**
 * The explanation panel (DR-0025 point 11). Every number comes from Rust;
 * this only lays out the product it was built from.
 */
@customElement("table-explanation")
export class TableExplanation extends LitElement {
  @property({ attribute: false }) explained?: Explained;
  @property() direction: Direction = "east";

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
    `,
  ];

  /**
   * The panel, then a slot for the host's action on the selected pair:
   * "Explore this tile" in the table, "Explore ┐" in the explore view
   * (DR-0025 point 13, DR-0026). The panel itself is the same in both.
   */
  render() {
    return html`${this.renderExplanation()}<slot class="actions"></slot>`;
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
    const exceptionLines =
      entry.exceptions.length === 0
        ? html`<li><span class="op">×</span> exceptions: none</li>`
        : entry.exceptions.map((i) => {
            const e = x.exceptions[i];
            return html`<li class="exception">
              <span class="op">×</span> ${e ? `${e.name} ${formatNumber(e.multiplier)}` : `exception ${i}`}
              ${e ? html`<div class="rule">${e.rule}</div>` : ""}
            </li>`;
          });

    return html`${header}
      <p class="share">${share}% of ${rowGlyph}'s ${this.direction} neighbours</p>
      ${facing}
      <h3>Where it comes from</h3>
      <ol class="sum">
        <li><span class="op"></span> ${sideName(entry.facing.row)}–${sideName(entry.facing.column)} base ${formatNumber(entry.base)}</li>
        <li><span class="op">×</span> ${rowTile.name} ${formatNumber(entry.rowWeight)}</li>
        <li><span class="op">×</span> ${columnTile.name} ${formatNumber(entry.columnWeight)}</li>
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
