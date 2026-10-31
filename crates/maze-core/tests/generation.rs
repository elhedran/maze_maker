//! WFC generation and the measures against the real core tile set and small
//! fixtures (DR-0028, DR-0030; roadmap 2.1 "done when").

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_RESTARTS, MAX_SIZE, neighbour};
use maze_core::{
    Direction, GenerateError, Generation, Table, TileSet, count_areas, generate, measures,
};
use std::time::Instant;

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    let set = TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml failed: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

fn default_maze(set: &TileSet, table: &Table, seed: u32) -> Generation {
    generate(set, table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap()
}

/// DR-0028 rule 8, the basic check: every side of every tile can meet
/// something with weight above 0.
#[test]
fn every_side_of_every_tile_can_meet_something() {
    let (set, table) = core();
    for d in Direction::ALL {
        for (row, variant) in set.variants.iter().enumerate() {
            assert!(
                table.row_total(d, row) > 0.0,
                "{} turned {} has nothing it can meet to the {d:?}",
                set.tile_of(variant).id,
                variant.orientation
            );
        }
    }
}

#[test]
fn same_seed_gives_the_same_maze() {
    let (set, table) = core();
    for seed in [0, 1, 42, u32::MAX] {
        assert_eq!(
            default_maze(&set, &table, seed),
            default_maze(&set, &table, seed)
        );
        // A freshly parsed copy of the same file too.
        let (set2, table2) = core();
        assert_eq!(
            default_maze(&set, &table, seed),
            default_maze(&set2, &table2, seed)
        );
    }
}

#[test]
fn different_seeds_give_different_mazes() {
    let (set, table) = core();
    let mazes: Vec<Vec<Option<usize>>> = (0..10)
        .map(|seed| default_maze(&set, &table, seed).cells)
        .collect();
    for i in 0..mazes.len() {
        for j in i + 1..mazes.len() {
            assert_ne!(mazes[i], mazes[j], "seeds {i} and {j} gave the same maze");
        }
    }
}

/// Every pair of neighbouring cells is one the table allows, both ways
/// round, over 200 seeds. Also: a maze that didn't fail is full.
#[test]
fn neighbours_always_have_probability_above_zero() {
    let (set, table) = core();
    let mut failures = 0;
    let mut restarts = 0;
    for seed in 0..200 {
        let maze = default_maze(&set, &table, seed);
        restarts += maze.restarts;
        if maze.failed {
            failures += 1;
        } else {
            assert!(maze.cells.iter().all(Option::is_some), "seed {seed}");
        }
        for i in 0..maze.cells.len() {
            for d in Direction::ALL {
                let Some(j) = neighbour(maze.width, maze.height, i, d) else {
                    continue;
                };
                if let (Some(a), Some(b)) = (maze.cells[i], maze.cells[j]) {
                    assert!(
                        table.probability(d, a, b) > 0.0,
                        "seed {seed}: cell {i} ({a}) has {b} to the {d:?}"
                    );
                }
            }
        }
    }
    // Not a pass mark, a signal: printed with --nocapture.
    println!("200 seeds: {failures} failed, {restarts} restarts in all");
    assert!(failures < 200, "every seed failed");
}

/// A tile set where nothing may meet anything (the only base is 0), so every
/// attempt contradicts at the first propagation.
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

#[test]
fn a_contradiction_fails_after_the_restart_limit() {
    let set = TileSet::from_toml(ALWAYS_CONTRADICTS).unwrap();
    let table = Table::calculate(&set);
    let maze = generate(&set, &table, 5, 3, 3).unwrap();
    assert!(maze.failed);
    assert_eq!(maze.restarts, MAX_RESTARTS);
    assert_eq!(maze.cells, vec![None; 9]);
    // With no neighbours there is nothing to contradict.
    let single = generate(&set, &table, 5, 1, 1).unwrap();
    assert!(!single.failed);
    assert_eq!(single.cells, vec![Some(0)]);
}

#[test]
fn a_tile_of_weight_zero_is_never_placed() {
    let text = ALWAYS_CONTRADICTS
        .replace("hedge_hedge = 0.0", "hedge_hedge = 1.0")
        .replace("weight = 1.0", "weight = 0.0");
    let set = TileSet::from_toml(&text).unwrap();
    let table = Table::calculate(&set);
    let maze = generate(&set, &table, 5, 1, 1).unwrap();
    assert!(maze.failed);
    assert_eq!(maze.cells, vec![None]);
}

/// A set that can contradict part-way: the crossroads may only meet
/// passages, and a dead end's passage wants a passage back. Whatever
/// happens, generation ends without a panic, and a non-failed maze is full.
#[test]
fn generation_ends_on_awkward_sets() {
    let text = r#"
format = 1
name = "awkward"

[bases]
hedge_hedge = 1.0
passage_passage = 1.0
passage_door = 1.0
door_door = 1.0
floor_floor = 1.0
floor_wall_floor_wall = 1.0

[[tile]]
id = "crossroads"
name = "Crossroads"
kind = "corridor"
sides = ["passage", "passage", "passage", "passage"]
orientations = 1
weight = 1.0

[[tile]]
id = "dead_end"
name = "Dead end"
kind = "corridor"
sides = ["passage", "hedge", "hedge", "hedge"]
orientations = 4
weight = 3.0
"#;
    let set = TileSet::from_toml(text).unwrap();
    let table = Table::calculate(&set);
    for seed in 0..50 {
        let maze = generate(&set, &table, seed, 6, 5).unwrap();
        assert!(maze.restarts <= MAX_RESTARTS);
        assert_eq!(maze.failed, maze.cells.iter().any(Option::is_none));
    }
}

#[test]
fn sizes_outside_1_to_64_are_refused() {
    let (set, table) = core();
    for (w, h) in [(0, 16), (24, 0), (65, 16), (24, 65)] {
        let err = generate(&set, &table, 1, w, h).unwrap_err();
        assert_eq!(
            err,
            GenerateError::BadSize {
                width: w,
                height: h
            }
        );
        assert!(err.to_string().contains("from 1 to 64"), "{err}");
    }
    assert_eq!(generate(&set, &table, 1, 1, 1).unwrap().cells.len(), 1);
    let big = generate(&set, &table, 1, MAX_SIZE, MAX_SIZE).unwrap();
    assert_eq!(big.cells.len(), MAX_SIZE * MAX_SIZE);
}

#[test]
fn the_default_maze_is_quick() {
    let (set, table) = core();
    let start = Instant::now();
    let runs = 20;
    for seed in 0..runs {
        default_maze(&set, &table, seed);
    }
    let each = start.elapsed() / runs;
    println!("24 × 16: {each:?} a maze (this build profile)");
    // Generous, so a slow CI box doesn't flake; the real figure is printed.
    assert!(each.as_millis() < 1000, "{each:?}");
}

// Measures (DR-0030).

/// The table index of `id` turned `orientation` times.
fn v(set: &TileSet, id: &str, orientation: u32) -> Option<usize> {
    Some(
        set.variants
            .iter()
            .position(|x| set.tile_of(x).id == id && x.orientation == orientation)
            .unwrap_or_else(|| panic!("no {id} turned {orientation}")),
    )
}

#[test]
fn measures_on_a_hand_built_grid() {
    let (set, _) = core();
    let hedge = v(&set, "hedge", 0);
    let straight = v(&set, "corridor_straight", 0); // passages N and S
    let dead_end = v(&set, "corridor_dead_end", 0); // passage N
    let room = v(&set, "open_room", 0);
    // A corridor down the middle, and a room beside it it doesn't open onto.
    #[rustfmt::skip]
    let cells = vec![
        hedge, straight, room,
        hedge, straight, room,
        hedge, dead_end, hedge,
    ];
    let counts = count_areas(&set, &cells, 3, 3);
    assert_eq!(counts.centre, (1, 1));
    assert_eq!(counts.walkable, 5);
    assert_eq!(counts.reachable_from_centre, 3);
    assert_eq!(counts.separate_areas, 2);
    let m = measures(&counts);
    assert_eq!(m[0].name, "Reachable from centre");
    assert_eq!(m[0].value, "3 of 5 walkable cells (60%)");
    assert_eq!(
        m[0].explanation,
        "Share of walkable cells the player could walk to from the middle of the maze."
    );
    assert_eq!(m[1].name, "Separate areas");
    assert_eq!(m[1].value, "2");
    assert_eq!(
        m[1].explanation,
        "Groups of walkable cells cut off from each other. 1 means everything joins up."
    );
}

#[test]
fn a_hedge_centre_falls_back_to_the_nearest_walkable_cell() {
    let (set, _) = core();
    let hedge = v(&set, "hedge", 0);
    let room = v(&set, "open_room", 0);
    // Three cells at distance 1: ties go to the lowest row, then column.
    #[rustfmt::skip]
    let cells = vec![
        hedge, hedge, hedge,
        room,  hedge, room,
        hedge, room,  hedge,
    ];
    let counts = count_areas(&set, &cells, 3, 3);
    assert_eq!(counts.centre, (0, 1));
    assert_eq!(counts.separate_areas, 3);
    assert_eq!(measures(&counts)[0].value, "1 of 3 walkable cells (33%)");
}

#[test]
fn open_cells_of_a_failed_maze_are_not_walkable() {
    let (set, _) = core();
    let room = v(&set, "open_room", 0);
    let counts = count_areas(&set, &[None, room, room, None], 2, 2);
    // The middle (1, 1) is open, so not walkable; (1, 0) is the nearest.
    assert_eq!(counts.centre, (1, 0));
    assert_eq!(counts.walkable, 2);
    assert_eq!(counts.separate_areas, 2);

    let empty = count_areas(&set, &[None; 9], 3, 3);
    assert_eq!(empty.centre, (1, 1));
    let m = measures(&empty);
    assert_eq!(m[0].value, "No walkable cells");
    assert_eq!(m[1].value, "No walkable cells");
}

#[test]
fn percentages_round_to_the_nearest_whole_number() {
    let at = |reachable, walkable| {
        measures(&maze_core::AreaCounts {
            centre: (0, 0),
            walkable,
            reachable_from_centre: reachable,
            separate_areas: 1,
        })[0]
            .value
            .clone()
    };
    assert_eq!(at(2, 3), "2 of 3 walkable cells (67%)");
    assert_eq!(at(1, 8), "1 of 8 walkable cells (13%)");
    assert_eq!(at(412, 480), "412 of 480 walkable cells (86%)");
    assert_eq!(at(7, 7), "7 of 7 walkable cells (100%)");
}

#[test]
fn measures_of_a_generated_maze_add_up() {
    let (set, table) = core();
    let maze = default_maze(&set, &table, 7);
    let counts = count_areas(&set, &maze.cells, maze.width, maze.height);
    assert!(counts.reachable_from_centre <= counts.walkable);
    assert!(counts.walkable == 0 || counts.reachable_from_centre >= 1);
    assert!(counts.separate_areas <= counts.walkable);
}
