//! Opening pressure in the single maze (DR-0036 rule 9, roadmap 2.3): the
//! same seed and the same numbers give the same maze, and a pressure below 1
//! changes what a seed draws. The trace side is in `src/trace.rs`'s tests;
//! the pins at pressure 1 are in `seeds_pinned.rs`.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use maze_core::{Table, TileSet, generate};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

/// `core.toml` with its `opening_pressure`, if any, replaced by `value`.
fn core_at(value: &str) -> (TileSet, Table) {
    let text: String = CORE
        .lines()
        .filter(|l| !l.trim_start().starts_with("opening_pressure"))
        .map(|l| format!("{l}\n"))
        .collect::<String>()
        .replace(
            "name = \"core\"\n",
            &format!("name = \"core\"\nopening_pressure = {value}\n"),
        );
    let set = TileSet::from_toml(&text).unwrap_or_else(|e| panic!("refused: {e:?}"));
    assert_eq!(set.opening_pressure.to_string(), value);
    let table = Table::calculate(&set);
    (set, table)
}

#[test]
fn same_seed_and_pressure_give_the_same_maze() {
    // Loaded twice, so nothing is shared between the two runs.
    let (a, a_table) = core_at("0.5");
    let (b, b_table) = core_at("0.5");
    for seed in [0, 1, 42, 2026, u32::MAX] {
        let first = generate(&a, &a_table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
        let second = generate(&b, &b_table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
        assert!(!first.failed, "seed {seed}");
        assert_eq!(first, second, "seed {seed}");
    }
}

#[test]
fn a_pressure_below_1_changes_the_maze() {
    let (off, off_table) = core_at("1");
    let (on, on_table) = core_at("0.5");
    let changed = (0..20)
        .filter(|&seed| {
            let plain = generate(&off, &off_table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
            let pressed = generate(&on, &on_table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
            plain != pressed
        })
        .count();
    // Nearly every maze has a candidate that closes something off, so
    // nearly every seed should draw differently.
    assert!(changed >= 15, "only {changed} of 20 seeds changed");
}
