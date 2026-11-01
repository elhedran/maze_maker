//! WFC runs from a `Start` (DR-0036 rule 3): per-cell candidate masks,
//! settled cells, and settled cells past the grid's edge. `wfc::Solver`
//! runs one attempt; all of it spreads through DR-0028's propagation.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH, Solver, Start, attempt_seed, neighbour};
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
fn an_open_start_gives_the_same_maze_as_generate() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let open = Start::open(w * h, table.size());
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
    let mut start = Start::open(w * h, n);
    for cell in 0..w * h {
        for d in Direction::ALL {
            if neighbour(w, h, cell, d).is_none() {
                for (v, variant) in set.variants.iter().enumerate() {
                    start.mask[cell * n + v] &= variant.side(d) == SideKind::Hedge;
                }
            }
        }
    }
    let mut finished = 0;
    for seed in 0..40 {
        let Some(cells) = solver.run(w, h, &start, seed) else {
            continue;
        };
        finished += 1;
        assert_eq!(cells.len(), w * h);
        for (cell, &v) in cells.iter().enumerate() {
            assert!(
                start.mask[cell * n + v],
                "seed {seed}: cell {cell} is {v}, outside its mask"
            );
        }
        assert_pairs_allowed(&table, &cells, w, h);
    }
    assert!(finished > 0, "every run contradicted");
}

/// A settled cell keeps its tile and its neighbours fit it.
#[test]
fn a_settled_cell_keeps_its_tile_and_its_neighbours_fit() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h) = (7, 7);
    let crossroads = index(&set, "corridor_crossroads", 0);
    let centre = 3 * w + 3;
    let mut start = Start::open(w * h, table.size());
    start.settled[centre] = Some(crossroads);
    for seed in 0..20 {
        if let Some(cells) = solver.run(w, h, &start, seed) {
            assert_eq!(cells[centre], crossroads, "seed {seed}");
            assert_pairs_allowed(&table, &cells, w, h);
        }
    }
}

/// A settled tile past the edge counts in support like a neighbour.
#[test]
fn a_settled_tile_outside_is_supported() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h) = (5, 4);
    let crossroads = index(&set, "corridor_crossroads", 0);
    let hedge = index(&set, "hedge", 0);
    let mut start = Start::open(w * h, table.size());
    // A crossroads west of every west-edge cell, hedge north of the top row.
    for y in 0..h {
        start.outside[(y * w) * 4 + Direction::West.index()] = Some(crossroads);
    }
    for x in 0..w {
        start.outside[x * 4 + Direction::North.index()] = Some(hedge);
    }
    let mut finished = 0;
    for seed in 0..20 {
        let Some(cells) = solver.run(w, h, &start, seed) else {
            continue;
        };
        finished += 1;
        for y in 0..h {
            let v = cells[y * w];
            assert!(table.entry(Direction::West, v, crossroads).weight > 0.0);
        }
        for (x, &v) in cells.iter().enumerate().take(w) {
            assert!(
                table.entry(Direction::North, v, hedge).weight > 0.0,
                "cell {x}"
            );
        }
        assert_pairs_allowed(&table, &cells, w, h);
    }
    assert!(finished > 0, "every run contradicted");
}

#[test]
fn a_cell_with_nothing_allowed_is_a_contradiction() {
    let (set, table) = core();
    let solver = Solver::new(&set, &table);
    let (w, h, n) = (4, 3, table.size());
    let mut start = Start::open(w * h, n);
    for v in 0..n {
        start.mask[5 * n + v] = false;
    }
    assert_eq!(solver.run(w, h, &start, 1), None);
    // Two settled cells that can't meet: hedge beside a crossroads.
    let mut clash = Start::open(2, n);
    clash.settled = vec![
        Some(index(&set, "hedge", 0)),
        Some(index(&set, "corridor_crossroads", 0)),
    ];
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
    let mut start = Start::open(1, n);
    start.mask = (0..n).map(|v| v == room).collect();
    assert_eq!(solver.run(1, 1, &start, 3), None);
}

#[test]
#[should_panic(expected = "mask doesn't fit the grid")]
fn a_start_of_the_wrong_size_panics() {
    let (set, table) = core();
    let mut start = Start::open(4, table.size());
    start.mask.pop();
    Solver::new(&set, &table).run(2, 2, &start, 0);
}
