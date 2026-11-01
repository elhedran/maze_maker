//! The tool's loaded state: the last good table, and loading and reloading
//! it from the tile set file (DR-0024). Plain Rust, so tests drive it with
//! temporary files; `lib.rs` wraps it in a `Mutex` as Tauri managed state.

use crate::ipc::{
    CellTrace, CompareRequest, CompareResult, EditRecord, EditRequest, EditResult, ExploreRequest,
    ExploreView, GenerateRequest, GeneratedMaze, ReloadResult, TableView, TableViewRequest,
    ToolError, ToolErrorKind, TraceCellRequest,
};
use crate::view::{self, Loaded};
use crate::{compare, edit, maze};
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
    /// The file text `current` loaded from: a save refuses to overwrite a
    /// file that no longer matches it (DR-0038 rule 3).
    current_text: String,
    /// The load before the last one that changed the text: compare's
    /// "before" (DR-0038 rule 7).
    previous: Option<Loaded>,
    /// The last save's edits, while its load is current.
    last_edits: Vec<EditRecord>,
}

pub const STALE_EDIT_MESSAGE: &str = "The tile set was reloaded since this number was shown. Press Reload, then make the change again.";

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
                    .map(|set| (set, text))
                    .map_err(|errors| errors.iter().map(|e| e.to_string()).collect())
            });
        match result {
            Ok((tile_set, text)) => {
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
                // A load of different text makes the current one the
                // "before"; a Reload of the same text keeps the before and
                // the edits that led to it (DR-0038 rule 7).
                if self.current.is_some() && text != self.current_text {
                    self.previous = self.current.take();
                    self.last_edits.clear();
                }
                self.current = Some(loaded);
                self.current_text = text;
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

    /// Saves edited numbers into the tile set file at `path` and reloads it
    /// (DR-0038 rule 3), in this order: `not_loaded`; `stale` if the
    /// revision isn't the loaded one or the file changed on disk since;
    /// `bad_request` for an unknown target; `refused` with the loader's
    /// reasons; `load_failed` if the file can't be written. Each of those
    /// leaves the file, the record and the loaded table as they were. A
    /// record that can't be appended to is a warning, not a failure.
    pub fn edit(&mut self, path: &Path, request: &EditRequest) -> Result<EditResult, ToolError> {
        let loaded = self.loaded()?;
        if request.revision != loaded.revision {
            return Err(ToolError {
                kind: ToolErrorKind::Stale,
                messages: vec![STALE_EDIT_MESSAGE.into()],
            });
        }
        let shown_path = path.display().to_string();
        let on_disk = std::fs::read_to_string(path).map_err(|e| ToolError {
            kind: ToolErrorKind::LoadFailed,
            messages: vec![format!("Couldn't read {shown_path}: {e}")],
        })?;
        if on_disk != self.current_text {
            return Err(ToolError {
                kind: ToolErrorKind::Stale,
                messages: vec![format!(
                    "{shown_path} has changed on disk since it was loaded. Press Reload to load it, then make the change again."
                )],
            });
        }
        let (text, applied) = edit::apply(&on_disk, &loaded.tile_set, &request.edits)?;
        edit::validate(&text)?;
        edit::write_file(path, &text).map_err(|e| ToolError {
            kind: ToolErrorKind::LoadFailed,
            messages: vec![format!("Couldn't save {shown_path}: {e}")],
        })?;
        let at = iso8601_utc(SystemTime::now());
        let records: Vec<EditRecord> = applied.into_iter().map(|a| a.record(&at)).collect();
        let mut warnings = Vec::new();
        if let Err(e) = edit::append_record(path, &records) {
            warnings.push(format!(
                "The change is saved, but it couldn't be added to {}: {e}",
                edit::record_path(path).display()
            ));
        }
        let changed = text != self.current_text;
        let reload = self.reload(path)?;
        // A save that wrote the same text (a number set to what it was)
        // keeps the before and its edits, as a Reload would.
        if changed {
            self.last_edits = records.clone();
        }
        Ok(EditResult {
            reload,
            edits: records,
            warnings,
        })
    }

    /// The same seed on the previous load and the current one (DR-0038
    /// rules 7 to 9), or the load errors if nothing has loaded yet.
    pub fn compare(&self, request: &CompareRequest) -> Result<CompareResult, ToolError> {
        compare::compare(
            self.previous.as_ref(),
            self.loaded()?,
            &self.last_edits,
            request,
        )
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

    // Editing and compare (DR-0038).

    use crate::ipc::{CompareRequest, EditRequest, NumberEdit, NumberTarget};

    fn save(
        state: &mut ToolState,
        file: &TempFile,
        target: NumberTarget,
        value: f64,
    ) -> Result<EditResult, ToolError> {
        let revision = state.revision;
        state.edit(
            &file.0,
            &EditRequest {
                revision,
                edits: vec![NumberEdit { target, value }],
            },
        )
    }

    fn hedge_weight() -> NumberTarget {
        NumberTarget::TileWeight {
            tile_id: "hedge".into(),
        }
    }

    fn record_of(file: &TempFile) -> Option<String> {
        std::fs::read_to_string(edit::record_path(&file.0)).ok()
    }

    const COMPARE: CompareRequest = CompareRequest {
        seed: 42,
        width: 24,
        height: 16,
    };

    #[test]
    fn a_save_writes_records_reloads_and_compares() {
        let file = TempFile::new("save", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        assert!(state.compare(&COMPARE).unwrap().before.is_none());
        let original = state
            .generate(&GenerateRequest {
                seed: 42,
                width: 24,
                height: 16,
            })
            .unwrap();

        let result = save(&mut state, &file, hedge_weight(), 0.5).unwrap();
        assert_eq!(result.reload.revision, 2);
        assert!(result.warnings.is_empty(), "{:?}", result.warnings);
        assert_eq!(result.edits.len(), 1);
        let e = &result.edits[0];
        assert_eq!((e.number.as_str(), e.to), ("tile.hedge.weight", 0.5));
        assert_eq!(
            e.from,
            Some(
                TileSet::from_toml(CORE)
                    .unwrap()
                    .tiles
                    .iter()
                    .find(|t| t.id == "hedge")
                    .unwrap()
                    .weight
            )
        );
        // On disk, loaded, and recorded.
        let on_disk = std::fs::read_to_string(&file.0).unwrap();
        assert_ne!(on_disk, CORE);
        let view = state.table_view(&TableViewRequest::everything()).unwrap();
        assert_eq!(view.revision, 2);
        assert!(
            view.tiles
                .iter()
                .filter(|t| t.id == "hedge")
                .all(|t| t.weight == 0.5)
        );
        let record = record_of(&file).expect("the record was created");
        assert!(record.starts_with("# Numbers changed in the tool in tiles.toml, oldest first (DR-0038).\n# To set one back"), "{record}");
        assert!(
            record.contains("number = \"tile.hedge.weight\"\n"),
            "{record}"
        );
        assert!(record.contains(&format!("at = \"{}\"", e.at)));

        // Compare: before is the original load, with the save's edits.
        let compared = state.compare(&COMPARE).unwrap();
        let before = compared.before.expect("a before");
        assert_eq!(before.maze, original);
        assert_eq!(compared.edits, result.edits);
        assert_eq!(compared.after.revision, 2);
        assert!(
            !compared.changed.is_empty(),
            "halving the hedge changed nothing"
        );

        // A Reload of unchanged text keeps the before and the edits.
        state.reload(&file.0).unwrap();
        let again = state.compare(&COMPARE).unwrap();
        assert_eq!(again.before.unwrap().maze, original);
        assert_eq!(again.edits, result.edits);
        assert_eq!(again.changed, compared.changed);
    }

    #[test]
    fn setting_a_number_back_returns_the_maze() {
        let file = TempFile::new("setback", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        let request = GenerateRequest {
            seed: 42,
            width: 24,
            height: 16,
        };
        let original = state.generate(&request).unwrap();
        let first = save(&mut state, &file, hedge_weight(), 0.5).unwrap();
        assert_ne!(state.generate(&request).unwrap().cells, original.cells);
        let from = first.edits[0].from.unwrap();
        let back = save(&mut state, &file, hedge_weight(), from).unwrap();
        assert_eq!(back.edits[0].from, Some(0.5));
        let returned = state.generate(&request).unwrap();
        assert_eq!(returned.cells, original.cells);
        assert_eq!(returned.measures, original.measures);
        assert_eq!(
            std::fs::read_to_string(&file.0).unwrap(),
            CORE,
            "core.toml writes the weight as it was"
        );
        let record = record_of(&file).unwrap();
        assert_eq!(record.matches("[[edit]]").count(), 2, "{record}");
        // Compare now shows the round trip's last step.
        let compared = state.compare(&COMPARE).unwrap();
        assert_eq!(compared.edits, back.edits);
        assert!(!compared.changed.is_empty());
    }

    #[test]
    fn an_absent_opening_pressure_is_recorded_without_from() {
        let bare: String = CORE
            .lines()
            .filter(|l| !l.trim_start().starts_with("opening_pressure"))
            .map(|l| format!("{l}\n"))
            .collect();
        let file = TempFile::new("pressure", &bare);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        let result = save(&mut state, &file, NumberTarget::OpeningPressure, 0.5).unwrap();
        assert_eq!(result.edits[0].from, None);
        let record = record_of(&file).unwrap();
        let entry = &record[record.find("[[edit]]").unwrap()..];
        assert!(!entry.contains("from"), "{entry}");
        assert!(
            entry.contains("number = \"opening_pressure\"\nto = 0.5\n"),
            "{entry}"
        );
        let view = state.table_view(&TableViewRequest::everything()).unwrap();
        assert_eq!(view.opening_pressure, 0.5);
    }

    /// Refused, stale and bad requests leave the file's bytes, the record
    /// and the loaded revision as they were.
    #[test]
    fn a_refused_or_stale_edit_changes_nothing() {
        let file = TempFile::new("refused", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        let unchanged = |state: &ToolState, file: &TempFile| {
            assert_eq!(std::fs::read_to_string(&file.0).unwrap(), CORE);
            assert_eq!(record_of(file), None);
            assert_eq!(state.revision, 1);
            assert_eq!(state.loaded().unwrap().revision, 1);
        };
        let passage = NumberTarget::Base {
            pair: "passage_passage".into(),
        };
        let err = save(&mut state, &file, passage, 0.0).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::Refused);
        assert!(
            err.messages.iter().any(|m| m.contains("DR-0036 rule 7")),
            "{:?}",
            err.messages
        );
        unchanged(&state, &file);

        let err = save(&mut state, &file, NumberTarget::OpeningPressure, 0.0).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::Refused);
        assert!(
            err.messages[0].starts_with("opening_pressure is 0"),
            "{:?}",
            err.messages
        );
        unchanged(&state, &file);

        for target in [
            NumberTarget::TileWeight {
                tile_id: "nope".into(),
            },
            NumberTarget::Base {
                pair: "nope".into(),
            },
            NumberTarget::Exception { index: 99 },
        ] {
            let err = save(&mut state, &file, target, 0.5).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::BadRequest);
            unchanged(&state, &file);
        }

        // A revision other than the loaded one.
        let err = state
            .edit(
                &file.0,
                &EditRequest {
                    revision: 7,
                    edits: vec![NumberEdit {
                        target: hedge_weight(),
                        value: 0.5,
                    }],
                },
            )
            .unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::Stale);
        assert_eq!(err.messages, vec![STALE_EDIT_MESSAGE.to_string()]);
        unchanged(&state, &file);

        // A hand edit on disk, not yet reloaded: not overwritten.
        let hand = CORE.replace("door_door = 0.05", "door_door = 0.4");
        file.write(&hand);
        let err = save(&mut state, &file, hedge_weight(), 0.5).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::Stale);
        assert!(
            err.messages[0].contains("changed on disk"),
            "{:?}",
            err.messages
        );
        assert_eq!(std::fs::read_to_string(&file.0).unwrap(), hand);
        assert_eq!(record_of(&file), None);
        assert_eq!(state.revision, 1);
    }

    #[test]
    fn not_loaded_comes_first() {
        let mut state = ToolState::default();
        let file = TempFile::new("unloaded", CORE);
        let err = save(&mut state, &file, hedge_weight(), 0.5).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::NotLoaded);
        assert_eq!(
            state.compare(&COMPARE).unwrap_err().kind,
            ToolErrorKind::NotLoaded
        );
    }

    /// A hand edit and Reload: the before is the old load, with no edits.
    #[test]
    fn a_reload_of_changed_text_moves_the_before_and_clears_the_edits() {
        let file = TempFile::new("hand", CORE);
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        save(&mut state, &file, hedge_weight(), 0.5).unwrap();
        let saved = std::fs::read_to_string(&file.0).unwrap();
        file.write(&saved.replace("door_door = 0.05", "door_door = 0.4"));
        state.reload(&file.0).unwrap();
        let compared = state.compare(&COMPARE).unwrap();
        assert!(compared.edits.is_empty());
        let before = compared.before.unwrap();
        assert_eq!(before.maze.revision, 2, "the saved load is now the before");
        assert_eq!(compared.after.revision, 3);
    }

    #[test]
    fn a_record_that_cannot_be_written_is_a_warning() {
        let file = TempFile::new("warn", CORE);
        // A folder where the record would go.
        std::fs::create_dir_all(edit::record_path(&file.0)).unwrap();
        let mut state = ToolState::default();
        state.reload(&file.0).unwrap();
        let result = save(&mut state, &file, hedge_weight(), 0.5).unwrap();
        assert_eq!(result.warnings.len(), 1);
        assert!(
            result.warnings[0].contains("tiles.edits.toml"),
            "{:?}",
            result.warnings
        );
        assert_eq!(result.reload.revision, 2);
        assert_ne!(std::fs::read_to_string(&file.0).unwrap(), CORE);
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
