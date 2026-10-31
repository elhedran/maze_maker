//! Builds the `generate_maze` response (DR-0028): generation and measures
//! run in `maze-core`; this maps them into IPC types. Pure, so tested
//! directly.

use crate::ipc::{CellPos, GenerateRequest, GeneratedMaze, Measure, ToolError, ToolErrorKind};
use crate::view::Loaded;

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
        let tile_set = TileSet::from_toml(CORE).expect("loads");
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
}
