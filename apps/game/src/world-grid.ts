// Lays a GameView out as rows of screen cells centred on the player.
// Pure, so the layout is tested without a DOM; the component only draws it.

import type { GameView, ViewCell } from "./ipc.ts";
import { tileLook, type Tint } from "./tile-glyphs.ts";
import { eastFiller } from "./east-filler.ts";

export interface ScreenCell {
  /** World position, for labels and tests. */
  x: number;
  y: number;
  /** Undefined where nothing is stored yet: "not yet known" (DR-0044 rule 5, DR-0045 point 7). */
  cell?: ViewCell;
  /** The first character: the tile glyph, or `@` on the player (DR-0045 point 4). */
  glyph: string;
  /** The second character, carrying the glyph's east side (DR-0029 point 7). */
  filler: string;
  tint: Tint;
  player: boolean;
}

/**
 * A (2r+1)-square window with the player in the middle (DR-0045 point 3).
 * Rows run north to south, so world y grows downwards, as the tool draws
 * its mazes. Cells the view doesn't send are not yet known.
 */
export function layoutGrid(view: GameView): ScreenCell[][] {
  const { player, radius } = view;
  const byPos = new Map<string, ViewCell>();
  for (const cell of view.cells) byPos.set(`${cell.x},${cell.y}`, cell);
  const rows: ScreenCell[][] = [];
  for (let y = player.y - radius; y <= player.y + radius; y++) {
    const row: ScreenCell[] = [];
    for (let x = player.x - radius; x <= player.x + radius; x++) {
      const cell = byPos.get(`${x},${y}`);
      const isPlayer = x === player.x && y === player.y;
      row.push({ x, y, cell, player: isPlayer, ...cellLook(cell, isPlayer) });
    }
    rows.push(row);
  }
  return rows;
}

function cellLook(cell: ViewCell | undefined, isPlayer: boolean): Pick<ScreenCell, "glyph" | "filler" | "tint"> {
  if (!cell) return { glyph: isPlayer ? "@" : " ", filler: " ", tint: "none" };
  const look = tileLook(cell.tile, cell.orientation);
  // The player replaces the glyph; the cell keeps its tint and east filler,
  // so the way east still joins (DR-0045 point 4).
  return { glyph: isPlayer ? "@" : look.glyph, filler: eastFiller(look.glyph), tint: look.tint };
}

/** "corridor_corner" → "Corridor corner", for labels; the game has no tile names yet. */
export function tileName(id: string): string {
  const words = id.replace(/_/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}

/** The screen reader label of one cell. */
export function cellLabel(c: ScreenCell): string {
  const what = c.cell ? tileName(c.cell.tile) : "Not yet known";
  const marks = [c.player ? "you are here" : "", c.cell?.visited && !c.player ? "visited" : ""].filter(Boolean);
  return [what, ...marks].join(", ");
}
