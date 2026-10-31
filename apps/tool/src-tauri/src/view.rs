//! Builds the `table_view` response from a loaded tile set and table
//! (DR-0024). Pure: no Tauri, no I/O, so it is tested directly.

use crate::ipc;
use maze_core::{Table, TileSet};

/// A tile set that loaded, with its table and where it came from.
#[derive(Debug)]
pub struct Loaded {
    pub tile_set: TileSet,
    pub table: Table,
    pub revision: u32,
    pub tile_set_path: String,
    pub loaded_at: String,
}

pub fn direction(d: maze_core::Direction) -> ipc::Direction {
    match d {
        maze_core::Direction::North => ipc::Direction::North,
        maze_core::Direction::East => ipc::Direction::East,
        maze_core::Direction::South => ipc::Direction::South,
        maze_core::Direction::West => ipc::Direction::West,
    }
}

pub fn side_kind(s: maze_core::SideKind) -> ipc::SideKind {
    use maze_core::SideKind as S;
    match s {
        S::Hedge => ipc::SideKind::Hedge,
        S::Passage => ipc::SideKind::Passage,
        S::Door => ipc::SideKind::Door,
        S::Floor => ipc::SideKind::Floor,
        S::FloorWallN => ipc::SideKind::FloorWallN,
        S::FloorWallE => ipc::SideKind::FloorWallE,
        S::FloorWallS => ipc::SideKind::FloorWallS,
        S::FloorWallW => ipc::SideKind::FloorWallW,
    }
}

pub fn tile_kind(k: maze_core::TileKind) -> ipc::TileKind {
    match k {
        maze_core::TileKind::Corridor => ipc::TileKind::Corridor,
        maze_core::TileKind::Room => ipc::TileKind::Room,
        maze_core::TileKind::Hedge => ipc::TileKind::Hedge,
    }
}

fn matches(filter: &ipc::TileFilter, tile: &ipc::TileView) -> bool {
    (filter.kinds.is_empty() || filter.kinds.contains(&tile.kind))
        && (filter.tile_ids.is_empty() || filter.tile_ids.contains(&tile.id))
        && (filter.tags.is_empty() || filter.tags.iter().any(|t| tile.tags.contains(t)))
}

/// The slice of `items` the window covers, clamped so any start and count
/// are safe (Ash asks for everything with `count = 0xFFFFFFFF`).
fn window(items: Vec<u32>, w: ipc::Window) -> Vec<u32> {
    items
        .into_iter()
        .skip(w.start as usize)
        .take(w.count as usize)
        .collect()
}

/// Never send NaN or infinity over IPC: JSON can't carry them (they become
/// `null`). Validation should make this unreachable; this is the backstop.
fn finite(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

pub fn table_view(loaded: &Loaded, request: &ipc::TableViewRequest) -> ipc::TableView {
    let set = &loaded.tile_set;
    let table = &loaded.table;

    let tiles: Vec<ipc::TileView> = set
        .variants
        .iter()
        .map(|v| {
            let tile = set.tile_of(v);
            ipc::TileView {
                id: tile.id.clone(),
                name: tile.name.clone(),
                kind: tile_kind(tile.kind),
                tags: tile.tags.clone(),
                orientation: v.orientation,
                sides: v.sides.map(side_kind),
                weight: finite(tile.weight),
            }
        })
        .collect();

    let filtered = |filter: &ipc::TileFilter| -> Vec<u32> {
        (0..tiles.len() as u32)
            .filter(|&i| matches(filter, &tiles[i as usize]))
            .collect()
    };
    let all_rows = filtered(&request.row_filter);
    let all_columns = filtered(&request.column_filter);
    let total_rows = all_rows.len() as u32;
    let total_columns = all_columns.len() as u32;
    let rows = window(all_rows, request.rows);
    let columns = window(all_columns, request.columns);

    // N, E, S, W order, each once, whatever order the request used.
    let directions: Vec<maze_core::Direction> = maze_core::Direction::ALL
        .into_iter()
        .filter(|d| request.directions.is_empty() || request.directions.contains(&direction(*d)))
        .collect();

    let mut entries = Vec::with_capacity(directions.len() * rows.len() * columns.len());
    for &d in &directions {
        for &r in &rows {
            let row_total = finite(table.row_total(d, r as usize));
            for &c in &columns {
                let e = table.entry(d, r as usize, c as usize);
                entries.push(ipc::EntryView {
                    direction: direction(d),
                    row: r,
                    column: c,
                    facing: ipc::Facing {
                        row: side_kind(e.row_side),
                        column: side_kind(e.column_side),
                    },
                    allowed: e.allowed,
                    may_meet: e
                        .row_side
                        .may_meet_list()
                        .into_iter()
                        .map(side_kind)
                        .collect(),
                    base: finite(e.base),
                    row_weight: finite(e.row_weight),
                    column_weight: finite(e.column_weight),
                    exceptions: e.exception.map(|i| i as u32).into_iter().collect(),
                    noise: finite(e.noise),
                    weight: finite(e.weight),
                    row_total,
                    probability: finite(table.probability(d, r as usize, c as usize)),
                });
            }
        }
    }

    ipc::TableView {
        revision: loaded.revision,
        tile_set_path: loaded.tile_set_path.clone(),
        loaded_at: loaded.loaded_at.clone(),
        tile_set_name: set.name.clone(),
        noise: set.noise.map(|n| ipc::NoiseView {
            seed: n.seed,
            amount: n.amount,
        }),
        tiles,
        total_rows,
        total_columns,
        rows,
        columns,
        directions: directions.into_iter().map(direction).collect(),
        exceptions: set
            .exceptions
            .iter()
            .map(|e| ipc::ExceptionView {
                name: e.name.clone(),
                multiplier: e.multiplier,
                rule: e.rule(),
            })
            .collect(),
        entries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{TableViewRequest, TileFilter, TileKind, Window};

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    fn loaded(text: &str) -> Loaded {
        let tile_set = TileSet::from_toml(text).expect("loads");
        let table = Table::calculate(&tile_set);
        Loaded {
            tile_set,
            table,
            revision: 1,
            tile_set_path: "core.toml".into(),
            loaded_at: "2026-10-31T20:00:00Z".into(),
        }
    }

    fn find<'a>(
        view: &'a ipc::TableView,
        d: ipc::Direction,
        row: &str,
        rk: u32,
        col: &str,
        ck: u32,
    ) -> &'a ipc::EntryView {
        let idx = |id: &str, k: u32| {
            view.tiles
                .iter()
                .position(|t| t.id == id && t.orientation == k)
                .unwrap() as u32
        };
        let (r, c) = (idx(row, rk), idx(col, ck));
        view.entries
            .iter()
            .find(|e| e.direction == d && e.row == r && e.column == c)
            .unwrap()
    }

    #[test]
    fn everything_is_29_by_29_by_4() {
        let view = table_view(&loaded(CORE), &TableViewRequest::everything());
        assert_eq!(view.tiles.len(), 29);
        assert_eq!((view.total_rows, view.total_columns), (29, 29));
        assert_eq!(view.directions.len(), 4);
        assert_eq!(view.entries.len(), 4 * 29 * 29);
        assert_eq!(view.entries.iter().filter(|e| e.allowed).count(), 916);
    }

    #[test]
    fn filtering_never_changes_a_probability() {
        let core = loaded(CORE);
        let full = table_view(&core, &TableViewRequest::everything());
        let mut request = TableViewRequest::everything();
        request.row_filter = TileFilter {
            kinds: vec![TileKind::Corridor],
            ..Default::default()
        };
        request.column_filter = TileFilter {
            tags: vec!["room_edge".into()],
            ..Default::default()
        };
        request.directions = vec![ipc::Direction::West, ipc::Direction::North];
        let part = table_view(&core, &request);
        assert_eq!((part.total_rows, part.total_columns), (15, 12));
        assert_eq!(
            part.directions,
            vec![ipc::Direction::North, ipc::Direction::West]
        );
        assert_eq!(part.entries.len(), 2 * 15 * 12);
        for e in &part.entries {
            assert_eq!(full.tiles[e.row as usize].kind, TileKind::Corridor);
            let same = full
                .entries
                .iter()
                .find(|f| f.direction == e.direction && f.row == e.row && f.column == e.column)
                .unwrap();
            assert_eq!(e, same);
        }
    }

    #[test]
    fn filter_fields_combine_with_and() {
        let core = loaded(CORE);
        let mut request = TableViewRequest::everything();
        request.row_filter = TileFilter {
            kinds: vec![TileKind::Corridor],
            tile_ids: vec![],
            tags: vec!["fallback".into()],
        };
        let view = table_view(&core, &request);
        // Crossroads (1) and dead end (4).
        assert_eq!(view.total_rows, 5);
        request.row_filter.tile_ids = vec!["corridor_crossroads".into()];
        assert_eq!(table_view(&core, &request).total_rows, 1);
    }

    #[test]
    fn window_is_clamped() {
        let core = loaded(CORE);
        let mut request = TableViewRequest::everything();
        request.rows = Window {
            start: 27,
            count: 10,
        };
        request.columns = Window {
            start: 100,
            count: u32::MAX,
        };
        let view = table_view(&core, &request);
        assert_eq!(view.rows, vec![27, 28]);
        assert!(view.columns.is_empty());
        assert_eq!(view.total_columns, 29);
        assert!(view.entries.is_empty());
        request.columns = Window {
            start: u32::MAX,
            count: u32::MAX,
        };
        assert!(table_view(&core, &request).columns.is_empty());
    }

    #[test]
    fn a_zero_explains_its_clash() {
        let view = table_view(&loaded(CORE), &TableViewRequest::everything());
        // A room wall (wall N) east of an E-W straight: passage meets floor_wall_n.
        let e = find(
            &view,
            ipc::Direction::East,
            "corridor_straight",
            1,
            "room_wall",
            0,
        );
        assert!(!e.allowed);
        assert_eq!(e.facing.row, ipc::SideKind::Passage);
        assert_eq!(e.facing.column, ipc::SideKind::FloorWallN);
        assert_eq!(
            e.may_meet,
            vec![ipc::SideKind::Passage, ipc::SideKind::Door]
        );
        assert_eq!((e.weight, e.probability, e.noise), (0.0, 0.0, 1.0));
    }

    #[test]
    fn corner_east_of_straight_explains_itself() {
        let text = format!(
            "{CORE}\n[[exception]]\nname = \"Test\"\na = {{ tile = \"corridor_straight\", side = \"passage\" }}\nb = {{ tile = \"corridor_corner\", side = \"passage\" }}\nmultiplier = 0.5\n"
        );
        let view = table_view(&loaded(&text), &TableViewRequest::everything());
        let e = find(
            &view,
            ipc::Direction::East,
            "corridor_straight",
            1,
            "corridor_corner",
            2,
        );
        assert!(e.allowed);
        assert_eq!(e.exceptions, vec![0]);
        assert_eq!(
            view.exceptions[0].rule,
            "tile corridor_straight (passage) beside tile corridor_corner (passage)"
        );
        let product = e.base * e.row_weight * e.column_weight * 0.5 * e.noise;
        assert!((e.weight - product).abs() < 1e-12);
        assert!((e.probability - e.weight / e.row_total).abs() < 1e-12);
    }

    #[test]
    fn same_data_gives_byte_identical_views() {
        let a = serde_json::to_string(&table_view(&loaded(CORE), &TableViewRequest::everything()))
            .unwrap();
        let b = serde_json::to_string(&table_view(&loaded(CORE), &TableViewRequest::everything()))
            .unwrap();
        assert_eq!(a, b);
        // serde_json writes NaN and infinity as null. Core sets noise, so
        // nothing in this view should be null.
        assert!(!a.contains("null"));
    }

    #[test]
    fn request_deserialises_from_camel_case_json() {
        let json = r#"{"rowFilter":{"kinds":["room"],"tileIds":[],"tags":[]},"columnFilter":{},
            "directions":["east"],"rows":{"start":0,"count":4294967295},"columns":{"start":0,"count":4294967295}}"#;
        let request: TableViewRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.directions, vec![ipc::Direction::East]);
        assert_eq!(request.rows.count, u32::MAX);
    }
}
