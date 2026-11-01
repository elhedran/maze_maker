// Lays a GameView out as rows of screen cells centred on the player.
// Pure, so the layout is tested without a DOM; the component only draws it.

import type { GameView, ViewCell } from "./ipc.ts";
import { tileLook, type Tint } from "./tile-glyphs.ts";
import { eastFiller } from "./east-filler.ts";
import { walkableSides } from "./walkable-sides.ts";

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
  /**
   * Not yet known, but sealed: drawn as hedge (Wren's G1). No stored cell
   * faces it with a walkable side and it doesn't reach the view's edge, so
   * no patch can ever grow into it (DR-0036 rules 2 and 4).
   */
  sealed: boolean;
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
      row.push({ x, y, cell, player: isPlayer, sealed: false, ...cellLook(cell, isPlayer) });
    }
    rows.push(row);
  }
  markSealed(rows);
  return rows;
}

// Row and column steps for N, E, S, W, matching walkableSides' order.
const STEPS: [number, number][] = [
  [-1, 0],
  [0, 1],
  [1, 0],
  [0, -1],
];

/**
 * Finds each pocket of unknown cells (4-connected) in the view and marks it
 * sealed if it touches no edge of the view and no stored cell shows it a
 * walkable side. A pocket on the edge may continue outside the view, so it
 * stays haze; so does anything an open endpoint faces.
 */
function markSealed(rows: ScreenCell[][]): void {
  const size = rows.length;
  const seen = rows.map((r) => r.map(() => false));
  for (let r0 = 0; r0 < size; r0++) {
    for (let c0 = 0; c0 < size; c0++) {
      if (seen[r0][c0] || rows[r0][c0].cell) continue;
      const pocket: ScreenCell[] = [];
      let open = false;
      const stack: [number, number][] = [[r0, c0]];
      seen[r0][c0] = true;
      while (stack.length) {
        const [r, c] = stack.pop()!;
        pocket.push(rows[r][c]);
        if (r === 0 || c === 0 || r === size - 1 || c === size - 1) open = true;
        // The player stands somewhere walkable, whatever the view says.
        if (rows[r][c].player) open = true;
        STEPS.forEach(([dr, dc], d) => {
          const nr = r + dr;
          const nc = c + dc;
          if (nr < 0 || nc < 0 || nr >= size || nc >= size) return;
          const next = rows[nr][nc];
          if (next.cell) {
            // The neighbour's side facing back towards this pocket cell.
            if (walkableSides(next.cell.tile, next.cell.orientation)[(d + 2) % 4]) open = true;
          } else if (!seen[nr][nc]) {
            seen[nr][nc] = true;
            stack.push([nr, nc]);
          }
        });
      }
      if (!open) {
        for (const p of pocket) {
          p.sealed = true;
          p.glyph = "░";
          p.filler = "░";
        }
      }
    }
  }
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
  const what = c.cell ? tileName(c.cell.tile) : c.sealed ? "Hedge" : "Not yet known";
  const marks = [c.player ? "you are here" : "", c.cell?.visited && !c.player ? "visited" : ""].filter(Boolean);
  return [what, ...marks].join(", ");
}
