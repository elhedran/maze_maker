import { LitElement, css, html, nothing } from "lit";
import { customElement, property } from "lit/decorators.js";
import { glyphStyles, renderGlyph, themeStyles } from "../table/draw-tile.ts";
import { DIRECTIONS } from "../table/format.ts";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import { sameTile, type TileRef, type Why, type WhyCandidate, type WhyFactor } from "./explore-data.ts";
import {
  NOTHING_SETTLED,
  NO_CHOICE,
  STATE_WORD,
  chanceText,
  formatValue,
  formulaText,
  headingText,
  ruledOutText,
} from "./why-text.ts";
import type { Direction } from "../bindings/Direction.ts";

/** A factor by the candidate it belongs to and the counted neighbour's direction. */
export interface FactorKey {
  candidate: TileRef;
  direction: Direction;
}

export const FACTOR_SELECT_EVENT = "factor-select";

const glyphOf = (tile: TileRef) => tileLook(tile.id, tile.orientation).glyph;

/**
 * The "Why it's here" band (DR-0033 points 9 to 16): how WFC drew one maze
 * cell, replayed by Rust. Presentation only: the explore view owns the trace
 * and the selected factor, and a click on a factor is sent up as
 * `factor-select`. Every number is Rust's; nothing is multiplied or divided here.
 */
@customElement("why-band")
export class WhyBand extends LitElement {
  @property({ attribute: false }) why?: Why;
  /** Shown instead of a trace: replaying, stale, nothing placed. With a trace, shown under its heading instead of the table. */
  @property() note?: string;
  /** Added to the heading after a reload: "after reload; was ═" (DR-0037 point 17). */
  @property() suffix = "";
  @property({ attribute: false }) selected?: FactorKey;

  static styles = [
    themeStyles,
    glyphStyles,
    css`
      :host {
        display: block;
        padding: 0.75rem 1rem;
        border: 1px solid var(--frame);
        font-family: system-ui, sans-serif;
        font-size: 14px;
      }
      h3 { font-size: 14px; margin: 0 0 0.5rem; }
      p { margin: 0.4rem 0; }
      .muted { color: var(--muted); }
      .cross {
        display: grid;
        grid-template-areas:
          ". north ."
          "west cell east"
          ". south .";
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        justify-items: center;
        gap: 0.25rem 0.75rem;
        margin: 0.5rem 0;
        max-width: 28rem;
      }
      .cross .cell { grid-area: cell; }
      .cross .cell .glyph { outline: 2px solid var(--text); outline-offset: 1px; }
      .neighbour { display: flex; align-items: center; gap: 0.35rem; }
      .neighbour.north { grid-area: north; }
      .neighbour.south { grid-area: south; }
      .neighbour.west { grid-area: west; justify-self: end; flex-direction: row-reverse; }
      .neighbour.east { grid-area: east; justify-self: start; }
      .neighbour.later .glyph { opacity: 0.45; }
      .glyph.blank { border-style: dotted; }
      table { border-collapse: collapse; font-variant-numeric: tabular-nums; margin: 0.5rem 0; }
      th { font-weight: normal; color: var(--muted); text-align: right; padding: 0.15rem 0.5rem; }
      th:first-child { text-align: left; }
      td { padding: 0.15rem 0.5rem; text-align: right; white-space: nowrap; }
      td.candidate { text-align: left; }
      td.candidate .glyph { margin-right: 0.35rem; }
      tr.drawn td { font-weight: 700; }
      td.factor { cursor: pointer; border: 1px solid var(--frame); }
      td.factor:hover { background: var(--highlight); }
      td.factor.selected { outline: 2px solid var(--selection); outline-offset: -2px; }
      .formula { color: var(--muted); font-size: 12px; max-width: 40rem; }
      td.pressure.off { color: var(--muted); }
    `,
  ];

  render() {
    if (!this.why) return html`<p class="note">${this.note ?? ""}</p>`;
    const why = this.why;
    const glyph = glyphOf(why.tile);
    const counted = DIRECTIONS.filter((d) => why.neighbours.find((n) => n.direction === d)?.state === "counted");
    const anyLater = why.neighbours.some((n) => n.state === "later");
    // DR-0037 point 15: the pressure column only while the loaded pressure is below 1.
    const withPressure = why.openingPressure < 1;
    const heading = html`<h3 id="why-heading">
      ${headingText(glyph, why.seed, why.cell.x, why.cell.y, why.step, why.steps)}${this.suffix ? ` · ${this.suffix}` : ""}
    </h3>`;
    if (this.note) return html`${heading}<p class="note">${this.note}</p>`;
    return html`
      ${heading}
      ${this.renderCross(why)}
      ${counted.length === 0 ? html`<p class="nothing-settled">${NOTHING_SETTLED}</p>` : nothing}
      ${why.candidates.length === 1 ? html`<p class="no-choice">${NO_CHOICE}</p>` : nothing}
      <table class="candidates">
        <thead>
          <tr>
            <th>Candidate</th>
            <th>weight</th>
            ${counted.map((d) => {
              const n = why.neighbours.find((x) => x.direction === d)!;
              return html`<th data-direction=${d}>× ${d} ${n.tile ? glyphOf(n.tile) : ""}</th>`;
            })}
            ${withPressure ? html`<th class="pressure">× pressure</th>` : nothing}
            <th>= score</th>
            <th>chance</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          ${why.candidates.map((c) => this.renderCandidate(c, counted, withPressure))}
        </tbody>
      </table>
      <p class="ruled-out muted">${ruledOutText(why.ruledOut)}</p>
      <p class="formula">${formulaText(why.total, glyph, anyLater, withPressure)}</p>
    `;
  }

  /** The cell and its four neighbours as in the maze, each named in words (point 10). */
  private renderCross(why: Why) {
    return html`<div class="cross">
      <span class="cell">${renderGlyph(why.tile, tileLabel(why.tile))}</span>
      ${why.neighbours.map((n) => {
        const word = STATE_WORD[n.state];
        const label = n.tile ? `${tileLabel(n.tile)}, ${word}` : word;
        const glyph = n.tile ? renderGlyph(n.tile, label) : html`<span class="glyph blank" role="img" aria-label=${label}></span>`;
        return html`<span class="neighbour ${n.direction} ${n.state}" data-direction=${n.direction}>
          ${glyph}<span class="word">${n.direction}: ${word}</span>
        </span>`;
      })}
    </div>`;
  }

  private renderCandidate(c: WhyCandidate, counted: Direction[], withPressure: boolean) {
    const label = tileLabel(c.tile);
    return html`<tr class=${c.drawn ? "drawn" : ""} data-tile="${c.tile.id}#${c.tile.orientation}">
      <td class="candidate">${renderGlyph(c.tile, label)}${label}</td>
      <td>${formatValue(c.weight)}</td>
      ${counted.map((d) => {
        const f = c.factors.find((x) => x.direction === d);
        return f ? this.renderFactor(c, f) : html`<td></td>`;
      })}
      ${withPressure
        ? c.pressure === 1
          ? html`<td class="pressure off">1</td>`
          : html`<td class="pressure">× ${formatValue(c.pressure)}</td>`
        : nothing}
      <td>= ${formatValue(c.score)}</td>
      <td class="chance">${chanceText(c.share)}</td>
      <td class="drawn-word">${c.drawn ? "drawn" : ""}</td>
    </tr>`;
  }

  private renderFactor(c: WhyCandidate, f: WhyFactor) {
    const s = this.selected;
    const isSelected = !!s && s.direction === f.direction && sameTile(s.candidate, c.tile);
    const select = () =>
      this.dispatchEvent(
        new CustomEvent<FactorKey>(FACTOR_SELECT_EVENT, {
          detail: { candidate: { id: c.tile.id, orientation: c.tile.orientation }, direction: f.direction },
        }),
      );
    return html`<td
      class="factor ${isSelected ? "selected" : ""}"
      data-direction=${f.direction}
      tabindex="0"
      title="${tileLabel(c.tile)} then ${tileLabel(f.neighbour)} to its ${f.direction}: see where this factor comes from"
      @click=${select}
      @keydown=${(e: KeyboardEvent) => e.key === "Enter" && select()}
    >
      × ${formatValue(f.factor)}
    </td>`;
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "why-band": WhyBand;
  }
}
