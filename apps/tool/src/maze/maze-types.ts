// Local copies of the generate_maze IPC types (DR-0028's contract), until
// Rowan's ts-rs bindings land in `bindings/`. The names and fields match the
// Rust types, so the switch is an import change: replace this file's exports
// with re-exports from `../bindings/GeneratedMaze.ts` and friends, or point
// the imports there and delete this file.

export type GenerateRequest = {
  seed: number;
  width: number;
  height: number;
};

export type CellPos = { x: number; y: number };

export type Measure = {
  /** The label, e.g. "Reachable from centre". */
  name: string;
  /** Ready to show, e.g. "412 of 480 walkable cells (86%)". */
  value: string;
  explanation: string;
};

export type GeneratedMaze = {
  /** The tile set revision the maze was generated from. */
  revision: number;
  seed: number;
  width: number;
  height: number;
  /**
   * Row-major from the north-west. Each is an index into `table_view`'s
   * `tiles` at `revision`; null only where a failed attempt left a cell open.
   */
  cells: Array<number | null>;
  /** The cell the measures count from (DR-0030); it moves off hedge. */
  centre: CellPos;
  /** 0 for a clean run. */
  restarts: number;
  failed: boolean;
  measures: Array<Measure>;
};
