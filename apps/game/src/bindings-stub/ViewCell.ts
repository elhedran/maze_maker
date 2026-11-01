// Hand-written stand-in for the ts-rs type Rowan generates into src/bindings/ (DR-0023).
// Same file and type name, so the swap is the import path in src/ipc.ts; then delete this folder.
export type ViewCell = {
  x: number;
  y: number;
  /** A DR-0024 tile id, such as "corridor_corner". */
  tile: string;
  /** Clockwise quarter turns, 0 to 3 (DR-0014). */
  orientation: number;
  /** "corridor" | "room" | "hedge" (DR-0015). */
  kind: string;
  /** The player has stood here (DR-0035). */
  visited: boolean;
};
