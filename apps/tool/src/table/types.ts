// IPC shapes for the table view, as the frontend sees them.
//
// TEMPORARY: hand-written from DR-0024 ("The view command") while Rowan's
// ts-rs bindings are being built. DR-0023 says the frontend imports IPC
// shapes only from `src/bindings/`. When the bindings land, replace this
// file's body with re-exports, for example:
//
//   export type { TableView } from "../bindings/TableView.ts";
//
// Everything else imports from here, so that swap is the only change, and
// `tsc` will point at any field whose name differs.

export type Direction = "north" | "east" | "south" | "west";

export type TileKind = "corridor" | "room" | "hedge";

/** DR-0015's side kinds, with the wall end spelled out (DR-0024). */
export type SideKind =
  | "hedge"
  | "passage"
  | "door"
  | "floor"
  | "floor_wall_n"
  | "floor_wall_e"
  | "floor_wall_s"
  | "floor_wall_w";

/** Empty means any; a tile must match every non-empty field. */
export interface TileFilter {
  kinds: TileKind[];
  tileIds: string[];
  tags: string[];
}

export interface IndexWindow {
  start: number;
  count: number;
}

export interface TableViewRequest {
  rowFilter: TileFilter;
  columnFilter: TileFilter;
  /** Empty means all four. */
  directions: Direction[];
  rows: IndexWindow;
  columns: IndexWindow;
}

/** One (tile, orientation): a row and a column of the table. */
export interface TileInfo {
  id: string;
  name: string;
  kind: TileKind;
  tags: string[];
  /** Clockwise quarter turns from the sides as written (DR-0024). */
  orientation: number;
  /** N, E, S, W in this orientation. */
  sides: SideKind[];
  weight: number;
}

export interface ExceptionInfo {
  /** The rule as written in the data. */
  name: string;
  multiplier: number;
}

export interface TableEntry {
  direction: Direction;
  /** Index into `tiles`. */
  row: number;
  /** Index into `tiles`. */
  column: number;
  /** The row tile's side, then the column tile's side, where they touch. */
  facing: [SideKind, SideKind];
  allowed: boolean;
  base: number;
  rowWeight: number;
  columnWeight: number;
  /** Indices into `exceptions`. */
  exceptions: number[];
  /** Seeded noise factor; 1 when there is none. */
  noise: number;
  /** The entry: base × both weights × exceptions × noise (DR-0022, DR-0024). */
  weight: number;
  rowTotal: number;
  /** weight ÷ rowTotal, over all neighbours, never the filtered ones. */
  probability: number;
  /** For a 0: the side kinds the row tile's side could meet. */
  mayMeet: SideKind[];
}

export interface TableView {
  revision: number;
  tileSetPath: string;
  /** ISO 8601. */
  loadedAt: string;
  tiles: TileInfo[];
  totalRows: number;
  totalColumns: number;
  /** Indices into `tiles`, in display order. */
  rows: number[];
  /** Indices into `tiles`, in display order. */
  columns: number[];
  exceptions: ExceptionInfo[];
  entries: TableEntry[];
}

export interface ReloadResult {
  revision: number;
  tileSetPath: string;
  loadedAt: string;
}
