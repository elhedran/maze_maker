// Hand-written stand-in for the ts-rs type Rowan generates into src/bindings/ (DR-0023).
// Same file and type name, so the swap is the import path in src/ipc.ts; then delete this folder.
import type { Pos } from "./Pos.ts";
import type { ViewCell } from "./ViewCell.ts";

export type GameView = {
  player: Pos;
  /** False when the step was blocked and the player stayed put. */
  moved: boolean;
  /** Cells within this many steps of the player (each axis) are sent. */
  radius: number;
  /** The stored cells within `radius`; anything missing is ungrown. */
  cells: ViewCell[];
};
