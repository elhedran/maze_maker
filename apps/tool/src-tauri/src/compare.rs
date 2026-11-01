//! The same seed before and after the last change (DR-0038 rules 7 to 9).
//! Both mazes and their measures come from `maze::generated_maze`, the code
//! `generate_maze` runs, so compare adds no generation or measure code of
//! its own.

use crate::ipc::{
    BeforeMaze, CompareRequest, CompareResult, EditRecord, GenerateRequest, GeneratedMaze,
    ToolError,
};
use crate::maze;
use crate::view::{self, Loaded};

/// Generates `request`'s seed on `previous` (if any) and on `current`, and
/// lists the cells that changed. `edits` are the tool's edits between the
/// two. Size errors are `generate_maze`'s.
pub fn compare(
    previous: Option<&Loaded>,
    current: &Loaded,
    edits: &[EditRecord],
    request: &CompareRequest,
) -> Result<CompareResult, ToolError> {
    let generate = GenerateRequest {
        seed: request.seed,
        width: request.width,
        height: request.height,
    };
    let after = maze::generated_maze(current, &generate)?;
    let Some(previous) = previous else {
        return Ok(CompareResult {
            before: None,
            after,
            changed: Vec::new(),
            edits: Vec::new(),
        });
    };
    let before = maze::generated_maze(previous, &generate)?;
    let changed = changed_cells(previous, &before, current, &after);
    Ok(CompareResult {
        before: Some(BeforeMaze {
            maze: before,
            tiles: view::tile_views(&previous.tile_set),
            loaded_at: previous.loaded_at.clone(),
        }),
        after,
        changed,
        edits: edits.to_vec(),
    })
}

/// Row-major indices, ascending, of the cells whose (tile id, orientation)
/// differ, or that are open on one side only. By identity, because each
/// maze's indices are into its own load's tile list (rule 8).
fn changed_cells(
    a_loaded: &Loaded,
    a: &GeneratedMaze,
    b_loaded: &Loaded,
    b: &GeneratedMaze,
) -> Vec<u32> {
    let identity = |loaded: &Loaded, cell: Option<u32>| {
        cell.map(|v| {
            let set = &loaded.tile_set;
            let variant = &set.variants[v as usize];
            (set.tile_of(variant).id.clone(), variant.orientation)
        })
    };
    a.cells
        .iter()
        .zip(&b.cells)
        .enumerate()
        .filter(|&(_, (&x, &y))| identity(a_loaded, x) != identity(b_loaded, y))
        .map(|(i, _)| i as u32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use maze_core::{Table, TileSet};

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    fn loaded(text: &str, revision: u32) -> Loaded {
        let tile_set = TileSet::from_toml(text).expect("loads");
        let table = Table::calculate(&tile_set);
        Loaded {
            tile_set,
            table,
            revision,
            tile_set_path: "core.toml".into(),
            loaded_at: format!("2026-11-01T10:00:0{revision}Z"),
        }
    }

    fn request(seed: u32) -> CompareRequest {
        CompareRequest {
            seed,
            width: 24,
            height: 16,
        }
    }

    fn identity(loaded: &Loaded, maze: &GeneratedMaze, i: usize) -> Option<(String, u32)> {
        maze.cells[i].map(|v| {
            let t = &view::tile_views(&loaded.tile_set)[v as usize];
            (t.id.clone(), t.orientation)
        })
    }

    #[test]
    fn with_no_previous_load_there_is_no_before() {
        let current = loaded(CORE, 1);
        let result = compare(None, &current, &[], &request(7)).unwrap();
        assert!(result.before.is_none());
        assert!(result.changed.is_empty() && result.edits.is_empty());
        let maze = maze::generated_maze(
            &current,
            &GenerateRequest {
                seed: 7,
                width: 24,
                height: 16,
            },
        )
        .unwrap();
        assert_eq!(result.after, maze);
    }

    /// `changed` is exactly the cells whose identity differs, checked
    /// against two `generate_maze` calls; measures are `generate_maze`'s.
    #[test]
    fn changed_cells_and_measures_match_two_generations() {
        let before = loaded(CORE, 1);
        let text = CORE
            .replace("door_door = 0.05", "door_door = 1.0")
            .replace("multiplier = 0.5", "multiplier = 1.0");
        let after = loaded(&text, 2);
        let mut any_changed = false;
        for seed in [0, 1, 42, 2026] {
            let result = compare(Some(&before), &after, &[], &request(seed)).unwrap();
            let generate = GenerateRequest {
                seed,
                width: 24,
                height: 16,
            };
            let a = maze::generated_maze(&before, &generate).unwrap();
            let b = maze::generated_maze(&after, &generate).unwrap();
            let shown = result.before.as_ref().unwrap();
            assert_eq!(shown.maze, a);
            assert_eq!(result.after, b);
            assert_eq!(shown.maze.measures, a.measures);
            assert_eq!(shown.maze.revision, 1);
            assert_eq!(shown.loaded_at, before.loaded_at);
            assert_eq!(shown.tiles, view::tile_views(&before.tile_set));
            let expected: Vec<u32> = (0..a.cells.len())
                .filter(|&i| identity(&before, &a, i) != identity(&after, &b, i))
                .map(|i| i as u32)
                .collect();
            assert_eq!(result.changed, expected, "seed {seed}");
            assert!(result.changed.windows(2).all(|w| w[0] < w[1]));
            any_changed |= !expected.is_empty();
        }
        assert!(any_changed, "the edit changed no maze; pick a bigger one");
    }

    #[test]
    fn the_same_numbers_change_nothing() {
        let (a, b) = (loaded(CORE, 1), loaded(CORE, 2));
        let result = compare(Some(&a), &b, &[], &request(5)).unwrap();
        assert!(result.changed.is_empty());
        assert_eq!(result.before.unwrap().maze.cells, result.after.cells);
    }

    /// Identity, not index: a tile moved in the file renumbers every
    /// variant, but the cells that kept their tile aren't marked.
    #[test]
    fn cells_are_compared_by_identity_not_index() {
        let before = loaded(CORE, 1);
        // Move the hedge tile to the front: same numbers, new table order.
        let start = CORE.find("[[tile]]\nid = \"hedge\"").unwrap();
        let end = start + CORE[start..].find("\n# Exceptions").unwrap();
        let hedge = &CORE[start..end];
        let first = CORE.find("[[tile]]").unwrap();
        let moved = format!(
            "{}{}\n\n{}{}",
            &CORE[..first],
            hedge.trim_end(),
            &CORE[first..start],
            &CORE[end..]
        );
        let after = loaded(&moved, 2);
        assert_ne!(
            view::tile_views(&after.tile_set)[0].id,
            view::tile_views(&before.tile_set)[0].id
        );
        // The same grid written in the new order, with one cell left open
        // and one given another tile: only those two are marked.
        let a = maze::generated_maze(
            &before,
            &GenerateRequest {
                seed: 3,
                width: 24,
                height: 16,
            },
        )
        .unwrap();
        let new_tiles = view::tile_views(&after.tile_set);
        let index_of = |id: &(String, u32)| {
            new_tiles
                .iter()
                .position(|t| (&t.id, t.orientation) == (&id.0, id.1))
                .unwrap() as u32
        };
        let mut b = a.clone();
        b.revision = 2;
        for (i, cell) in b.cells.iter_mut().enumerate() {
            *cell = identity(&before, &a, i).map(|id| index_of(&id));
        }
        assert_ne!(a.cells, b.cells, "the move renumbered nothing");
        b.cells[10] = None;
        let other = (b.cells[20].unwrap() + 1) % new_tiles.len() as u32;
        b.cells[20] = Some(other);
        assert_eq!(changed_cells(&before, &a, &after, &b), [10, 20]);
    }

    #[test]
    fn a_bad_size_is_generate_mazes_error() {
        let current = loaded(CORE, 1);
        let bad = CompareRequest {
            seed: 1,
            width: 0,
            height: 16,
        };
        let err = compare(Some(&current), &current, &[], &bad).unwrap_err();
        assert_eq!(err.kind, crate::ipc::ToolErrorKind::BadRequest);
    }
}
