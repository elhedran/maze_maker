//! The tool's loaded state: the last good table, and loading and reloading
//! it from the tile set file (DR-0024). Plain Rust, so tests drive it with
//! temporary files; `lib.rs` wraps it in a `Mutex` as Tauri managed state.

use crate::ipc::{
    CellTrace, ExploreRequest, ExploreView, GenerateRequest, GeneratedMaze, ReloadResult,
    TableView, TableViewRequest, ToolError, ToolErrorKind, TraceCellRequest,
};
use crate::maze;
use crate::view::{self, Loaded};
use maze_core::{Table, TileSet};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// `data/tile-sets/core.toml` in the repository. Phase 1 runs with
/// `tauri dev` only; bundling it as a resource waits for a release build.
pub fn default_tile_set_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("src-tauri sits three levels below the repository root")
        .join("data")
        .join("tile-sets")
        .join("core.toml")
}

#[derive(Debug, Default)]
pub struct ToolState {
    /// The last tile set that loaded. Kept when a reload fails.
    current: Option<Loaded>,
    /// Errors from the most recent failed load, for `table_view` to return
    /// when nothing has loaded yet.
    last_errors: Vec<String>,
    /// The revision the last successful load got.
    revision: u32,
}

impl ToolState {
    /// Reads and validates the file. On success replaces the table and bumps
    /// the revision; on failure keeps the last good table and returns every
    /// error as readable text.
    pub fn reload(&mut self, path: &Path) -> Result<ReloadResult, ToolError> {
        let shown_path = path.display().to_string();
        let result = std::fs::read_to_string(path)
            .map_err(|e| vec![format!("Couldn't read {shown_path}: {e}")])
            .and_then(|text| {
                TileSet::from_toml(&text)
                    .map_err(|errors| errors.iter().map(|e| e.to_string()).collect())
            });
        match result {
            Ok(tile_set) => {
                let table = Table::calculate(&tile_set);
                self.revision = self.revision.wrapping_add(1);
                let loaded = Loaded {
                    tile_set,
                    table,
                    revision: self.revision,
                    tile_set_path: shown_path,
                    loaded_at: iso8601_utc(SystemTime::now()),
                };
                let out = ReloadResult {
                    revision: loaded.revision,
                    tile_set_path: loaded.tile_set_path.clone(),
                    loaded_at: loaded.loaded_at.clone(),
                };
                self.current = Some(loaded);
                self.last_errors.clear();
                Ok(out)
            }
            Err(messages) => {
                self.last_errors = messages.clone();
                Err(ToolError {
                    kind: ToolErrorKind::LoadFailed,
                    messages,
                })
            }
        }
    }

    /// The view of the last good table, or the load errors if nothing has
    /// loaded yet.
    pub fn table_view(&self, request: &TableViewRequest) -> Result<TableView, ToolError> {
        view::table_view(self.loaded()?, request)
    }

    /// A maze from the last good table (DR-0028), or the load errors if
    /// nothing has loaded yet.
    pub fn generate(&self, request: &GenerateRequest) -> Result<GeneratedMaze, ToolError> {
        maze::generated_maze(self.loaded()?, request)
    }

    /// The explore view of one tile and orientation (DR-0034), or the load
    /// errors if nothing has loaded yet.
    pub fn explore(&self, request: &ExploreRequest) -> Result<ExploreView, ToolError> {
        view::explore(self.loaded()?, request)
    }

    /// Why a maze cell holds its tile (DR-0034): the load errors if nothing
    /// has loaded yet, then `stale` if the maze is from another revision.
    pub fn trace(&self, request: &TraceCellRequest) -> Result<CellTrace, ToolError> {
        maze::cell_trace(self.loaded()?, request)
    }

    pub fn loaded(&self) -> Result<&Loaded, ToolError> {
        self.current.as_ref().ok_or_else(|| ToolError {
            kind: ToolErrorKind::NotLoaded,
            messages: if self.last_errors.is_empty() {
                vec!["No tile set has been loaded.".into()]
            } else {
                self.last_errors.clone()
            },
        })
    }
}

/// `YYYY-MM-DDTHH:MM:SSZ`, without a date crate: days to a civil date by
/// Howard Hinnant's algorithm (http://howardhinnant.github.io/date_algorithms.html).
pub fn iso8601_utc(time: SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{h:02}:{m:02}:{s:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    /// A file in a fresh temp folder, removed on drop.
    struct TempFile(PathBuf);
    impl TempFile {
        fn new(name: &str, text: &str) -> TempFile {
            let dir =
                std::env::temp_dir().join(format!("maze-tool-test-{}-{name}", std::process::id()));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("tiles.toml");
            std::fs::write(&path, text).unwrap();
            TempFile(path)
        }
        fn write(&self, text: &str) {
            std::fs::write(&self.0, text).unwrap();
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
        }
    }

    #[test]
    fn default_path_finds_core_toml() {
        assert!(
            default_tile_set_path().is_file(),
            "{}",
            default_tile_set_path().display()
        );
    }

    #[test]
    fn startup_failure_is_reported_by_the_view() {
        let file = TempFile::new("startup", "format = 1\n");
        let mut state = ToolState::default();
        let err = state.reload(&file.0).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::LoadFailed);
        let err = state
            .table_view(&TableViewRequest::everything())
            .unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::NotLoaded);
        assert!(
            err.messages[0].contains("doesn't parse"),
            "{:?}",
            err.messages
        );
        let request = GenerateRequest {
            seed: 1,
            width: 24,
            height: 16,
        };
        let err = state.generate(&request).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::NotLoaded);
        assert!(err.messages[0].contains("doesn't parse"));
        let err = state
            .explore(&ExploreRequest {
                tile_id: "hedge".into(),
                orientation: 0,
            })
            .unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::NotLoaded);
        assert!(err.messages[0].contains("doesn't parse"));
        // Not loaded comes before stale: there is no revision to compare.
        let err = state.trace(&trace_request(0)).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::NotLoaded);
        assert!(err.messages[0].contains("doesn't parse"));
    }

    fn trace_request(revision: u32) -> TraceCellRequest {
        TraceCellRequest {
            revision,
            seed: 9,
            width: 4,
            height: 3,
            cell: crate::ipc::CellPos { x: 3, y: 2 },
        }
    }

    /// The race DR-0034 names: a maze generated, then a Reload, then the
    /// trace asked for with the maze's revision.
    #[test]
    fn a_trace_after_a_reload_is_stale() {
        let file = TempFile::new("trace", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        let maze = state
            .generate(&GenerateRequest {
                seed: 9,
                width: 4,
                height: 3,
            })
            .unwrap();
        let trace = state.trace(&trace_request(maze.revision)).unwrap();
        assert_eq!(trace.collapse.unwrap().chosen, maze.cells[11].unwrap());

        state.reload(&file.0).unwrap();
        let err = state.trace(&trace_request(maze.revision)).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::Stale);
        assert_eq!(err.messages, vec![maze::STALE_MESSAGE.to_string()]);
        assert_eq!(state.trace(&trace_request(2)).unwrap().revision, 2);
    }

    #[test]
    fn explore_uses_the_current_revision() {
        let file = TempFile::new("explore", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        state.reload(&file.0).unwrap();
        let view = state
            .explore(&ExploreRequest {
                tile_id: "open_room".into(),
                orientation: 0,
            })
            .unwrap();
        assert_eq!(view.revision, 2);
        assert_eq!(view.arms.len(), 4);
    }

    #[test]
    fn generate_uses_the_current_revision() {
        let file = TempFile::new("generate", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        state.reload(&file.0).unwrap();
        let request = GenerateRequest {
            seed: 9,
            width: 4,
            height: 3,
        };
        let maze = state.generate(&request).unwrap();
        assert_eq!(maze.revision, 2);
        assert_eq!(maze.cells.len(), 12);
    }

    #[test]
    fn missing_file_names_the_path() {
        let mut state = ToolState::default();
        let err = state.reload(Path::new("no/such/tiles.toml")).unwrap_err();
        assert!(err.messages[0].contains("tiles.toml"), "{:?}", err.messages);
    }

    #[test]
    fn reload_keeps_last_good_table_on_error() {
        let file = TempFile::new("reload", CORE);
        let mut state = ToolState::default();
        assert_eq!(state.reload(&file.0).unwrap().revision, 1);
        let before = state.table_view(&TableViewRequest::everything()).unwrap();

        file.write(&CORE.replace("amount = 0.15", "amount = 1.5"));
        let err = state.reload(&file.0).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::LoadFailed);
        assert_eq!(err.messages.len(), 1, "{:?}", err.messages);
        assert_eq!(
            state.table_view(&TableViewRequest::everything()).unwrap(),
            before
        );

        // A good edit is picked up with no rebuild, and bumps the revision.
        file.write(&CORE.replace("amount = 0.15", "amount = 0.0"));
        assert_eq!(state.reload(&file.0).unwrap().revision, 2);
        let after = state.table_view(&TableViewRequest::everything()).unwrap();
        assert_eq!(after.revision, 2);
        assert!(after.entries.iter().all(|e| e.noise == 1.0));
    }

    #[test]
    fn iso8601_known_dates() {
        assert_eq!(iso8601_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        let t = UNIX_EPOCH + Duration::from_secs(1_793_480_096);
        assert_eq!(iso8601_utc(t), "2026-10-31T20:54:56Z");
        let leap = UNIX_EPOCH + Duration::from_secs(951_782_400);
        assert_eq!(iso8601_utc(leap), "2000-02-29T00:00:00Z");
    }
}
