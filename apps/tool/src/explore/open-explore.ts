// The one way into the explore view (DR-0026 point 10). Any view sends this
// event; the app switches to the Explore tab and opens the view on it. The
// table's "Explore this tile" and the maze's "Why is this here?" use it.

import type { Direction } from "../bindings/Direction.ts";
import type { MazeCell, TileRef } from "./explore-data.ts";

export const OPEN_EXPLORE_EVENT = "open-explore";
/** The explore view asks the app for the Maze tab again (DR-0033 point 8). */
export const BACK_TO_MAZE_EVENT = "back-to-maze";

export interface ExploreTarget {
  /** The tile to put in the middle. */
  centre: TileRef;
  /** A neighbour to select, so the panel opens on that pair. */
  select?: { direction: Direction; neighbour: TileRef };
  /** The maze cell the tile came from; only then does the view show why it's there (DR-0033 point 7). */
  maze?: MazeCell;
}

/** Asks the app to open the explore view. It starts a new trail. */
export function openExplore(from: HTMLElement, target: ExploreTarget): void {
  from.dispatchEvent(
    new CustomEvent<ExploreTarget>(OPEN_EXPLORE_EVENT, { detail: target, bubbles: true, composed: true }),
  );
}
