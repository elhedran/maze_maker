// Test data for the world view: a few tiles and a patch builder. Shapes are
// DR-0041's, so the same tests run unchanged against the generated bindings.

import type { TileView } from "../bindings/TileView.ts";
import type { PatchDelta } from "./world-types.ts";

const tile = (id: string, name: string, kind: TileView["kind"], orientation: number, sides: TileView["sides"]): TileView => ({
  id,
  name,
  kind,
  tags: [],
  orientation,
  sides,
  weight: 1,
});

/** 0 ─ (open E and W), 1 │ (open N and S), 2 ┼, 3 · open room, 4 ░ hedge. */
export const TILES: TileView[] = [
  tile("corridor_straight", "Corridor straight", "corridor", 1, ["hedge", "passage", "hedge", "passage"]),
  tile("corridor_straight", "Corridor straight", "corridor", 0, ["passage", "hedge", "passage", "hedge"]),
  tile("corridor_crossroads", "Corridor crossroads", "corridor", 0, ["passage", "passage", "passage", "passage"]),
  tile("open_room", "Open room", "room", 0, ["floor", "floor", "floor", "floor"]),
  tile("hedge", "Hedge", "hedge", 0, ["hedge", "hedge", "hedge", "hedge"]),
];

/** A patch with these [x, y, tile] cells (and sealed cells); the rest of its fields are plain defaults. */
export function patch(
  index: number,
  cells: [number, number, number][],
  step = 0,
  sealed: [number, number, number][] = [],
): PatchDelta {
  return {
    index,
    step,
    anchor: { x: 0, y: 0 },
    from: null,
    window: { x: -2, y: -2, width: 5, height: 5 },
    attempts: 1,
    spine: false,
    fallback: false,
    cells: cells.map(([x, y, t]) => ({ x, y, tile: t })),
    spineCells: [],
    discarded: [],
    sealed: sealed.map(([x, y, t]) => ({ x, y, tile: t })),
  };
}
