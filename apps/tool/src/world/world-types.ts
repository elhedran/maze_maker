// STUB of DR-0041's world IPC types, written by hand before Rowan's ts-rs
// bindings exist. Names and shapes are copied exactly from DR-0041's
// "IPC contract" section. Everything in src/world/ imports these types from
// this file only, so swapping in the generated bindings touches nothing else.
//
// The swap, once `src/bindings/` has the generated files: replace this whole
// file with
//
//   export type { WorldPos } from "../bindings/WorldPos.ts";
//   export type { WorldRect } from "../bindings/WorldRect.ts";
//   export type { WorldStartRequest } from "../bindings/WorldStartRequest.ts";
//   export type { WorldRef } from "../bindings/WorldRef.ts";
//   export type { WalkRequest } from "../bindings/WalkRequest.ts";
//   export type { Walk } from "../bindings/Walk.ts";
//   export type { ReplayStepRequest } from "../bindings/ReplayStepRequest.ts";
//   export type { WorldStarted } from "../bindings/WorldStarted.ts";
//   export type { WorldUpdate } from "../bindings/WorldUpdate.ts";
//   export type { PatchDelta } from "../bindings/PatchDelta.ts";
//   export type { StoredCell } from "../bindings/StoredCell.ts";
//   export type { Endpoint } from "../bindings/Endpoint.ts";
//   export type { ReplayUpdate } from "../bindings/ReplayUpdate.ts";
//   export type { ReplayCheck } from "../bindings/ReplayCheck.ts";
//
// then run `npx tsc --noEmit`: any drift between the stub and Rust shows up there.

import type { Direction } from "../bindings/Direction.ts";
import type { TileView } from "../bindings/TileView.ts";

export type WorldPos = { x: number, y: number, };
export type WorldRect = { x: number, y: number, width: number, height: number, };

export type WorldStartRequest = { seed: number, };
export type WorldRef = { worldId: number, };
export type WalkRequest = { worldId: number, walk: Walk, };
export type Walk = { "kind": "step", direction: Direction } | { "kind": "random", seed: number, steps: number };
export type ReplayStepRequest = { worldId: number, patches: number, };

export type WorldStarted = { worldId: number, seed: number, revision: number, loadedAt: string, tiles: Array<TileView>, update: WorldUpdate, };
export type WorldUpdate = { worldId: number, player: WorldPos, path: Array<WorldPos>, patches: Array<PatchDelta>, frontier: Array<Endpoint>, bounds: WorldRect, steps: number, patchCount: number, storedCount: number, };
export type PatchDelta = { index: number, step: number, anchor: WorldPos, from: Direction | null, window: WorldRect, attempts: number, spine: boolean, fallback: boolean, cells: Array<StoredCell>, spineCells: Array<WorldPos>, discarded: Array<WorldPos>, };
export type StoredCell = { x: number, y: number, tile: number, };
export type Endpoint = { cell: WorldPos, facing: Direction, anchor: WorldPos, outward: boolean, };
export type ReplayUpdate = { worldId: number, patches: Array<PatchDelta>, frontier: Array<Endpoint>, bounds: WorldRect | null, replayed: number, total: number, check: ReplayCheck | null, };
export type ReplayCheck = { same: boolean, cells: number, firstDifference: WorldPos | null, };
