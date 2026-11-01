//! WFC runs that start from per-cell candidate masks (DR-0031 rule 3), the
//! building block for 2.6's chunks. `wfc::Solver` runs one attempt; the
//! masks spread through DR-0028's propagation with no other change.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH, Solver, attempt_seed, neighbour};
use maze_core::{Direction, SideKind, Table, TileSet, generate};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    let set = TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml failed: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

fn index(set: &TileSet, id: &str, orientation: u32) -> usize {
    set.variants
        .iter()
        .position(|v| set.tile_of(v).id == id && v.orientation == orientation)
        .unwrap_or_else(|| panic!("no {id} turned {orientation}"))
}

/// Every neighbouring pair is one the table gives a weight above 0.
fn assert_pairs_allowed(table: &Table, cells: &[usize], width: usize, height: usize) {
    for (i, &a) in cells.iter().enumerate() {
        for d in Direction::ALL {
            if let Some(j) = neighbour(width, height, i, d) {
                let w = table.entry(d, a, cells[j]).weight;
                assert!(w > 0.0, "cell {i} ({a}) has {} to the {d:?}", cells[j]);
            }
        }
    }
}

/// With nothing masked out, a run is exactly 2.1's first attempt.
#[test]
fn an_open_mask_gives_the_same_maze_as_generate() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let open = vec![true; w * h * table.size()];
    for seed in [0, 1, 42] {
        let maze = generate(&set, &table, seed, w, h).unwrap();
        assert_eq!(maze.restarts, 0, "pick seeds that don't restart");
        let cells = solver.run(w, h, &open, attempt_seed(seed, 0)).unwrap();
        let cells: Vec<Option<usize>> = cells.into_iter().map(Some).collect();
        assert_eq!(cells, maze.cells, "seed {seed}");
    }
}

/// Border cells cut to hedge facing out: every result respects its mask,
/// and the masks spread, so every neighbouring pair is still allowed.
#[test]
fn every_cell_stays_inside_its_mask() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h, n) = (12, 9, table.size());
    let mut mask = vec![true; w * h * n];
    for cell in 0..w * h {
        for d in Direction::ALL {
            if neighbour(w, h, cell, d).is_none() {
                for (v, variant) in set.variants.iter().enumerate() {
                    mask[cell * n + v] &= variant.side(d) == SideKind::Hedge;
                }
            }
        }
    }
    let mut finished = 0;
    for seed in 0..40 {
        let Some(cells) = solver.run(w, h, &mask, seed) else {
            continue;
        };
        finished += 1;
        assert_eq!(cells.len(), w * h);
        for (cell, &v) in cells.iter().enumerate() {
            assert!(
                mask[cell * n + v],
                "seed {seed}: cell {cell} is {v}, outside its mask"
            );
        }
        assert_pairs_allowed(&table, &cells, w, h);
    }
    assert!(finished > 0, "every run contradicted");
}

/// One cell pinned to one variant gets it, and its neighbours fit it.
#[test]
fn a_pinned_cell_keeps_its_tile_and_its_neighbours_fit() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h, n) = (7, 7, table.size());
    let crossroads = index(&set, "corridor_crossroads", 0);
    let centre = 3 * w + 3;
    let mut mask = vec![true; w * h * n];
    for v in 0..n {
        mask[centre * n + v] = v == crossroads;
    }
    for seed in 0..20 {
        if let Some(cells) = solver.run(w, h, &mask, seed) {
            assert_eq!(cells[centre], crossroads, "seed {seed}");
            assert_pairs_allowed(&table, &cells, w, h);
        }
    }
}

#[test]
fn a_cell_with_nothing_allowed_is_a_contradiction() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h, n) = (4, 3, table.size());
    let mut mask = vec![true; w * h * n];
    for v in 0..n {
        mask[5 * n + v] = false;
    }
    assert_eq!(solver.run(w, h, &mask, 1), None);
    // Two pins that can't meet: hedge beside a crossroads.
    let mut clash = vec![true; 2 * n];
    let hedge = index(&set, "hedge", 0);
    let crossroads = index(&set, "corridor_crossroads", 0);
    for v in 0..n {
        clash[v] = v == hedge;
        clash[n + v] = v == crossroads;
    }
    assert_eq!(solver.run(2, 1, &clash, 1), None);
}

/// A mask can't bring back a tile of weight 0 (DR-0028 rule 1).
#[test]
fn a_mask_never_allows_a_zero_weight_tile() {
    // The open room is the only tile of weight 0.8.
    let text = CORE.replace("weight = 0.8", "weight = 0.0");
    assert_ne!(text, CORE, "the open room weight edit didn't apply");
    let set = TileSet::from_toml(&text).unwrap();
    let table = Table::calculate(&set);
    let solver = Solver::new(&set, &table);
    let n = table.size();
    let room = index(&set, "open_room", 0);
    let only_room: Vec<bool> = (0..n).map(|v| v == room).collect();
    assert_eq!(solver.run(1, 1, &only_room, 3), None);
}

#[test]
#[should_panic(expected = "mask doesn't fit the grid")]
fn a_mask_of_the_wrong_size_panics() {
    let (set, table) = core();
    Solver::new(&set, &table).run(2, 2, &[true; 3], 0);
}
