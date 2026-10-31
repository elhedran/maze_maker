//! IPC types for the tool's Tauri commands (DR-0023, DR-0024).
//!
//! Rust is the source of truth. Each type derives `ts_rs::TS`, and the
//! generated TypeScript is checked in at `apps/tool/src/bindings/`. The test
//! at the bottom fails when those files are stale; run it with
//! `UPDATE_BINDINGS=1` to rewrite them.
//!
//! Conventions (DR-0023): camelCase fields, snake_case enum values, no 64-bit
//! integers (ts-rs maps them to `bigint`). These are not `maze-core` types:
//! `view.rs` maps core values into them, so core never sees ts-rs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A compass direction; the neighbour lies this way from the row tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TileKind {
    Corridor,
    Room,
    Hedge,
}

/// DR-0015's side kinds. `floor_wall_n` is floor with the wall at the N end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum SideKind {
    Hedge,
    Passage,
    Door,
    Floor,
    FloorWallN,
    FloorWallE,
    FloorWallS,
    FloorWallW,
}

/// Picks tiles. Empty fields mean any; a tile must match every non-empty
/// field. Within a field, matching any one value is enough.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", default)]
pub struct TileFilter {
    pub kinds: Vec<TileKind>,
    pub tile_ids: Vec<String>,
    pub tags: Vec<String>,
}

/// A window over the filtered rows or columns. Clamped to what exists, so
/// `{ start: 0, count: 0xFFFFFFFF }` asks for everything.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub start: u32,
    pub count: u32,
}

impl Window {
    pub const ALL: Window = Window {
        start: 0,
        count: u32::MAX,
    };
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TableViewRequest {
    pub row_filter: TileFilter,
    pub column_filter: TileFilter,
    /// Empty means all four. The response lists them N, E, S, W.
    pub directions: Vec<Direction>,
    pub rows: Window,
    pub columns: Window,
}

impl TableViewRequest {
    /// Everything, all four directions.
    pub fn everything() -> TableViewRequest {
        TableViewRequest {
            row_filter: TileFilter::default(),
            column_filter: TileFilter::default(),
            directions: Vec::new(),
            rows: Window::ALL,
            columns: Window::ALL,
        }
    }
}

/// One (tile, orientation): a row and a column of the table.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TileView {
    /// The tile type, e.g. `corridor_corner`. With `orientation`, this is
    /// the identity; the frontend maps it to a glyph (DR-0017).
    pub id: String,
    pub name: String,
    pub kind: TileKind,
    pub tags: Vec<String>,
    /// Clockwise quarter turns from the sides as written in the data.
    pub orientation: u32,
    /// N, E, S, W in this orientation.
    pub sides: [SideKind; 4],
    pub weight: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExceptionView {
    pub name: String,
    pub multiplier: f64,
    /// The rule as written in the data, e.g.
    /// `tile room_doorway (door) beside tile corridor_dead_end (passage)`.
    pub rule: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NoiseView {
    pub seed: u32,
    pub amount: f64,
}

/// The two sides that touch: the row tile's side facing `direction`, and the
/// neighbour's side facing back.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Facing {
    pub row: SideKind,
    pub column: SideKind,
}

/// One entry with its explanation (DR-0022 condition 5, DR-0024).
/// `weight = base × rowWeight × columnWeight × each exception's multiplier ×
/// noise`; `probability = weight / rowTotal`, or 0 if `rowTotal` is 0.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EntryView {
    pub direction: Direction,
    /// Index into `TableView.tiles`.
    pub row: u32,
    /// Index into `TableView.tiles`.
    pub column: u32,
    pub facing: Facing,
    /// False when the facing sides may not meet; then every number below
    /// except the tile weights and `rowTotal` is 0, and `noise` is 1.
    pub allowed: bool,
    /// The side kinds the row's side may meet (DR-0015), so a 0 can say
    /// what would have fitted. Filled for every entry.
    pub may_meet: Vec<SideKind>,
    pub base: f64,
    pub row_weight: f64,
    pub column_weight: f64,
    /// Indices into `TableView.exceptions`.
    pub exceptions: Vec<u32>,
    /// The seeded noise factor; 1 when there is no noise or the entry is 0.
    pub noise: f64,
    pub weight: f64,
    /// Sum of `weight` over every neighbour in this direction, ignoring
    /// filters, so filtering never changes a probability.
    pub row_total: f64,
    pub probability: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TableView {
    /// Bumped by every successful load.
    pub revision: u32,
    pub tile_set_path: String,
    /// ISO 8601, UTC.
    pub loaded_at: String,
    pub tile_set_name: String,
    pub noise: Option<NoiseView>,
    /// Every (tile, orientation) in table order, unfiltered.
    pub tiles: Vec<TileView>,
    /// Rows and columns after filtering, before the window.
    pub total_rows: u32,
    pub total_columns: u32,
    /// Indices into `tiles`, in display order, inside the window.
    pub rows: Vec<u32>,
    pub columns: Vec<u32>,
    pub directions: Vec<Direction>,
    pub exceptions: Vec<ExceptionView>,
    /// One per direction × row × column, in that nesting order.
    pub entries: Vec<EntryView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ReloadResult {
    pub revision: u32,
    pub tile_set_path: String,
    pub loaded_at: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ToolErrorKind {
    /// The tile set file couldn't be read or failed validation. After a
    /// reload, the last good table is still served.
    LoadFailed,
    /// No table has loaded yet; `messages` holds the startup load errors.
    NotLoaded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ToolError {
    pub kind: ToolErrorKind,
    /// Readable text, one item per message.
    pub messages: Vec<String>,
}

#[cfg(test)]
mod bindings {
    //! The staleness test for `apps/tool/src/bindings/` (DR-0023).

    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use ts_rs::Config;

    fn bindings_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("src")
            .join("bindings")
    }

    /// Every exported type, by file name. Add new IPC types here.
    fn expected() -> BTreeMap<String, String> {
        fn render<T: TS + 'static>(cfg: &Config, out: &mut BTreeMap<String, String>) {
            let file = format!("{}.ts", T::ident(cfg));
            let text = T::export_to_string(cfg).expect("ts-rs export");
            out.insert(file, text.replace("\r\n", "\n"));
        }
        let cfg = Config::new();
        let mut out = BTreeMap::new();
        render::<Direction>(&cfg, &mut out);
        render::<TileKind>(&cfg, &mut out);
        render::<SideKind>(&cfg, &mut out);
        render::<TileFilter>(&cfg, &mut out);
        render::<Window>(&cfg, &mut out);
        render::<TableViewRequest>(&cfg, &mut out);
        render::<TileView>(&cfg, &mut out);
        render::<ExceptionView>(&cfg, &mut out);
        render::<NoiseView>(&cfg, &mut out);
        render::<Facing>(&cfg, &mut out);
        render::<EntryView>(&cfg, &mut out);
        render::<TableView>(&cfg, &mut out);
        render::<ReloadResult>(&cfg, &mut out);
        render::<ToolErrorKind>(&cfg, &mut out);
        render::<ToolError>(&cfg, &mut out);
        out
    }

    /// The `.ts` files currently in the bindings folder.
    fn on_disk() -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let Ok(dir) = std::fs::read_dir(bindings_dir()) else {
            return out;
        };
        for entry in dir {
            let path = entry.expect("read bindings dir").path();
            if path.extension().is_some_and(|e| e == "ts") {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                // Git on Windows may check files out with CRLF.
                let text = std::fs::read_to_string(&path).expect("read binding");
                out.insert(name, text.replace("\r\n", "\n"));
            }
        }
        out
    }

    #[test]
    fn bindings_are_up_to_date() {
        let expected = expected();
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            let dir = bindings_dir();
            std::fs::create_dir_all(&dir).expect("create bindings dir");
            for stale in on_disk().keys().filter(|k| !expected.contains_key(*k)) {
                std::fs::remove_file(dir.join(stale)).expect("remove stale binding");
            }
            for (name, text) in &expected {
                std::fs::write(dir.join(name), text).expect("write binding");
            }
            return;
        }
        let actual = on_disk();
        let mut problems = Vec::new();
        for (name, text) in &expected {
            match actual.get(name) {
                None => problems.push(format!("missing: {name}")),
                Some(found) if found != text => problems.push(format!("stale: {name}")),
                Some(_) => {}
            }
        }
        for name in actual.keys().filter(|k| !expected.contains_key(*k)) {
            problems.push(format!("left over from a removed type: {name}"));
        }
        assert!(
            problems.is_empty(),
            "apps/tool/src/bindings is out of step with ipc.rs; \
             run `UPDATE_BINDINGS=1 cargo test -p maze-tool bindings` and commit the result:\n  {}",
            problems.join("\n  ")
        );
    }
}
