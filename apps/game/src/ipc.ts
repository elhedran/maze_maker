// The one place the game names Tauri commands (DR-0023). Components and
// their mockIPC tests call these functions, never bare `invoke` strings.

import { invoke } from "@tauri-apps/api/core";
// Hand-written until Rowan's ts-rs files land in ./bindings/; then this path changes.
import type { Direction } from "./bindings-stub/Direction.ts";
import type { GameView } from "./bindings-stub/GameView.ts";

export type { Direction, GameView };
export type { Pos } from "./bindings-stub/Pos.ts";
export type { ViewCell } from "./bindings-stub/ViewCell.ts";

export const COMMANDS = {
  startGame: "start_game",
  step: "step",
} as const;

/** A new world from `seed`, with the player at its start (DR-0035, DR-0036). */
export function startGame(seed: number): Promise<GameView> {
  return invoke<GameView>(COMMANDS.startGame, { seed });
}

/** One step. Rust grows the world as needed and says whether the player moved. */
export function step(direction: Direction): Promise<GameView> {
  return invoke<GameView>(COMMANDS.step, { direction });
}

/**
 * Turns a rejected command into text to show as given. Tauri commands reject
 * with the serialized `Err`, usually a string; anything else is shown as JSON
 * rather than lost. Simplified from apps/tool/src/ipc.ts.
 */
export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && typeof (error as { message?: unknown }).message === "string") {
    return (error as { message: string }).message;
  }
  return JSON.stringify(error);
}
