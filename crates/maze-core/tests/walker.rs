//! The walker that avoids its own path (2.6 step 2b): seeded and
//! deterministic, prefers cells it hasn't walked, can still go back, and at
//! weight 1 is the plain random walk.

use maze_core::walker::{REVISIT_WEIGHT, Walker};
use maze_core::wfc::SplitMix64;
use maze_core::world::{Pos, Settings, World};
use maze_core::{Table, TileSet};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    let set = TileSet::from_toml(CORE).expect("core loads");
    let table = Table::calculate(&set);
    (set, table)
}

/// A world with plenty of stored cells to stand walkers on.
fn grown(set: &TileSet, table: &Table) -> World {
    let mut world = World::new(set, table, 4, Settings::default()).unwrap();
    world.random_walk((0, 0), 8, 300);
    world
}

#[test]
fn the_same_seed_walks_the_same_way() {
    let (set, table) = core();
    let walk = |seed| {
        let mut world = World::new(&set, &table, 3, Settings::default()).unwrap();
        let mut walker = Walker::new((0, 0), seed);
        let path: Vec<Pos> = (0..200).map(|_| walker.step(&mut world)).collect();
        (path, walker.visits().clone(), world.log().to_vec())
    };
    assert_eq!(walk(5), walk(5));
    assert_ne!(walk(5).0, walk(6).0);
}

#[test]
fn every_step_is_to_a_joined_neighbour_and_is_remembered() {
    let (set, table) = core();
    let mut world = World::new(&set, &table, 7, Settings::default()).unwrap();
    let mut walker = Walker::new((0, 0), 1);
    let mut path = vec![(0, 0)];
    for _ in 0..300 {
        let before = walker.at();
        let ways = world.joined_neighbours(before);
        let at = walker.step(&mut world);
        assert!(ways.contains(&at), "{before:?} to {at:?} isn't a join");
        assert_eq!(walker.at(), at);
        path.push(at);
    }
    // Each cell's count is how often the path stood on it.
    let mut counts = std::collections::BTreeMap::new();
    for p in path {
        *counts.entry(p).or_insert(0) += 1;
    }
    assert_eq!(walker.visits(), &counts);
}

/// At weight 1 every neighbour is alike, so the walker draws as
/// `World::random_step` does, and walks the same world.
#[test]
fn at_weight_1_it_is_the_plain_random_walk() {
    let (set, table) = core();
    let mut a = World::new(&set, &table, 5, Settings::default()).unwrap();
    let mut b = World::new(&set, &table, 5, Settings::default()).unwrap();
    let mut walker = Walker::with_revisit_weight((0, 0), 77, 1.0).unwrap();
    let mut rng = SplitMix64::new(77);
    let mut at = (0, 0);
    for _ in 0..200 {
        at = b.random_step(at, &mut rng);
        b.grow_around(at);
        assert_eq!(walker.step(&mut a), at);
    }
    assert_eq!(a.log(), b.log());
}

/// The second step of many short walks: after one step, the cell it came
/// from is the only walked neighbour, walked once. It goes back at the rate the weights
/// say, w / (w + k) with k unwalked neighbours, which at the default is far
/// below weight 1's 1 / (1 + k), and never 0.
#[test]
fn it_prefers_unwalked_cells_and_still_goes_back() {
    let (set, table) = core();
    let world = grown(&set, &table);
    let starts: Vec<Pos> = world
        .cells()
        .map(|(p, _)| p)
        .filter(|&p| !world.joined_neighbours(p).is_empty())
        .collect();
    let rate = |weight: f64| {
        let (mut back, mut expected, mut trials) = (0, 0.0, 0);
        for (i, &start) in starts.iter().enumerate() {
            for seed in 0..4 {
                let mut walker =
                    Walker::with_revisit_weight(start, (i * 4 + seed) as u64, weight).unwrap();
                let next = walker.advance(&world);
                let k = world.joined_neighbours(next).len() - 1;
                if k == 0 {
                    continue;
                }
                trials += 1;
                expected += weight / (weight + k as f64);
                back += usize::from(walker.advance(&world) == start);
            }
        }
        assert!(trials > 1000, "only {trials} trials");
        (back as f64 / trials as f64, expected / trials as f64)
    };
    let (plain, plain_expected) = rate(1.0);
    let (biased, biased_expected) = rate(REVISIT_WEIGHT);
    println!(
        "back: weight 1 {plain:.3} ({plain_expected:.3} expected), default {biased:.3} ({biased_expected:.3})"
    );
    assert!((plain - plain_expected).abs() < 0.03);
    assert!((biased - biased_expected).abs() < 0.02);
    assert!(biased > 0.0, "going back must stay possible");
    assert!(biased < plain / 4.0);
}

/// In a dead end the only way out is back the way it came, walked or not.
#[test]
fn in_a_dead_end_it_goes_back() {
    let (set, table) = core();
    let world = grown(&set, &table);
    let mut found = 0;
    for (p, _) in world.cells() {
        let ways = world.joined_neighbours(p);
        if ways.len() != 1 {
            continue;
        }
        let way = ways[0];
        // Start beside the dead end; walk until it steps in, then once more.
        for seed in 0..20 {
            let mut walker = Walker::with_revisit_weight(way, seed, 0.001).unwrap();
            if walker.advance(&world) == p {
                assert_eq!(
                    walker.advance(&world),
                    way,
                    "stuck in the dead end at {p:?}"
                );
                found += 1;
                break;
            }
        }
    }
    assert!(found > 0, "the world has no dead end to test");
}
