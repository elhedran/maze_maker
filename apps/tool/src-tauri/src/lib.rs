//! The adjacency tool's Tauri app: commands and IPC types. Domain logic lives
//! in `maze-core` (DR-0012).
//!
//! Commands (DR-0024):
//! - `table_view(request: TableViewRequest) -> Result<TableView, ToolError>`
//! - `reload_tile_set() -> Result<ReloadResult, ToolError>`
//! - `generate_maze(request: GenerateRequest) -> Result<GeneratedMaze, ToolError>`
//!   (DR-0028)
//! - `explore_tile(request: ExploreRequest) -> Result<ExploreView, ToolError>`
//!   (DR-0034)
//! - `trace_cell(request: TraceCellRequest) -> Result<CellTrace, ToolError>`
//!   (DR-0034)
//! - `world_start`, `world_walk`, `world_replay_start`, `world_replay_step`:
//!   the World tab, in `world.rs` (DR-0041)
//! - `core_version() -> String`

pub mod ipc;
pub mod maze;
pub mod state;
pub mod view;
pub mod world;

use ipc::{
    CellTrace, ExploreRequest, ExploreView, GenerateRequest, GeneratedMaze, ReloadResult,
    TableView, TableViewRequest, ToolError, TraceCellRequest,
};
use state::ToolState;
use std::sync::Mutex;

/// Returns `maze-core`'s version, proving the frontend -> Tauri -> core path.
#[tauri::command]
fn core_version() -> String {
    maze_core::version().to_string()
}

/// A poisoned lock means a panic mid-update; the state is replaced whole on
/// each load, so it is still consistent and safe to keep using.
fn lock(state: &Mutex<ToolState>) -> std::sync::MutexGuard<'_, ToolState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
fn table_view(
    state: tauri::State<'_, Mutex<ToolState>>,
    request: TableViewRequest,
) -> Result<TableView, ToolError> {
    lock(&state).table_view(&request)
}

#[tauri::command]
fn reload_tile_set(state: tauri::State<'_, Mutex<ToolState>>) -> Result<ReloadResult, ToolError> {
    lock(&state).reload(&state::default_tile_set_path())
}

/// A 24 × 16 maze takes a few milliseconds in release, so it runs in the
/// command directly rather than on a worker.
#[tauri::command]
fn generate_maze(
    state: tauri::State<'_, Mutex<ToolState>>,
    request: GenerateRequest,
) -> Result<GeneratedMaze, ToolError> {
    lock(&state).generate(&request)
}

#[tauri::command]
fn explore_tile(
    state: tauri::State<'_, Mutex<ToolState>>,
    request: ExploreRequest,
) -> Result<ExploreView, ToolError> {
    lock(&state).explore(&request)
}

/// Replays the maze to trace one cell: one generation's work, so like
/// `generate_maze` it runs in the command directly.
#[tauri::command]
fn trace_cell(
    state: tauri::State<'_, Mutex<ToolState>>,
    request: TraceCellRequest,
) -> Result<CellTrace, ToolError> {
    lock(&state).trace(&request)
}

pub fn run() {
    // A failed startup load doesn't stop the app: `table_view` returns the
    // errors so they show in the tool (DR-0024).
    let mut initial = ToolState::default();
    if let Err(e) = initial.reload(&state::default_tile_set_path()) {
        eprintln!("Tile set failed to load:\n  {}", e.messages.join("\n  "));
    }
    tauri::Builder::default()
        .manage(Mutex::new(initial))
        .manage(Mutex::new(world::WorldState::default()))
        .invoke_handler(tauri::generate_handler![
            core_version,
            table_view,
            reload_tile_set,
            generate_maze,
            explore_tile,
            trace_cell,
            world::world_start,
            world::world_walk,
            world::world_replay_start,
            world::world_replay_step
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Maze Maker tool");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_version_comes_from_maze_core() {
        assert_eq!(core_version(), maze_core::version());
    }
}
