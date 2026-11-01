// The one place the frontend names Tauri commands (DR-0023). Components and
// their mockIPC tests call these functions, never bare `invoke` strings.

import { invoke } from "@tauri-apps/api/core";
import type { Direction } from "./bindings/Direction.ts";
import type { ReloadResult } from "./bindings/ReloadResult.ts";
import type { TableView } from "./bindings/TableView.ts";
import type { TableViewRequest } from "./bindings/TableViewRequest.ts";
import type { ToolError } from "./bindings/ToolError.ts";
import type { GenerateRequest } from "./bindings/GenerateRequest.ts";
import type { GeneratedMaze } from "./bindings/GeneratedMaze.ts";
// DR-0034's types, stubbed until Rowan's bindings land.
import type { CellTrace, ExploreRequest, ExploreView, TraceCellRequest } from "./explore/contract-types.ts";

export const COMMANDS = {
  coreVersion: "core_version",
  tableView: "table_view",
  reloadTileSet: "reload_tile_set",
  generateMaze: "generate_maze",
  exploreTile: "explore_tile",
  traceCell: "trace_cell",
} as const;

/** One tile's four arms, sorted, with their 0s split off (DR-0034 rules 1 to 3). */
export function exploreTile(request: ExploreRequest): Promise<ExploreView> {
  return invoke<ExploreView>(COMMANDS.exploreTile, { request });
}

/** Replays a maze's seed and reports how one cell was drawn (DR-0034 rules 4 to 7). */
export function traceCell(request: TraceCellRequest): Promise<CellTrace> {
  return invoke<CellTrace>(COMMANDS.traceCell, { request });
}

/** One sample maze from the loaded tile set (DR-0028). */
export function generateMaze(request: GenerateRequest): Promise<GeneratedMaze> {
  return invoke<GeneratedMaze>(COMMANDS.generateMaze, { request });
}

export function coreVersion(): Promise<string> {
  return invoke<string>(COMMANDS.coreVersion);
}

export function tableView(request: TableViewRequest): Promise<TableView> {
  return invoke<TableView>(COMMANDS.tableView, { request });
}

/** Re-reads the tile set file. On failure Rust keeps the last good table. */
export function reloadTileSet(): Promise<ReloadResult> {
  return invoke<ReloadResult>(COMMANDS.reloadTileSet);
}

// The largest u32, meaning "all of them". Rust clamps the window to the
// filtered rows and columns, and IPC types carry no 64-bit numbers (DR-0023).
const ALL = 0xffff_ffff;

/** Slice 1.1 asks for the whole unfiltered table, one direction at a time. */
export function wholeTableRequest(direction: Direction): TableViewRequest {
  const anyTile = () => ({ kinds: [], tileIds: [], tags: [], sides: [] });
  return {
    rowFilter: anyTile(),
    columnFilter: anyTile(),
    directions: [direction],
    rows: { start: 0, count: ALL },
    columns: { start: 0, count: ALL },
  };
}

/**
 * Just the tile list and the load details: an empty window, so no entries.
 * `tiles` is always the whole set in table order, whatever the window.
 */
export function tilesOnlyRequest(): TableViewRequest {
  const anyTile = () => ({ kinds: [], tileIds: [], tags: [], sides: [] });
  return {
    rowFilter: anyTile(),
    columnFilter: anyTile(),
    directions: ["east"],
    rows: { start: 0, count: 0 },
    columns: { start: 0, count: 0 },
  };
}

/**
 * Turns a rejected command into text to show as given. Our commands reject
 * with a `ToolError` (one readable message per load error); Tauri itself can
 * reject with a string, and anything else is shown as JSON rather than lost.
 */
export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (Array.isArray(error)) return error.map(errorText).join("\n");
  if (error && typeof error === "object") {
    const e = error as Partial<ToolError> & { message?: unknown };
    if (Array.isArray(e.messages)) return e.messages.join("\n");
    if (typeof e.message === "string") return e.message;
  }
  return JSON.stringify(error);
}
