//! The game's state between commands: the tile set loading and at most one
//! game. Plain Rust, so tests drive it with temporary files; `lib.rs` wraps
//! it in a `Mutex` as Tauri managed state.

use crate::game::Game;
use crate::ipc::{Direction, GameError, GameView};
use maze_core::{Table, TileSet};
use std::path::{Path, PathBuf};

/// `data/tile-sets/core.toml` in the repository, as the tool loads it. The
/// game runs with `tauri dev` only for now; bundling the file as a
/// resource waits for a release build.
pub fn default_tile_set_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("src-tauri sits three levels below the repository root")
        .join("data")
        .join("tile-sets")
        .join("core.toml")
}

#[derive(Default)]
pub struct GameState {
    game: Option<Game>,
}

impl GameState {
    /// Starts a new game for `seed`, replacing any game in progress. Reads
    /// the tile set afresh, so an edit to the file shows in the next game.
    /// On a load failure the game in progress is kept.
    pub fn start(&mut self, path: &Path, seed: u32) -> Result<GameView, GameError> {
        let shown = path.display();
        let text = std::fs::read_to_string(path).map_err(|e| GameError {
            message: format!("Couldn't read {shown}: {e}"),
        })?;
        let load_failed = |errors: Vec<maze_core::LoadError>| GameError {
            message: errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let set = TileSet::from_toml(&text).map_err(load_failed)?;
        let table = Table::calculate(&set);
        let game = Game::new(&set, &table, seed).map_err(load_failed)?;
        let view = game.view(true)?;
        self.game = Some(game);
        Ok(view)
    }

    /// One step of the game in progress.
    pub fn step(&mut self, direction: Direction) -> Result<GameView, GameError> {
        let game = self.game.as_mut().ok_or_else(|| GameError {
            message: NOT_STARTED.into(),
        })?;
        let moved = game.step(direction.into());
        game.view(moved)
    }
}

pub const NOT_STARTED: &str = "No game has been started.";

/// The environment variable that sets the first game's seed, for the demo's
/// restart-with-the-same-seed step (DR-0045 has no seed field in G.1).
pub const SEED_VAR: &str = "MAZE_SEED";
/// The seed when `MAZE_SEED` isn't set.
pub const DEFAULT_SEED: u32 = 1;

/// The seed from `MAZE_SEED`'s value, if set. A value that isn't a u32 is an
/// error rather than a quiet fallback, so a typo can't start the wrong
/// world while the player thinks it's theirs.
pub fn seed_from(value: Option<&str>) -> Result<u32, GameError> {
    let Some(text) = value else {
        return Ok(DEFAULT_SEED);
    };
    text.trim().parse().map_err(|_| GameError {
        message: format!("{SEED_VAR} is {text:?}; it must be a whole number from 0 to 4294967295."),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    /// A file in a fresh temp folder, removed on drop.
    struct TempFile(PathBuf);
    impl TempFile {
        fn new(name: &str, text: &str) -> TempFile {
            let dir =
                std::env::temp_dir().join(format!("maze-game-test-{}-{name}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("tiles.toml");
            std::fs::write(&path, text).unwrap();
            TempFile(path)
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
        }
    }

    #[test]
    fn default_path_finds_core_toml() {
        assert!(default_tile_set_path().is_file());
    }

    #[test]
    fn a_step_before_a_start_is_an_error() {
        let err = GameState::default().step(Direction::North).unwrap_err();
        assert_eq!(err.message, NOT_STARTED);
    }

    #[test]
    fn start_then_step() {
        let mut state = GameState::default();
        let view = state.start(&default_tile_set_path(), 1).unwrap();
        assert!(view.moved);
        assert_eq!((view.player.x, view.player.y), (0, 0));
        let mut moved = false;
        for d in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let after = state.step(d).unwrap();
            if after.moved {
                assert_ne!((after.player.x, after.player.y), (0, 0));
                moved = true;
                break;
            }
            assert_eq!(after.cells, view.cells);
        }
        assert!(moved, "the start cell has a way on (DR-0036 rule 2)");
    }

    #[test]
    fn a_failed_load_names_the_problem_and_keeps_the_game() {
        let mut state = GameState::default();
        let started = state.start(&default_tile_set_path(), 2).unwrap();

        let err = state.start(Path::new("no/such/tiles.toml"), 3).unwrap_err();
        assert!(err.message.contains("tiles.toml"), "{}", err.message);

        let bad = TempFile::new("bad", &CORE.replace("amount = 0.15", "amount = 1.5"));
        let err = state.start(&bad.0, 3).unwrap_err();
        assert!(err.message.contains("amount"), "{}", err.message);

        // The game in progress is still seed 2's.
        let mut fresh = GameState::default();
        fresh.start(&default_tile_set_path(), 2).unwrap();
        for d in [Direction::North, Direction::East, Direction::South] {
            assert_eq!(state.step(d).unwrap(), fresh.step(d).unwrap());
        }
        let _ = started;
    }

    #[test]
    fn the_seed_comes_from_maze_seed_or_defaults_to_1() {
        assert_eq!(seed_from(None).unwrap(), 1);
        assert_eq!(seed_from(Some("42")).unwrap(), 42);
        assert_eq!(seed_from(Some(" 7 ")).unwrap(), 7);
        assert_eq!(seed_from(Some("4294967295")).unwrap(), u32::MAX);
        for bad in ["", "-1", "4294967296", "abc", "1.5"] {
            let err = seed_from(Some(bad)).unwrap_err();
            assert!(err.message.contains("MAZE_SEED"), "{}", err.message);
        }
    }

    #[test]
    fn start_reads_the_file_it_is_given() {
        let file = TempFile::new("good", CORE);
        let mut state = GameState::default();
        let a = state.start(&file.0, 6).unwrap();
        let b = state.start(&default_tile_set_path(), 6).unwrap();
        assert_eq!(a, b);
    }
}
