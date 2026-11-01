// Copied from apps/tool/src/table/tile-glyphs.ts at 91db073 (DR-0043 point 5).
// A fix here is made in the tool's copy too, unless Damson says the game draws differently.

// DR-0017's presentation mapping: how the tool draws each (tile, orientation).
// Generation never sees this; code identifies tiles by id and orientation.
//
// Keys are DR-0024 tile ids, which are the DR-0015 names in snake_case.
// Each list is indexed by orientation k (clockwise quarter turns from the
// sides as written in the data), which DR-0024 keeps in DR-0017's order.

/** Where the room floor tint sits in the cell (DR-0017 rule 4). */
export type Tint =
  | "none"
  | "full"
  | "lower"
  | "left"
  | "upper"
  | "right"
  | "lower-left"
  | "upper-left"
  | "upper-right"
  | "lower-right";

export interface TileLook {
  glyph: string;
  tint: Tint;
  /** How this orientation reads, added to the tile's name: "open N and E". */
  facing: string;
}

const look = (glyph: string, tint: Tint, facing: string): TileLook => ({ glyph, tint, facing });

const LOOKS: Record<string, TileLook[]> = {
  corridor_straight: [look("│", "none", "N–S"), look("─", "none", "E–W")],
  corridor_corner: [
    look("└", "none", "open N and E"),
    look("┌", "none", "open E and S"),
    look("┐", "none", "open S and W"),
    look("┘", "none", "open W and N"),
  ],
  corridor_t_intersection: [
    look("┴", "none", "closed S"),
    look("├", "none", "closed W"),
    look("┬", "none", "closed N"),
    look("┤", "none", "closed E"),
  ],
  corridor_crossroads: [look("┼", "none", "")],
  corridor_dead_end: [
    look("╵", "none", "open N"),
    look("╶", "none", "open E"),
    look("╷", "none", "open S"),
    look("╴", "none", "open W"),
  ],
  open_room: [look("·", "full", "")],
  room_wall: [
    look("═", "lower", "wall N"),
    look("║", "left", "wall E"),
    look("═", "upper", "wall S"),
    look("║", "right", "wall W"),
  ],
  room_corner: [
    look("╗", "lower-left", "NE"),
    look("╝", "upper-left", "SE"),
    look("╚", "upper-right", "SW"),
    look("╔", "lower-right", "NW"),
  ],
  room_doorway: [
    look("╧", "lower", "door N"),
    look("╟", "left", "door E"),
    look("╤", "upper", "door S"),
    look("╢", "right", "door W"),
  ],
  hedge: [look("░", "none", "")],
};

/**
 * DR-0017 says never reuse a glyph for a tile that has none; the answer is
 * drawing from edges, which isn't built. Until then an unknown tile shows
 * "?" and its name, so the gap is visible rather than disguised.
 */
const UNKNOWN: TileLook = look("?", "none", "");

export function tileLook(id: string, orientation: number): TileLook {
  return LOOKS[id]?.[orientation] ?? UNKNOWN;
}

/** "Corridor corner, open N and E", or just "Hedge" for a single orientation. */
export function tileLabel(tile: { id: string; name: string; orientation: number }): string {
  const { facing } = tileLook(tile.id, tile.orientation);
  return facing ? `${tile.name}, ${facing}` : tile.name;
}
