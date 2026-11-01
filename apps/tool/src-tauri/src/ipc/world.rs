//! IPC types for the World tab (DR-0041, the view in DR-0039).
//!
//! Positions are `i32`, so ts-rs writes them as `number` (DR-0023: no
//! 64-bit integers over IPC). Core keeps `i64`; the world's step cap keeps
//! every position far inside `i32` (DR-0041 rule 10).

use super::{Direction, TileView};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A world cell: x grows east, y grows south, the origin is where the
/// player spawns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldPos {
    pub x: i32,
    pub y: i32,
}

/// A box of world cells with its north-west corner at `x, y`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Starts a new world from the loaded tile set, replacing any world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldStartRequest {
    pub seed: u32,
}

/// Names the world a request is for. A world replaced since is `stale`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldRef {
    pub world_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WalkRequest {
    pub world_id: u32,
    pub walk: Walk,
}

/// How to walk. `step` moves one cell if the neighbour that way is stored
/// and joined; otherwise nothing moves and nothing grows. `random` is
/// `steps` (1 to 1,000) steps of the seeded random walk; the same `seed`
/// as the last random walk carries its stream on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Walk {
    Step { direction: Direction },
    Random { seed: u32, steps: u32 },
}

/// Replays the next `patches` (1 or more, clamped to what's left) patches
/// of the growth log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReplayStepRequest {
    pub world_id: u32,
    pub patches: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldStarted {
    pub world_id: u32,
    pub seed: u32,
    /// The tile set load the world was grown with, as in `TableView`.
    pub revision: u32,
    pub loaded_at: String,
    /// The world's own tiles, in variant order. `StoredCell.tile` indexes
    /// this list, never `TableView.tiles`, so a Reload can't renumber them.
    pub tiles: Vec<TileView>,
    pub update: WorldUpdate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WorldUpdate {
    pub world_id: u32,
    /// Where the walker stands after this request.
    pub player: WorldPos,
    /// The cells this request walked, in order, after the starting cell.
    /// Empty for `world_start` and for a blocked key step.
    pub path: Vec<WorldPos>,
    /// This request's patches, in growth order.
    pub patches: Vec<PatchDelta>,
    /// The whole frontier after this request's last patch.
    pub frontier: Vec<Endpoint>,
    /// The stored cells' box.
    pub bounds: WorldRect,
    /// The world's totals so far.
    pub steps: u32,
    pub patch_count: u32,
    pub stored_count: u32,
}

/// One patch: its growth log entry, the cells it stored, and its debug data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PatchDelta {
    /// Its place in the growth log, from 0.
    pub index: u32,
    /// The walk step after which it grew; 0 for `world_start`'s.
    pub step: u32,
    pub anchor: WorldPos,
    /// From the stored cell to the anchor; `null` only for the origin patch.
    pub from: Option<Direction>,
    /// P × P.
    pub window: WorldRect,
    /// WFC runs made, 1 for a clean first run.
    pub attempts: u32,
    pub spine: bool,
    pub fallback: bool,
    /// The stored cells, in the window's row-major order.
    pub cells: Vec<StoredCell>,
    /// From the anchor out; empty without a spine.
    pub spine_cells: Vec<WorldPos>,
    /// Window cells the run filled that the patch didn't keep, row-major.
    pub discarded: Vec<WorldPos>,
    /// Cells it stored as the seal tile because nothing can ever reach them
    /// (DR-0047), in (y, x) order; never in `cells`, and may lie outside
    /// the window. Stored like `cells`, so the view draws them the same way.
    pub sealed: Vec<StoredCell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StoredCell {
    pub x: i32,
    pub y: i32,
    /// Index into `WorldStarted.tiles`.
    pub tile: u32,
}

/// An open endpoint: stored `cell`'s walkable side facing `facing`, onto
/// the unsettled `anchor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub cell: WorldPos,
    pub facing: Direction,
    pub anchor: WorldPos,
    /// The anchor connects through unsettled cells to the unbounded region.
    pub outward: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReplayUpdate {
    pub world_id: u32,
    /// This request's replayed patches, in growth order.
    pub patches: Vec<PatchDelta>,
    pub frontier: Vec<Endpoint>,
    /// `null` until the first patch is replayed.
    pub bounds: Option<WorldRect>,
    pub replayed: u32,
    pub total: u32,
    /// `null` until replay ends: when `replayed == total`, or earlier if
    /// the replayed world diverged from the walked one (then `same` is
    /// false). A non-null `check` means replay is over.
    pub check: Option<ReplayCheck>,
}

/// Rust's answer to "same world as walked?".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReplayCheck {
    /// Same cells, same tiles, and the same growth log, discards included.
    pub same: bool,
    /// How many stored cells matched.
    pub cells: u32,
    /// The first cell, in (y, x) order, where the worlds differ.
    pub first_difference: Option<WorldPos>,
}
