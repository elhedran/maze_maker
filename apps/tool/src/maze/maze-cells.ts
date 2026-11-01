// Drawing a maze's cells (DR-0017, DR-0029 point 7), shared by the single
// maze and both sides of the compare (DR-0037 point 10).

import { css, html } from "lit";
import { tileLabel, tileLook } from "../table/tile-glyphs.ts";
import { eastFiller } from "./maze-text.ts";
import type { GeneratedMaze } from "../bindings/GeneratedMaze.ts";
import type { TileView } from "../bindings/TileView.ts";

/** The tile at a cell, or undefined for an open cell (or an index outside the tile list). */
export function tileAt(maze: GeneratedMaze, tiles: TileView[] | undefined, x: number, y: number): TileView | undefined {
  const index = maze.cells[y * maze.width + x];
  return index === null || index === undefined ? undefined : tiles?.[index];
}

/** `marks(x, y)` gives a cell's extra classes: outlines, fading. */
export function renderRows(maze: GeneratedMaze, tiles: TileView[] | undefined, marks: (x: number, y: number) => string) {
  const rows = [];
  for (let y = 0; y < maze.height; y++) {
    const cells = [];
    for (let x = 0; x < maze.width; x++) cells.push(renderCell(tileAt(maze, tiles, x, y), x, y, marks(x, y)));
    rows.push(html`<span class="row">${cells}</span>`);
  }
  return rows;
}

function renderCell(tile: TileView | undefined, x: number, y: number, mark: string) {
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

/** The cell under a pointer event on a map, if any. */
export function cellAt(event: Event): { x: number; y: number } | undefined {
  const cell = (event.target as HTMLElement).closest<HTMLElement>(".cell");
  return cell ? { x: Number(cell.dataset.x), y: Number(cell.dataset.y) } : undefined;
}

/**
 * A box about the size the maze will be: two map characters by one line per
 * cell. `ch` is the map's monospace width; 1.2em approximates its natural line height.
 */
export function boxStyle(width: number, height: number): string {
  return `width: ${width * 2}ch; height: ${height * 1.2}em`;
}

export const mapStyles = css`
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
  /* The selected cell, in the selection colour, on top of the centre mark (DR-0033 point 1). */
  .cell.selected { outline: 2px solid var(--selection); outline-offset: -2px; }
`;
