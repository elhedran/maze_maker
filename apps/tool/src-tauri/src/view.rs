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

/// The filter's side kinds use the same matching as exceptions do (DR-0024),
/// so `floor_wall` means a wall at any end in both places.
fn side_match(f: ipc::SideFilter) -> maze_core::SideMatch {
    match f {
        ipc::SideFilter::Hedge => maze_core::SideMatch::Hedge,
        ipc::SideFilter::Passage => maze_core::SideMatch::Passage,
        ipc::SideFilter::Door => maze_core::SideMatch::Door,
        ipc::SideFilter::Floor => maze_core::SideMatch::Floor,
        ipc::SideFilter::FloorWall => maze_core::SideMatch::FloorWall,
    }
}

/// `side` is the tile's side on the shared edge (DR-0027), or `None` when
/// the request has no single direction; then a side filter can't match, but
/// `table_view` rejects that request before it gets here.
fn matches(
    filter: &ipc::TileFilter,
    tile: &ipc::TileView,
    side: Option<maze_core::SideKind>,
) -> bool {
    (filter.kinds.is_empty() || filter.kinds.contains(&tile.kind))
        && (filter.tile_ids.is_empty() || filter.tile_ids.contains(&tile.id))
        && (filter.tags.is_empty() || filter.tags.iter().any(|t| tile.tags.contains(t)))
        && (filter.sides.is_empty()
            || side.is_some_and(|s| filter.sides.iter().any(|&f| side_match(f).matches(s))))
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

/// One table entry with its explanation. The one place an `EntryView` is
/// built, so `table_view`, the explore view's arms and a trace's pair
/// factors show the same numbers from the same source (DR-0026, DR-0034
/// rule 1). Panics if `row` or `column` is out of range, as `Table::entry`
/// does; callers index from the table itself.
pub fn entry_view(
    table: &Table,
    d: maze_core::Direction,
    row: usize,
    column: usize,
) -> ipc::EntryView {
    let e = table.entry(d, row, column);
    // Indices are at most the variant count, so `as u32` is lossless.
    ipc::EntryView {
        direction: direction(d),
        row: row as u32,
        column: column as u32,
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
        row_total: finite(table.row_total(d, row)),
        probability: finite(table.probability(d, row, column)),
    }
}

/// The explore view for one (tile, orientation) (DR-0026, DR-0034 rules 1
/// to 3). Fails with `bad_request` if the loaded tile set has no such pair.
pub fn explore(
    loaded: &Loaded,
    request: &ipc::ExploreRequest,
) -> Result<ipc::ExploreView, ipc::ToolError> {
    let set = &loaded.tile_set;
    let table = &loaded.table;
    let centre = set
        .variants
        .iter()
        .position(|v| set.tile_of(v).id == request.tile_id && v.orientation == request.orientation)
        .ok_or_else(|| ipc::ToolError {
            kind: ipc::ToolErrorKind::BadRequest,
            messages: vec![format!(
                "The loaded tile set has no tile \"{}\" in orientation {}.",
                request.tile_id, request.orientation
            )],
        })?;
    let arm = |d: maze_core::Direction| {
        let (mut neighbours, zeros): (Vec<_>, Vec<_>) = (0..table.size())
            .map(|c| entry_view(table, d, centre, c))
            .partition(|e| e.weight > 0.0);
        // `partition` keeps table order; sort the visible part only.
        neighbours.sort_by(|a, b| {
            b.probability
                .total_cmp(&a.probability)
                .then(a.column.cmp(&b.column))
        });
        ipc::ArmView {
            direction: direction(d),
            centre_side: side_kind(set.variants[centre].side(d)),
            neighbours,
            zeros,
        }
    };
    Ok(ipc::ExploreView {
        revision: loaded.revision,
        centre: centre as u32,
        arms: maze_core::Direction::ALL.map(arm),
    })
}

/// Fails with `bad_request` if a side filter is set and the request doesn't
/// name exactly one direction (DR-0027): which side is "on the shared edge"
/// depends on the direction.
pub fn table_view(
    loaded: &Loaded,
    request: &ipc::TableViewRequest,
) -> Result<ipc::TableView, ipc::ToolError> {
    let set = &loaded.tile_set;
    let table = &loaded.table;

    // N, E, S, W order, each once, whatever order the request used.
    let directions: Vec<maze_core::Direction> = maze_core::Direction::ALL
        .into_iter()
        .filter(|d| request.directions.is_empty() || request.directions.contains(&direction(*d)))
        .collect();
    let side_direction = match directions.as_slice() {
        [d] => Some(*d),
        _ => None,
    };
    let filters_sides =
        !request.row_filter.sides.is_empty() || !request.column_filter.sides.is_empty();
    if filters_sides && side_direction.is_none() {
        return Err(ipc::ToolError {
            kind: ipc::ToolErrorKind::BadRequest,
            messages: vec![format!(
                "Filtering by side needs exactly one direction, but the request covers {}.",
                directions.len()
            )],
        });
    }

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

    // `edge` is which of a tile's sides is on the shared edge: the row's side
    // facing the direction, the column's side facing back.
    let filtered = |filter: &ipc::TileFilter, edge: Option<maze_core::Direction>| -> Vec<u32> {
        (0..tiles.len() as u32)
            .filter(|&i| {
                let side = edge.map(|d| set.variants[i as usize].sides[d.index()]);
                matches(filter, &tiles[i as usize], side)
            })
            .collect()
    };
    let all_rows = filtered(&request.row_filter, side_direction);
    let all_columns = filtered(
        &request.column_filter,
        side_direction.map(maze_core::Direction::opposite),
    );
    let total_rows = all_rows.len() as u32;
    let total_columns = all_columns.len() as u32;
    let rows = window(all_rows, request.rows);
    let columns = window(all_columns, request.columns);

    let mut entries = Vec::with_capacity(directions.len() * rows.len() * columns.len());
    for &d in &directions {
        for &r in &rows {
            for &c in &columns {
                entries.push(entry_view(table, d, r as usize, c as usize));
            }
        }
    }

    Ok(ipc::TableView {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{SideFilter, SideKind, TableViewRequest, TileFilter, TileKind, Window};

    fn ok_view(loaded: &Loaded, request: &TableViewRequest) -> ipc::TableView {
        table_view(loaded, request).expect("a valid request")
    }

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
        let view = ok_view(&loaded(CORE), &TableViewRequest::everything());
        assert_eq!(view.tiles.len(), 29);
        assert_eq!((view.total_rows, view.total_columns), (29, 29));
        assert_eq!(view.directions.len(), 4);
        assert_eq!(view.entries.len(), 4 * 29 * 29);
        assert_eq!(view.entries.iter().filter(|e| e.allowed).count(), 916);
    }

    #[test]
    fn filtering_never_changes_a_probability() {
        let core = loaded(CORE);
        let full = ok_view(&core, &TableViewRequest::everything());
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
        let part = ok_view(&core, &request);
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
            sides: vec![],
        };
        let view = ok_view(&core, &request);
        // Crossroads (1) and dead end (4).
        assert_eq!(view.total_rows, 5);
        request.row_filter.tile_ids = vec!["corridor_crossroads".into()];
        assert_eq!(ok_view(&core, &request).total_rows, 1);
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
        let view = ok_view(&core, &request);
        assert_eq!(view.rows, vec![27, 28]);
        assert!(view.columns.is_empty());
        assert_eq!(view.total_columns, 29);
        assert!(view.entries.is_empty());
        request.columns = Window {
            start: u32::MAX,
            count: u32::MAX,
        };
        assert!(ok_view(&core, &request).columns.is_empty());
    }

    #[test]
    fn a_zero_explains_its_clash() {
        let view = ok_view(&loaded(CORE), &TableViewRequest::everything());
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
        let view = ok_view(&loaded(&text), &TableViewRequest::everything());
        let e = find(
            &view,
            ipc::Direction::East,
            "corridor_straight",
            1,
            "corridor_corner",
            2,
        );
        assert!(e.allowed);
        // The test exception is the last one, after any in core.toml.
        let test_exception = view.exceptions.len() as u32 - 1;
        assert_eq!(e.exceptions, vec![test_exception]);
        assert_eq!(
            view.exceptions[test_exception as usize].rule,
            "tile corridor_straight (passage) beside tile corridor_corner (passage)"
        );
        let product = e.base * e.row_weight * e.column_weight * 0.5 * e.noise;
        assert!((e.weight - product).abs() < 1e-12);
        assert!((e.probability - e.weight / e.row_total).abs() < 1e-12);
    }

    #[test]
    fn same_data_gives_byte_identical_views() {
        let a = serde_json::to_string(&ok_view(&loaded(CORE), &TableViewRequest::everything()))
            .unwrap();
        let b = serde_json::to_string(&ok_view(&loaded(CORE), &TableViewRequest::everything()))
            .unwrap();
        assert_eq!(a, b);
        // serde_json writes NaN and infinity as null. Core sets noise, so
        // nothing in this view should be null.
        assert!(!a.contains("null"));
    }

    const SIDE_FILTERS: [SideFilter; 5] = [
        SideFilter::Hedge,
        SideFilter::Passage,
        SideFilter::Door,
        SideFilter::Floor,
        SideFilter::FloorWall,
    ];

    /// DR-0027's meaning of each side filter, written out independently of
    /// `side_match` so a slip there fails here.
    fn means(f: SideFilter, s: SideKind) -> bool {
        match f {
            SideFilter::Hedge => s == SideKind::Hedge,
            SideFilter::Passage => s == SideKind::Passage,
            SideFilter::Door => s == SideKind::Door,
            SideFilter::Floor => s == SideKind::Floor,
            SideFilter::FloorWall => matches!(
                s,
                SideKind::FloorWallN
                    | SideKind::FloorWallE
                    | SideKind::FloorWallS
                    | SideKind::FloorWallW
            ),
        }
    }

    fn sides_request(
        d: ipc::Direction,
        rows: &[SideFilter],
        cols: &[SideFilter],
    ) -> TableViewRequest {
        let mut request = TableViewRequest::everything();
        request.directions = vec![d];
        request.row_filter.sides = rows.to_vec();
        request.column_filter.sides = cols.to_vec();
        request
    }

    fn index(view: &ipc::TableView, id: &str, k: u32) -> u32 {
        view.tiles
            .iter()
            .position(|t| t.id == id && t.orientation == k)
            .unwrap() as u32
    }

    /// Every side kind in every direction, rows and columns, checked against
    /// the table's own `facing` (the sides that touch), so a filter that
    /// looked at the wrong side of the column would fail.
    #[test]
    fn side_filters_match_the_sides_that_touch() {
        let core = loaded(CORE);
        let full = ok_view(&core, &TableViewRequest::everything());
        for d in [
            ipc::Direction::North,
            ipc::Direction::East,
            ipc::Direction::South,
            ipc::Direction::West,
        ] {
            for f in SIDE_FILTERS {
                let in_d = full.entries.iter().filter(|e| e.direction == d);
                let mut rows: Vec<u32> = in_d
                    .clone()
                    .filter(|e| means(f, e.facing.row))
                    .map(|e| e.row)
                    .collect();
                let mut columns: Vec<u32> = in_d
                    .filter(|e| means(f, e.facing.column))
                    .map(|e| e.column)
                    .collect();
                rows.sort();
                rows.dedup();
                columns.sort();
                columns.dedup();
                assert!(!rows.is_empty() && !columns.is_empty(), "{d:?} {f:?}");

                let view = ok_view(&core, &sides_request(d, &[f], &[f]));
                assert_eq!(view.rows, rows, "rows {d:?} {f:?}");
                assert_eq!(view.columns, columns, "columns {d:?} {f:?}");
                assert_eq!(view.total_rows as usize, rows.len());
                assert_eq!(view.entries.len(), rows.len() * columns.len());
            }
        }
    }

    #[test]
    fn floor_wall_matches_a_wall_at_any_end() {
        let core = loaded(CORE);
        // A west side's wall is at its N or S end, a north side's at its E
        // or W end; wall ends turn with the tile, so both ends turn up, and
        // `floor_wall` takes every one.
        let ends_on = |d: ipc::Direction, side: usize| -> Vec<SideKind> {
            let view = ok_view(&core, &sides_request(d, &[SideFilter::FloorWall], &[]));
            let floor = ok_view(&core, &sides_request(d, &[SideFilter::Floor], &[]));
            assert!(floor.rows.iter().all(|r| !view.rows.contains(r)));
            let mut ends: Vec<SideKind> = view
                .rows
                .iter()
                .map(|&r| view.tiles[r as usize].sides[side])
                .collect();
            ends.sort_by_key(|s| *s as u8);
            ends.dedup();
            ends
        };
        assert_eq!(
            ends_on(ipc::Direction::West, 3),
            vec![SideKind::FloorWallN, SideKind::FloorWallS]
        );
        assert_eq!(
            ends_on(ipc::Direction::North, 0),
            vec![SideKind::FloorWallE, SideKind::FloorWallW]
        );
    }

    /// Yew's test (DR-0027): one tile checked both ways round. The corridor
    /// corner (turn 0) has passage N and E, hedge S and W. To the east, as
    /// a row it shows its east side (passage); as a column, its west side
    /// facing back (hedge).
    #[test]
    fn row_uses_its_side_column_uses_the_side_facing_back() {
        let core = loaded(CORE);
        let east = ipc::Direction::East;
        let full = ok_view(&core, &TableViewRequest::everything());
        let corner = index(&full, "corridor_corner", 0);

        let passage = ok_view(
            &core,
            &sides_request(east, &[SideFilter::Passage], &[SideFilter::Passage]),
        );
        assert!(passage.rows.contains(&corner));
        assert!(!passage.columns.contains(&corner));

        let hedge = ok_view(
            &core,
            &sides_request(east, &[SideFilter::Hedge], &[SideFilter::Hedge]),
        );
        assert!(!hedge.rows.contains(&corner));
        assert!(hedge.columns.contains(&corner));

        // The same pair from the entry's own facing sides.
        let e = find(&full, east, "corridor_corner", 0, "corridor_corner", 0);
        assert_eq!(
            (e.facing.row, e.facing.column),
            (SideKind::Passage, SideKind::Hedge)
        );
    }

    #[test]
    fn side_filters_or_within_and_across_fields() {
        let core = loaded(CORE);
        let mut request = sides_request(
            ipc::Direction::North,
            &[SideFilter::Door, SideFilter::Hedge],
            &[],
        );
        request.row_filter.kinds = vec![TileKind::Room];
        let view = ok_view(&core, &request);
        assert!(!view.rows.is_empty());
        for &r in &view.rows {
            let t = &view.tiles[r as usize];
            assert_eq!(t.kind, TileKind::Room);
            assert!(matches!(t.sides[0], SideKind::Door | SideKind::Hedge));
        }
    }

    #[test]
    fn side_filter_needs_exactly_one_direction() {
        let core = loaded(CORE);
        // All four (empty), two, and none in the row filter but one in the
        // column filter.
        for (directions, rows, cols) in [
            (vec![], vec![SideFilter::Door], vec![]),
            (
                vec![ipc::Direction::North, ipc::Direction::South],
                vec![],
                vec![SideFilter::Hedge],
            ),
        ] {
            let mut request = TableViewRequest::everything();
            request.directions = directions;
            request.row_filter.sides = rows;
            request.column_filter.sides = cols;
            let err = table_view(&core, &request).unwrap_err();
            assert_eq!(err.kind, ipc::ToolErrorKind::BadRequest);
            assert_eq!(err.messages.len(), 1);
        }
        // The same direction twice is one direction.
        let mut request = sides_request(ipc::Direction::East, &[SideFilter::Door], &[]);
        request.directions.push(ipc::Direction::East);
        assert!(table_view(&core, &request).is_ok());
        // No side filter: any number of directions is fine, as before.
        assert!(table_view(&core, &TableViewRequest::everything()).is_ok());
    }

    #[test]
    fn no_match_is_an_empty_view_not_an_error() {
        let core = loaded(CORE);
        // No corridor has a door.
        let mut request = sides_request(ipc::Direction::North, &[SideFilter::Door], &[]);
        request.row_filter.kinds = vec![TileKind::Corridor];
        let view = ok_view(&core, &request);
        assert_eq!(view.total_rows, 0);
        assert!(view.rows.is_empty());
        assert!(view.entries.is_empty());
        assert_eq!(view.total_columns, 29);
        assert_eq!(view.tiles.len(), 29);
    }

    #[test]
    fn request_without_sides_still_parses() {
        let json = r#"{"rowFilter":{"kinds":["room"],"tileIds":[],"tags":[]},"columnFilter":{"tags":["junction"]},
            "directions":["east"],"rows":{"start":0,"count":4294967295},"columns":{"start":0,"count":4294967295}}"#;
        let request: TableViewRequest = serde_json::from_str(json).unwrap();
        assert!(request.row_filter.sides.is_empty());
        assert!(request.column_filter.sides.is_empty());
        let json = r#"{"rowFilter":{"sides":["floor_wall","door"]},"columnFilter":{},
            "directions":["east"],"rows":{"start":0,"count":1},"columns":{"start":0,"count":1}}"#;
        let request: TableViewRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.row_filter.sides,
            vec![SideFilter::FloorWall, SideFilter::Door]
        );
        assert!(
            serde_json::from_str::<TableViewRequest>(&json.replace("floor_wall", "floor_wall_n"))
                .is_err()
        );
    }

    #[test]
    fn request_deserialises_from_camel_case_json() {
        let json = r#"{"rowFilter":{"kinds":["room"],"tileIds":[],"tags":[]},"columnFilter":{},
            "directions":["east"],"rows":{"start":0,"count":4294967295},"columns":{"start":0,"count":4294967295}}"#;
        let request: TableViewRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.directions, vec![ipc::Direction::East]);
        assert_eq!(request.rows.count, u32::MAX);
    }

    // explore_tile (DR-0034 rules 1 to 3).

    const DIRECTIONS: [ipc::Direction; 4] = [
        ipc::Direction::North,
        ipc::Direction::East,
        ipc::Direction::South,
        ipc::Direction::West,
    ];

    fn explore_request(id: &str, orientation: u32) -> ipc::ExploreRequest {
        ipc::ExploreRequest {
            tile_id: id.into(),
            orientation,
        }
    }

    /// Every pair in every direction: an arm is that row of `table_view`,
    /// split at weight 0, the visible part sorted and the 0s in table order.
    #[test]
    fn explore_agrees_with_the_table() {
        let core = loaded(CORE);
        let full = ok_view(&core, &TableViewRequest::everything());
        let mut arms = 0;
        for (i, tile) in full.tiles.iter().enumerate() {
            let i = i as u32;
            let view = explore(&core, &explore_request(&tile.id, tile.orientation)).unwrap();
            assert_eq!((view.revision, view.centre), (core.revision, i));
            for (k, (arm, d)) in view.arms.iter().zip(DIRECTIONS).enumerate() {
                assert_eq!(arm.direction, d);
                assert_eq!(arm.centre_side, tile.sides[k]);
                let row: Vec<&ipc::EntryView> = full
                    .entries
                    .iter()
                    .filter(|e| e.direction == d && e.row == i)
                    .collect();
                let mut both: Vec<&ipc::EntryView> =
                    arm.neighbours.iter().chain(&arm.zeros).collect();
                both.sort_by_key(|e| e.column);
                assert_eq!(both, row, "{} turned {} {d:?}", tile.id, tile.orientation);

                assert!(!arm.neighbours.is_empty(), "every side meets something");
                assert!(arm.neighbours.iter().all(|e| e.weight > 0.0));
                for pair in arm.neighbours.windows(2) {
                    let (a, b) = (&pair[0], &pair[1]);
                    assert!(
                        a.probability > b.probability
                            || (a.probability == b.probability && a.column < b.column),
                        "{} {d:?}: {} before {}",
                        tile.id,
                        a.column,
                        b.column
                    );
                }
                assert!(arm.zeros.iter().all(|e| e.weight == 0.0));
                assert!(arm.zeros.windows(2).all(|p| p[0].column < p[1].column));
                arms += 1;
            }
        }
        assert_eq!(arms, 29 * 4);
    }

    #[test]
    fn explore_names_the_centre_by_identity() {
        let core = loaded(CORE);
        let full = ok_view(&core, &TableViewRequest::everything());
        let view = explore(&core, &explore_request("corridor_corner", 2)).unwrap();
        let tile = &full.tiles[view.centre as usize];
        assert_eq!((tile.id.as_str(), tile.orientation), ("corridor_corner", 2));
        // Passage S and W, hedge N and E, at two quarter turns.
        let sides: Vec<SideKind> = view.arms.iter().map(|a| a.centre_side).collect();
        assert_eq!(
            sides,
            [
                SideKind::Hedge,
                SideKind::Hedge,
                SideKind::Passage,
                SideKind::Passage
            ]
        );
        let json = serde_json::to_string(&view).unwrap();
        assert!(!json.contains("null"));
        assert!(json.contains("\"centreSide\""));
    }

    #[test]
    fn explore_refuses_an_unknown_pair() {
        let core = loaded(CORE);
        // An unknown tile, a turn past the tile's distinct orientations,
        // and a case that differs.
        for (id, k) in [
            ("no_such_tile", 0),
            ("hedge", 1),
            ("corridor_straight", 2),
            ("Hedge", 0),
        ] {
            let err = explore(&core, &explore_request(id, k)).unwrap_err();
            assert_eq!(err.kind, ipc::ToolErrorKind::BadRequest);
            assert_eq!(
                err.messages,
                vec![format!(
                    "The loaded tile set has no tile \"{id}\" in orientation {k}."
                )]
            );
        }
    }

    #[test]
    fn explore_request_deserialises_from_camel_case_json() {
        let request: ipc::ExploreRequest =
            serde_json::from_str(r#"{"tileId":"room_wall","orientation":3}"#).unwrap();
        assert_eq!(request, explore_request("room_wall", 3));
    }
}
