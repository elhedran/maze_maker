// The one place the frontend names Tauri commands (DR-0023). Components and
// their mockIPC tests call these functions, never bare `invoke` strings.

import { invoke } from "@tauri-apps/api/core";
import type { Direction } from "./bindings/Direction.ts";
import type { ReloadResult } from "./bindings/ReloadResult.ts";
import type { TableView } from "./bindings/TableView.ts";
import type { TableViewRequest } from "./bindings/TableViewRequest.ts";
import type { ToolError } from "./bindings/ToolError.ts";

export const COMMANDS = {
  coreVersion: "core_version",
  tableView: "table_view",
  reloadTileSet: "reload_tile_set",
} as const;

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
  const anyTile = () => ({ kinds: [], tileIds: [], tags: [] });
  return {
    rowFilter: anyTile(),
    columnFilter: anyTile(),
    directions: [direction],
    rows: { start: 0, count: ALL },
    columns: { start: 0, count: ALL },
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
