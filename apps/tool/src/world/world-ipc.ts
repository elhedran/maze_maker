// The world view's four Tauri commands (DR-0041). DR-0041's build plan puts
// these in ../ipc.ts; they live here instead while a parallel 2.3 session
// edits ipc.ts, and can be folded into its COMMANDS once that lands.

import { invoke } from "@tauri-apps/api/core";
import type {
  ReplayStepRequest,
  ReplayUpdate,
  WalkRequest,
  WorldRef,
  WorldStartRequest,
  WorldStarted,
  WorldUpdate,
} from "./world-types.ts";

export const WORLD_COMMANDS = {
  worldStart: "world_start",
  worldWalk: "world_walk",
  worldReplayStart: "world_replay_start",
  worldReplayStep: "world_replay_step",
} as const;

/** A new world from a seed, on the loaded tile set, with the origin patch and growth around it. */
export function worldStart(request: WorldStartRequest): Promise<WorldStarted> {
  return invoke<WorldStarted>(WORLD_COMMANDS.worldStart, { request });
}

/** One key step, or seeded random steps; returns the player and the patches the walk grew. */
export function worldWalk(request: WalkRequest): Promise<WorldUpdate> {
  return invoke<WorldUpdate>(WORLD_COMMANDS.worldWalk, { request });
}

/** Starts regrowing the world from its growth log into a fresh world. */
export function worldReplayStart(request: WorldRef): Promise<ReplayUpdate> {
  return invoke<ReplayUpdate>(WORLD_COMMANDS.worldReplayStart, { request });
}

/** Regrows the next `patches` patches of the log; carries Rust's check when replay ends. */
export function worldReplayStep(request: ReplayStepRequest): Promise<ReplayUpdate> {
  return invoke<ReplayUpdate>(WORLD_COMMANDS.worldReplayStep, { request });
}
