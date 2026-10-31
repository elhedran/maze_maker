//! The tool's loaded state: the last good table, and loading and reloading
//! it from the tile set file (DR-0024). Plain Rust, so tests drive it with
//! temporary files; `lib.rs` wraps it in a `Mutex` as Tauri managed state.

use crate::ipc::{ReloadResult, TableView, TableViewRequest, ToolError, ToolErrorKind};
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
        match &self.current {
            Some(loaded) => Ok(view::table_view(loaded, request)),
            None => Err(ToolError {
                kind: ToolErrorKind::NotLoaded,
                messages: if self.last_errors.is_empty() {
                    vec!["No tile set has been loaded.".into()]
                } else {
                    self.last_errors.clone()
                },
            }),
        }
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

        file.write(&CORE.replace("weight = 1.0", "weight = -2.0"));
        let err = state.reload(&file.0).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::LoadFailed);
        assert_eq!(err.messages.len(), 10, "one per tile: {:?}", err.messages);
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
