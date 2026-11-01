// Which sides of a stored cell are walkable (anything but hedge, DR-0036),
// for drawing sealed unknown pockets as hedge (Wren's G1, g1-fun-review).
//
// The sides at orientation 0 are copied from data/tile-sets/core.toml at
// cac4b35, N, E, S, W, true where the side isn't hedge. Orientation k turns
// the tile k quarter turns clockwise, so the side facing direction d is the
// base side (d − k) mod 4 (DR-0014, DR-0024). If the tile set's sides
// change, change this too; better still, Rowan's ViewCell can carry
// `open: [bool; 4]` and this file goes.

/** N, E, S, W. */
export type Sides = [boolean, boolean, boolean, boolean];

const BASE: Record<string, Sides> = {
  corridor_straight: [true, false, true, false],
  corridor_corner: [true, true, false, false],
  corridor_t_intersection: [true, true, false, true],
  corridor_crossroads: [true, true, true, true],
  corridor_dead_end: [true, false, false, false],
  open_room: [true, true, true, true],
  room_wall: [false, true, true, true],
  room_corner: [false, false, true, true],
  room_doorway: [true, true, true, true],
  hedge: [false, false, false, false],
};

/**
 * The walkable sides of (tile, orientation). An unknown tile counts as
 * open on every side, so a pocket next to it stays haze: the safe mistake.
 */
export function walkableSides(tile: string, orientation: number): Sides {
  const base = BASE[tile];
  if (!base) return [true, true, true, true];
  const k = ((orientation % 4) + 4) % 4;
  return [0, 1, 2, 3].map((d) => base[(d - k + 4) % 4]) as Sides;
}
