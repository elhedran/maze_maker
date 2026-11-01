//! Builds the `generate_maze` response (DR-0028) and the `trace_cell` one
//! (DR-0034): generation, measures and the trace run in `maze-core`; this
//! maps them into IPC types. Pure, so tested directly.

use crate::ipc::{
    self, CellPos, GenerateRequest, GeneratedMaze, Measure, NeighbourState, ToolError,
    ToolErrorKind, TraceCellRequest,
};
use crate::view::{self, Loaded};
use maze_core::{NeighbourAtDraw, Table};

/// What `trace_cell` says when the maze came from an earlier revision.
/// Damson's wording for the band (DR-0033 point 20), so it can be shown as
/// it is.
pub const STALE_MESSAGE: &str = "The tile set was reloaded. Pick the cell again in the Maze tab.";

/// Why the cell holds its tile (DR-0034 rules 4 to 7). Checked before any
/// replay, in this order: the revision (`stale`), then the size and the
/// cell (`bad_request`). A failed maze is not an error.
pub fn cell_trace(
    loaded: &Loaded,
    request: &TraceCellRequest,
) -> Result<ipc::CellTrace, ToolError> {
    if request.revision != loaded.revision {
        return Err(ToolError {
            kind: ToolErrorKind::Stale,
            messages: vec![STALE_MESSAGE.into()],
        });
    }
    let trace = maze_core::trace_cell(
        &loaded.tile_set,
        &loaded.table,
        request.seed,
        request.width as usize,
        request.height as usize,
        (request.cell.x as usize, request.cell.y as usize),
    )
    .map_err(|e| ToolError {
        kind: ToolErrorKind::BadRequest,
        messages: vec![e.to_string()],
    })?;
    // Steps are at most 64 × 64 and indices at most the variant count, so
    // every `as u32` here and below is lossless.
    Ok(ipc::CellTrace {
        revision: loaded.revision,
        cell: request.cell,
        restarts: trace.restarts,
        failed: trace.failed,
        steps: trace.steps as u32,
        collapse: trace.collapse.map(|c| collapse_trace(&loaded.table, &c)),
    })
}

/// Maps the draw. Every number passes through as `maze-core` worked it out:
/// the solver only draws when its total is finite and above 0, so none of
/// them is NaN or infinite.
fn collapse_trace(table: &Table, c: &maze_core::CollapseTrace) -> ipc::CollapseTrace {
    let side = |d: maze_core::Direction| {
        let (state, tile) = match c.neighbours[d.index()] {
            NeighbourAtDraw::Counted(n) => (NeighbourState::Counted, Some(n as u32)),
            NeighbourAtDraw::Later(n) => (NeighbourState::Later, n.map(|n| n as u32)),
            NeighbourAtDraw::Edge => (NeighbourState::Edge, None),
        };
        ipc::NeighbourAtDraw {
            direction: view::direction(d),
            state,
            tile,
        }
    };
    let candidate = |k: &maze_core::CandidateScore| ipc::CandidateScore {
        tile: k.variant as u32,
        tile_weight: k.tile_weight,
        factors: k
            .factors
            .iter()
            .map(|f| ipc::PairFactor {
                direction: view::direction(f.direction),
                neighbour: f.neighbour as u32,
                base: f.base,
                exceptions: f.exception.map(|i| i as u32).into_iter().collect(),
                multiplier: f.multiplier,
                noise: f.noise,
                factor: f.factor,
                entry: view::entry_view(table, f.direction, k.variant, f.neighbour),
            })
            .collect(),
        score: k.score,
        share: k.share,
    };
    ipc::CollapseTrace {
        step: c.step as u32,
        neighbours: maze_core::Direction::ALL.map(side),
        candidates: c.candidates.iter().map(candidate).collect(),
        ruled_out: c.ruled_out as u32,
        total: c.total,
        chosen: c.chosen as u32,
    }
}

pub fn generated_maze(
    loaded: &Loaded,
    request: &GenerateRequest,
) -> Result<GeneratedMaze, ToolError> {
    let set = &loaded.tile_set;
    let generation = maze_core::generate(
        set,
        &loaded.table,
        request.seed,
        request.width as usize,
        request.height as usize,
    )
    .map_err(|e| ToolError {
        kind: ToolErrorKind::BadRequest,
        messages: vec![e.to_string()],
    })?;
    let counts =
        maze_core::count_areas(set, &generation.cells, generation.width, generation.height);
    // Sizes are at most 64, and indices at most the variant count, so every
    // `as u32` below is lossless.
    Ok(GeneratedMaze {
        revision: loaded.revision,
        seed: request.seed,
        width: request.width,
        height: request.height,
        cells: generation
            .cells
            .iter()
            .map(|c| c.map(|v| v as u32))
            .collect(),
        centre: CellPos {
            x: counts.centre.0 as u32,
            y: counts.centre.1 as u32,
        },
        restarts: generation.restarts,
        failed: generation.failed,
        measures: maze_core::measures(&counts)
            .into_iter()
            .map(|m| Measure {
                name: m.name.to_string(),
                value: m.value,
                explanation: m.explanation.to_string(),
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use maze_core::{Table, TileSet};

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    fn loaded() -> Loaded {
        loaded_from(CORE)
    }

    fn loaded_from(text: &str) -> Loaded {
        let tile_set = TileSet::from_toml_allow_incomplete(text).expect("loads");
        let table = Table::calculate(&tile_set);
        Loaded {
            tile_set,
            table,
            revision: 3,
            tile_set_path: "core.toml".into(),
            loaded_at: "2026-11-01T12:00:00Z".into(),
        }
    }

    fn request(seed: u32, width: u32, height: u32) -> GenerateRequest {
        GenerateRequest {
            seed,
            width,
            height,
        }
    }

    #[test]
    fn a_maze_carries_its_request_and_revision() {
        let maze = generated_maze(&loaded(), &request(42, 24, 16)).unwrap();
        assert_eq!(maze.revision, 3);
        assert_eq!((maze.seed, maze.width, maze.height), (42, 24, 16));
        assert_eq!(maze.cells.len(), 24 * 16);
        assert!(!maze.failed);
        assert!(maze.cells.iter().all(Option::is_some));
        assert!(maze.centre.x < 24 && maze.centre.y < 16);
        let names: Vec<&str> = maze.measures.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, ["Reachable from centre", "Separate areas"]);
        assert_eq!(
            maze,
            generated_maze(&loaded(), &request(42, 24, 16)).unwrap()
        );
    }

    #[test]
    fn a_bad_size_is_a_bad_request() {
        for (w, h) in [(0, 16), (65, 16), (24, 0), (24, u32::MAX)] {
            let err = generated_maze(&loaded(), &request(1, w, h)).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::BadRequest);
            assert!(
                err.messages[0].contains("from 1 to 64"),
                "{:?}",
                err.messages
            );
        }
    }

    #[test]
    fn serializes_as_the_bindings_say() {
        let maze = generated_maze(&loaded(), &request(1, 1, 1)).unwrap();
        let json = serde_json::to_value(&maze).unwrap();
        for key in [
            "revision", "seed", "width", "height", "cells", "centre", "restarts", "failed",
            "measures",
        ] {
            assert!(json.get(key).is_some(), "missing {key}: {json}");
        }
        assert_eq!(json["centre"], serde_json::json!({ "x": 0, "y": 0 }));
        let err = serde_json::to_value(ToolErrorKind::BadRequest).unwrap();
        assert_eq!(err, "bad_request");
    }

    // trace_cell (DR-0034).

    fn trace_request(seed: u32, width: u32, height: u32, x: u32, y: u32) -> TraceCellRequest {
        TraceCellRequest {
            revision: 3,
            seed,
            width,
            height,
            cell: CellPos { x, y },
        }
    }

    fn trace(loaded: &Loaded, seed: u32, w: u32, h: u32, x: u32, y: u32) -> ipc::CellTrace {
        cell_trace(loaded, &trace_request(seed, w, h, x, y)).expect("a valid request")
    }

    /// Every cell of a few small mazes, and a few of the default size: the
    /// trace's draw is the maze's cell. (Every cell of 24 × 16 is one replay
    /// each, too slow unoptimised; `maze-core`'s tests cover that size.)
    #[test]
    fn the_trace_matches_the_maze() {
        let core = loaded();
        let mut cells = Vec::new();
        for seed in [1, 42, 48213] {
            let (w, h) = (8, 6);
            let maze = generated_maze(&core, &request(seed, w, h)).unwrap();
            for y in 0..h {
                for x in 0..w {
                    cells.push((maze.clone(), x, y));
                }
            }
        }
        let maze = generated_maze(&core, &request(48213, 24, 16)).unwrap();
        for (x, y) in [(0, 0), (23, 0), (0, 15), (23, 15), (14, 2), (12, 8)] {
            cells.push((maze.clone(), x, y));
        }
        for (maze, x, y) in cells {
            let t = trace(&core, maze.seed, maze.width, maze.height, x, y);
            assert_eq!((t.revision, t.cell), (3, CellPos { x, y }));
            assert_eq!((t.restarts, t.failed), (maze.restarts, maze.failed));
            assert_eq!(t.steps, maze.width * maze.height);
            let c = t.collapse.expect("a full maze has every cell's draw");
            let i = (y * maze.width + x) as usize;
            assert_eq!(
                Some(c.chosen),
                maze.cells[i],
                "seed {} ({x}, {y})",
                maze.seed
            );
            assert!(c.step >= 1 && c.step <= t.steps);
            assert_eq!(c.candidates.len() as u32 + c.ruled_out, 29);
            let drawn: Vec<_> = c.candidates.iter().filter(|k| k.tile == c.chosen).collect();
            assert_eq!(drawn.len(), 1);
            assert!(drawn[0].share > 0.0);
            let sum: f64 = c.candidates.iter().map(|k| k.share).sum();
            assert!((sum - 1.0).abs() < 1e-9);

            let dirs = [
                ipc::Direction::North,
                ipc::Direction::East,
                ipc::Direction::South,
                ipc::Direction::West,
            ];
            for (n, d) in c.neighbours.iter().zip(dirs) {
                assert_eq!(n.direction, d);
                let at = match d {
                    ipc::Direction::North if y > 0 => Some(i - maze.width as usize),
                    ipc::Direction::South if y + 1 < maze.height => Some(i + maze.width as usize),
                    ipc::Direction::West if x > 0 => Some(i - 1),
                    ipc::Direction::East if x + 1 < maze.width => Some(i + 1),
                    _ => None,
                };
                match (n.state, at) {
                    (NeighbourState::Edge, None) => assert_eq!(n.tile, None),
                    (NeighbourState::Counted | NeighbourState::Later, Some(j)) => {
                        assert_eq!(n.tile, maze.cells[j], "the neighbour's tile");
                    }
                    (state, at) => panic!("{d:?} is {state:?}, but the neighbour is {at:?}"),
                }
            }
            let counted: Vec<(ipc::Direction, u32)> = c
                .neighbours
                .iter()
                .filter(|n| n.state == NeighbourState::Counted)
                .map(|n| (n.direction, n.tile.unwrap()))
                .collect();
            for k in &c.candidates {
                let factors: Vec<(ipc::Direction, u32)> = k
                    .factors
                    .iter()
                    .map(|f| (f.direction, f.neighbour))
                    .collect();
                assert_eq!(factors, counted, "a factor per counted neighbour");
            }
        }
    }

    /// Each factor's `entry` is `table_view`'s entry for the pair, and its
    /// parts are that entry's.
    #[test]
    fn each_factor_carries_the_table_entry() {
        let core = loaded();
        let table = crate::view::table_view(&core, &ipc::TableViewRequest::everything()).unwrap();
        let mut factors = 0;
        for (x, y) in [(14, 2), (12, 8), (5, 5), (20, 11)] {
            let c = trace(&core, 48213, 24, 16, x, y).collapse.unwrap();
            for k in &c.candidates {
                for f in &k.factors {
                    let e = table
                        .entries
                        .iter()
                        .find(|e| {
                            e.direction == f.direction && e.row == k.tile && e.column == f.neighbour
                        })
                        .unwrap();
                    assert_eq!(&f.entry, e);
                    assert_eq!(
                        (f.base, f.noise, &f.exceptions),
                        (e.base, e.noise, &e.exceptions)
                    );
                    let multiplier = e
                        .exceptions
                        .iter()
                        .map(|&i| table.exceptions[i as usize].multiplier)
                        .product::<f64>();
                    assert_eq!(f.multiplier, multiplier);
                    factors += 1;
                }
            }
        }
        assert!(factors > 0, "none of the cells had a counted neighbour");
    }

    #[test]
    fn a_stale_revision_is_refused() {
        let core = loaded();
        for revision in [2, 4, 0, u32::MAX] {
            let mut request = trace_request(1, 24, 16, 0, 0);
            request.revision = revision;
            let err = cell_trace(&core, &request).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::Stale);
            assert_eq!(err.messages, vec![STALE_MESSAGE.to_string()]);
        }
        // Stale wins over a bad size: it is checked first.
        let mut request = trace_request(1, 0, 0, 9, 9);
        request.revision = 2;
        assert_eq!(
            cell_trace(&core, &request).unwrap_err().kind,
            ToolErrorKind::Stale
        );
    }

    #[test]
    fn a_bad_size_or_a_cell_outside_is_a_bad_request() {
        let core = loaded();
        for (w, h) in [(0, 16), (65, 16), (24, 0), (24, u32::MAX)] {
            let err = cell_trace(&core, &trace_request(1, w, h, 0, 0)).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::BadRequest);
            assert!(
                err.messages[0].contains("from 1 to 64"),
                "{:?}",
                err.messages
            );
        }
        for (x, y) in [(24, 0), (0, 16), (u32::MAX, u32::MAX)] {
            let err = cell_trace(&core, &trace_request(1, 24, 16, x, y)).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::BadRequest);
            assert!(
                err.messages[0].contains("outside the 24 × 16 maze"),
                "{:?}",
                err.messages
            );
        }
    }

    /// DR-0028's fixture: every attempt contradicts before placing a cell.
    const ALWAYS_CONTRADICTS: &str = r#"
format = 1
name = "always contradicts"

[bases]
hedge_hedge = 0.0
passage_passage = 1.0
passage_door = 1.0
door_door = 1.0
floor_floor = 1.0
floor_wall_floor_wall = 1.0

[[tile]]
id = "hedge"
name = "Hedge"
kind = "hedge"
sides = ["hedge", "hedge", "hedge", "hedge"]
orientations = 1
weight = 1.0
"#;

    /// At 3 × 3, every attempt places one cell, then contradicts (see
    /// `maze-core`'s `tests/trace.rs`).
    const FAILS_AFTER_ONE: &str = r#"
format = 1
name = "fails after one"

[bases]
hedge_hedge = 1.0
passage_passage = 1.0
passage_door = 1.0
door_door = 1.0
floor_floor = 1.0
floor_wall_floor_wall = 1.0

[[tile]]
id = "twisted"
name = "Twisted"
kind = "room"
sides = ["floor_wall_n", "floor_wall_w", "door", "hedge"]
orientations = 4
weight = 1.0
"#;

    #[test]
    fn a_failed_maze_is_a_trace_not_an_error() {
        let never = loaded_from(ALWAYS_CONTRADICTS);
        let t = trace(&never, 5, 3, 3, 1, 1);
        assert!(t.failed);
        assert_eq!((t.restarts, t.steps), (20, 0));
        assert_eq!(t.collapse, None);

        let once = loaded_from(FAILS_AFTER_ONE);
        let maze = generated_maze(&once, &request(5, 3, 3)).unwrap();
        assert!(maze.failed);
        let placed: Vec<usize> = (0..9).filter(|&i| maze.cells[i].is_some()).collect();
        assert_eq!(placed.len(), 1, "premise: one cell placed");
        for i in 0..9u32 {
            let t = trace(&once, 5, 3, 3, i % 3, i / 3);
            assert!(t.failed);
            assert_eq!(t.steps, 1);
            if i as usize == placed[0] {
                let c = t.collapse.expect("the placed cell");
                assert_eq!(Some(c.chosen), maze.cells[i as usize]);
                for n in &c.neighbours {
                    assert!(matches!(
                        n.state,
                        NeighbourState::Later | NeighbourState::Edge
                    ));
                    assert_eq!(n.tile, None, "left open, or past the edge");
                }
            } else {
                assert_eq!(t.collapse, None, "cell {i} was left open");
            }
        }
    }

    /// Every `null` in `json`, as a path of keys and indices.
    fn nulls(json: &serde_json::Value, path: String, out: &mut Vec<String>) {
        match json {
            serde_json::Value::Null => out.push(path),
            serde_json::Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    nulls(v, format!("{path}[{i}]"), out);
                }
            }
            serde_json::Value::Object(fields) => {
                for (k, v) in fields {
                    nulls(v, format!("{path}.{k}"), out);
                }
            }
            _ => {}
        }
    }

    /// No NaN or infinity (serde_json would write `null`), so the only
    /// nulls are where the bindings allow them: an open cell's `collapse`
    /// and a neighbour's `tile` past the edge or left open.
    #[test]
    fn trace_serializes_as_the_bindings_say() {
        let core = loaded();
        let t = trace(&core, 48213, 24, 16, 0, 0);
        let json = serde_json::to_value(&t).unwrap();
        for key in [
            "revision", "cell", "restarts", "failed", "steps", "collapse",
        ] {
            assert!(json.get(key).is_some(), "missing {key}");
        }
        let collapse = &json["collapse"];
        for key in [
            "step",
            "neighbours",
            "candidates",
            "ruledOut",
            "total",
            "chosen",
        ] {
            assert!(collapse.get(key).is_some(), "missing collapse.{key}");
        }
        let mut found = Vec::new();
        nulls(&json, String::new(), &mut found);
        // The north-west corner: north and west are past the edge.
        assert_eq!(
            found,
            [
                ".collapse.neighbours[0].tile",
                ".collapse.neighbours[3].tile"
            ]
        );
        assert_eq!(collapse["neighbours"][0]["state"], "edge");
        assert_eq!(collapse["neighbours"][3]["direction"], "west");
        let stale = serde_json::to_value(ToolErrorKind::Stale).unwrap();
        assert_eq!(stale, "stale");

        // A cell in the middle has no nulls at all.
        let json = serde_json::to_value(trace(&core, 48213, 24, 16, 12, 8)).unwrap();
        let mut found = Vec::new();
        nulls(&json, String::new(), &mut found);
        assert!(found.is_empty(), "{found:?}");

        // A pair factor's fields, from the first cell that has one.
        let factor = (0..16)
            .map(|y| serde_json::to_value(trace(&core, 48213, 24, 16, 12, y)).unwrap())
            .find_map(|json| {
                let factor = &json["collapse"]["candidates"][0]["factors"][0];
                (!factor.is_null()).then(|| factor.clone())
            })
            .expect("some cell in column 12 had a counted neighbour");
        for key in [
            "direction",
            "neighbour",
            "base",
            "exceptions",
            "multiplier",
            "noise",
            "factor",
            "entry",
        ] {
            assert!(factor.get(key).is_some(), "missing factor.{key}: {factor}");
        }
    }
}
