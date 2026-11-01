// What the world view has on screen: the stored cells revealed so far, the
// patches they came from, the frontier and the player. Pure data, so the
// growth order and the lookups are tested without a DOM. Rust grows the world;
// this only collects the deltas it sends (DR-0041).

import type { Endpoint, PatchDelta, WorldPos } from "./world-types.ts";

export interface ShownCell {
  /** Index into the world's own tile list (`WorldStarted.tiles`). */
  tile: number;
  /** The growth-log index of the patch that stored it. */
  patch: number;
}

/** Map key for a world cell. Positions fit in i32 (DR-0041 rule 10), so this is exact. */
export function cellKey(x: number, y: number): string {
  return `${x},${y}`;
}

export class WorldModel {
  readonly cells = new Map<string, ShownCell>();
  readonly patches: PatchDelta[] = [];
  frontier: Endpoint[] = [];
  /** Open endpoints by anchor cell; an outward one wins when two share an anchor. */
  private endpointsByAnchor = new Map<string, Endpoint>();
  player?: WorldPos;

  /** Adds one patch's stored cells, in growth order, with the cells it sealed as hedge (DR-0047). */
  addPatch(patch: PatchDelta): void {
    this.patches.push(patch);
    for (const cell of [...patch.cells, ...patch.sealed]) {
      this.cells.set(cellKey(cell.x, cell.y), { tile: cell.tile, patch: patch.index });
    }
  }

  setFrontier(frontier: Endpoint[]): void {
    this.frontier = frontier;
    this.endpointsByAnchor.clear();
    for (const e of frontier) {
      const key = cellKey(e.anchor.x, e.anchor.y);
      const seen = this.endpointsByAnchor.get(key);
      if (!seen || (!seen.outward && e.outward)) this.endpointsByAnchor.set(key, e);
    }
  }

  cellAt(x: number, y: number): ShownCell | undefined {
    return this.cells.get(cellKey(x, y));
  }

  endpointAt(x: number, y: number): Endpoint | undefined {
    return this.endpointsByAnchor.get(cellKey(x, y));
  }

  get lastPatch(): PatchDelta | undefined {
    return this.patches[this.patches.length - 1];
  }

  get outwardCount(): number {
    return this.frontier.filter((e) => e.outward).length;
  }
}

/** The map's size in cells: DR-0039 rule 5's minimum, fixed (a build call, see DR-0039 Concerns). */
export const VIEW_WIDTH = 40;
export const VIEW_HEIGHT = 24;
/** Follow player re-centres when the player comes this close to an edge (DR-0039 rule 5). */
export const FOLLOW_MARGIN = 6;
/** Shift+arrows pan this many cells. */
export const PAN_STEP = 8;

/** The view's north-west corner that puts `at` in the middle. */
export function centredOn(at: WorldPos): WorldPos {
  return { x: at.x - Math.floor(VIEW_WIDTH / 2), y: at.y - Math.floor(VIEW_HEIGHT / 2) };
}

/**
 * Follow player: the same corner while the player is at least FOLLOW_MARGIN
 * cells inside every edge, otherwise re-centred on the player.
 */
export function followed(corner: WorldPos, player: WorldPos): WorldPos {
  const dx = player.x - corner.x;
  const dy = player.y - corner.y;
  const inside =
    dx >= FOLLOW_MARGIN && dx < VIEW_WIDTH - FOLLOW_MARGIN && dy >= FOLLOW_MARGIN && dy < VIEW_HEIGHT - FOLLOW_MARGIN;
  return inside ? corner : centredOn(player);
}
