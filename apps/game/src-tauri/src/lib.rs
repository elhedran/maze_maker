//! Maze Maker, the game's Tauri app: commands and IPC types. The game's
//! rules are in `game.rs` and the world in `maze-core` (DR-0012); the
//! commands here are thin.
//!
//! Commands:
//! - `start_game(seed: u32) -> Result<GameView, GameError>`: a new world,
//!   the player on the origin.
//! - `step(direction: Direction) -> Result<GameView, GameError>`: one move;
//!   `moved` is false when it was blocked.
//! - `initial_seed() -> Result<u32, GameError>`: the seed to start with,
//!   from the `MAZE_SEED` environment variable, or 1.

pub mod game;
pub mod ipc;
pub mod state;

use ipc::{Direction, GameError, GameView};
use state::GameState;
use std::sync::Mutex;

/// A poisoned lock means a panic mid-command. A step stores whole patches
/// and then moves the player, so the game is still consistent and safe to
/// keep using.
fn lock(state: &Mutex<GameState>) -> std::sync::MutexGuard<'_, GameState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A new world grows a few patches before the first view, a few
/// milliseconds in release, so it runs in the command directly.
#[tauri::command]
fn start_game(state: tauri::State<'_, Mutex<GameState>>, seed: u32) -> Result<GameView, GameError> {
    lock(&state).start(&state::default_tile_set_path(), seed)
}

/// Read at each call rather than once at startup, so there is no global.
#[tauri::command]
fn initial_seed() -> Result<u32, GameError> {
    let value = std::env::var(state::SEED_VAR).ok();
    state::seed_from(value.as_deref())
}

#[tauri::command]
fn step(
    state: tauri::State<'_, Mutex<GameState>>,
    direction: Direction,
) -> Result<GameView, GameError> {
    lock(&state).step(direction)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(GameState::default()))
        .invoke_handler(tauri::generate_handler![start_game, step, initial_seed])
        .run(tauri::generate_context!())
        .expect("error while running Maze Maker");
}
