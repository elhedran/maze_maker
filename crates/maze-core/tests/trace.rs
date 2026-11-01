//! The observer and the trace against whole generations (DR-0034): watching
//! changes nothing, and the trace comes from the attempt that is shown.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_RESTARTS, MAX_SIZE, neighbour};
use maze_core::{
    CollapseStep, Direction, NeighbourAtDraw, Observer, Table, TileSet, generate,
    generate_observed, trace_cell,
};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    let set = TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml failed: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

fn load(text: &str) -> (TileSet, Table) {
    let set = TileSet::from_toml(text).unwrap_or_else(|e| panic!("fixture failed: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

/// Every attempt's collapses, in order: (cell, variant drawn).
#[derive(Default)]
struct Recorder {
    attempts: Vec<(u32, Vec<(usize, usize)>)>,
}

impl Observer for Recorder {
    fn attempt_started(&mut self, attempt: u32) {
        self.attempts.push((attempt, Vec::new()));
    }

    fn collapsed(&mut self, step: &CollapseStep<'_>) {
        // What the solver hands over is self-consistent.
        let total: f64 = step.scores.iter().map(|&(_, s)| s).sum();
        assert_eq!(total, step.total);
        assert!(step.scores.iter().any(|&(c, _)| c == step.chosen));
        assert!(
            step.scores.windows(2).all(|w| w[0].0 < w[1].0),
            "table order"
        );
        assert!(
            step.settled.windows(2).all(|w| w[0].0 < w[1].0),
            "N, E, S, W"
        );
        let (_, cells) = self
            .attempts
            .last_mut()
            .expect("a collapse before any attempt started");
        cells.push((step.cell, step.chosen));
    }
}

/// DR-0034's identity test for one maze: `generate` and `generate_observed`
/// agree, attempts are announced in order, and the last attempt collapsed
/// each filled cell exactly once, to what the maze holds. Returns the record.
fn check_identity(set: &TileSet, table: &Table, seed: u32, w: usize, h: usize) -> Recorder {
    let plain = generate(set, table, seed, w, h).unwrap();
    let mut recorder = Recorder::default();
    let watched = generate_observed(set, table, seed, w, h, &mut recorder).unwrap();
    assert_eq!(plain, watched, "seed {seed} at {w} × {h}");

    let numbers: Vec<u32> = recorder.attempts.iter().map(|(k, _)| *k).collect();
    assert_eq!(numbers, (0..=plain.restarts).collect::<Vec<_>>());
    let (_, last) = recorder.attempts.last().unwrap();
    let mut seen = vec![false; w * h];
    for &(cell, variant) in last {
        assert!(!seen[cell], "seed {seed}: cell {cell} collapsed twice");
        seen[cell] = true;
        assert_eq!(plain.cells[cell], Some(variant), "seed {seed} cell {cell}");
    }
    let filled = plain.cells.iter().filter(|c| c.is_some()).count();
    assert_eq!(last.len(), filled);
    if !plain.failed {
        assert_eq!(last.len(), w * h);
    }
    recorder
}

/// Seeds 0 to 199 at 24 × 16, in four tests so they run in parallel: one
/// takes about 14 s unoptimised.
fn identity_at_the_default_size(seeds: std::ops::Range<u32>) {
    let (set, table) = core();
    for seed in seeds {
        check_identity(&set, &table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT);
    }
}

#[test]
fn watching_changes_nothing_at_the_default_size_seeds_0_to_49() {
    identity_at_the_default_size(0..50);
}

#[test]
fn watching_changes_nothing_at_the_default_size_seeds_50_to_99() {
    identity_at_the_default_size(50..100);
}

#[test]
fn watching_changes_nothing_at_the_default_size_seeds_100_to_149() {
    identity_at_the_default_size(100..150);
}

#[test]
fn watching_changes_nothing_at_the_default_size_seeds_150_to_199() {
    identity_at_the_default_size(150..200);
}

#[test]
fn watching_changes_nothing_at_the_smallest_and_largest_sizes() {
    let (set, table) = core();
    for seed in [0, 1, 2, u32::MAX] {
        check_identity(&set, &table, seed, 1, 1);
    }
    for seed in [0, 9] {
        check_identity(&set, &table, seed, MAX_SIZE, MAX_SIZE);
    }
}

/// DR-0028's fixture: nothing may meet anything, so every attempt
/// contradicts at the first propagation, before any collapse.
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
fn a_maze_that_never_starts_traces_to_nothing() {
    let (set, table) = load(ALWAYS_CONTRADICTS);
    let recorder = check_identity(&set, &table, 5, 3, 3);
    assert_eq!(recorder.attempts.len() as u32, MAX_RESTARTS + 1);
    assert!(recorder.attempts.iter().all(|(_, cells)| cells.is_empty()));
    for cell in [(0, 0), (1, 1), (2, 2)] {
        let t = trace_cell(&set, &table, 5, 3, 3, cell).unwrap();
        assert!(t.failed);
        assert_eq!(t.restarts, MAX_RESTARTS);
        assert_eq!(t.steps, 0);
        assert_eq!(t.collapse, None);
    }
}

/// One tile, found by search: at 3 × 3, whichever cell is placed first,
/// propagation then finds the rest can't be filled. So every attempt places
/// exactly one cell, at random, and contradicts. The tests below assert
/// that much first, so if the solver changes and it stops holding, they say
/// so rather than quietly testing less.
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
fn a_failed_maze_traces_its_last_attempt_only() {
    let (set, table) = load(FAILS_AFTER_ONE);
    let mut earlier_cell_seen = false;
    for seed in 0..10 {
        let recorder = check_identity(&set, &table, seed, 3, 3);
        let maze = generate(&set, &table, seed, 3, 3).unwrap();
        assert!(
            maze.failed && maze.restarts == MAX_RESTARTS,
            "premise: seed {seed}"
        );
        assert!(
            recorder.attempts.iter().all(|(_, c)| c.len() == 1),
            "premise: one cell an attempt"
        );
        let (_, last) = recorder.attempts.last().unwrap();
        let (placed, variant) = last[0];

        // The placed cell: drawn first, with nothing settled round it, and
        // every in-grid neighbour later but left open.
        let t = trace_cell(&set, &table, seed, 3, 3, (placed % 3, placed / 3)).unwrap();
        assert!(t.failed);
        assert_eq!((t.restarts, t.steps), (MAX_RESTARTS, 1));
        let c = t.collapse.expect("the placed cell has a collapse");
        assert_eq!((c.step, c.chosen), (1, variant));
        for d in Direction::ALL {
            let expected = match neighbour(3, 3, placed, d) {
                None => NeighbourAtDraw::Edge,
                Some(_) => NeighbourAtDraw::Later(None),
            };
            assert_eq!(c.neighbours[d.index()], expected);
        }
        assert!(c.candidates.iter().all(|k| k.factors.is_empty()));

        // Every other cell is open, including any an earlier attempt placed.
        for i in (0..9).filter(|&i| i != placed) {
            let t = trace_cell(&set, &table, seed, 3, 3, (i % 3, i / 3)).unwrap();
            assert_eq!(t.collapse, None, "seed {seed} cell {i}");
            assert_eq!(t.steps, 1);
        }
        earlier_cell_seen |= recorder.attempts.iter().any(|(_, c)| c[0].0 != placed);
    }
    assert!(
        earlier_cell_seen,
        "no earlier attempt placed a different cell"
    );
}
