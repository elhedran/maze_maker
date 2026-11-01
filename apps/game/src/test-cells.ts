// Test data only: builds ViewCells as Rust would send them, for tests that
// write a view by hand. The real `open` sides come from Rust, from the
// loaded tile set (ViewCell.open); nothing outside tests imports this.
//
// The sides at orientation 0 are copied from data/tile-sets/core.toml at
// cac4b35, N, E, S, W, true where the side isn't hedge. Orientation k turns
// the tile k quarter turns clockwise, so the side facing direction d is the
// base side (d - k) mod 4 (DR-0014, DR-0024). They only need to be
// plausible for the tests, not to track the tile set.

import type { TileKind, ViewCell } from "./ipc.ts";

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

/** The walkable sides of (tile, orientation); an unknown tile is open all round. */
function walkableSides(tile: string, orientation: number): Sides {
  const base = BASE[tile];
  if (!base) return [true, true, true, true];
  const k = ((orientation % 4) + 4) % 4;
  return [0, 1, 2, 3].map((d) => base[(d - k + 4) % 4]) as Sides;
}

/** A ViewCell as Rust would send it, with `open` from the table above. */
export function cell(
  x: number,
  y: number,
  tile: string,
  orientation: number,
  kind: TileKind,
  visited = false,
): ViewCell {
  return { x, y, tile, orientation, kind, open: walkableSides(tile, orientation), visited };
}
