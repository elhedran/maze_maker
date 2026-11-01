// The one way into the explore view (DR-0026 point 10). Any view sends this
// event; the app switches to the Explore tab and opens the view on it. The
// table's "Explore this tile" uses it now, and 2.2's link from a maze cell
// will use it too.

import type { Direction } from "../bindings/Direction.ts";
import type { TileRef } from "./explore-data.ts";

export const OPEN_EXPLORE_EVENT = "open-explore";

export interface ExploreTarget {
  /** The tile to put in the middle. */
  centre: TileRef;
  /** A neighbour to select, so the panel opens on that pair. */
  select?: { direction: Direction; neighbour: TileRef };
}

/** Asks the app to open the explore view. It starts a new trail. */
export function openExplore(from: HTMLElement, target: ExploreTarget): void {
  from.dispatchEvent(
    new CustomEvent<ExploreTarget>(OPEN_EXPLORE_EVENT, { detail: target, bubbles: true, composed: true }),
  );
}
