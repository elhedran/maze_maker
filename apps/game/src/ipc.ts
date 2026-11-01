// The one place the game names Tauri commands (DR-0023). Components and
// their mockIPC tests call these functions, never bare `invoke` strings.

import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
// Hand-written until Rowan's ts-rs files land in ./bindings/; then these paths change.
import type { Direction } from "./bindings-stub/Direction.ts";
import type { GameView } from "./bindings-stub/GameView.ts";
import type { GameError } from "./bindings-stub/GameError.ts";

export type { Direction, GameView, GameError };
export type { Pos } from "./bindings-stub/Pos.ts";
export type { ViewCell } from "./bindings-stub/ViewCell.ts";
export type { TileKind } from "./bindings-stub/TileKind.ts";

export const COMMANDS = {
  initialSeed: "initial_seed",
  startGame: "start_game",
  step: "step",
} as const;

/**
 * The seed to launch with: the one given at launch, or a random one (Rust
 * picks, because only Rust sees the command line; DR-0045 Marram's concern).
 */
export function initialSeed(): Promise<number> {
  return invoke<number>(COMMANDS.initialSeed);
}

/** A new world from `seed`, with the player at its start (DR-0043 point 7). */
export function startGame(seed: number): Promise<GameView> {
  return invoke<GameView>(COMMANDS.startGame, { seed });
}

/** One step. Rust grows the world as needed and says whether the player moved. */
export function step(direction: Direction): Promise<GameView> {
  return invoke<GameView>(COMMANDS.step, { direction });
}

/**
 * Asks Tauri to focus the window, because WebView2 can start without
 * keyboard focus (DR-0045 point 8). Needs `core:window:allow-set-focus`.
 * Best effort: if it fails, keys still work once the window has focus.
 */
export async function focusWindow(): Promise<void> {
  try {
    await getCurrentWindow().setFocus();
  } catch {
    // Outside Tauri, or the permission is missing. Nothing to show the player.
  }
  window.focus();
}

/**
 * Turns a rejected command into text to show as given. Commands reject with
 * a `GameError`; Tauri itself can reject with a string, and anything else is
 * shown as JSON rather than lost.
 */
export function errorText(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && typeof (error as Partial<GameError>).message === "string") {
    return (error as GameError).message;
  }
  return JSON.stringify(error);
}
