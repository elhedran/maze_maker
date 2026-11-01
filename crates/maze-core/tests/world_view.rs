//! DR-0041's accessors on the world, for the tool's World tab: the
//! frontier, bounds, joined, the random walk's single step, anchors, and
//! each patch's discarded cells.
//!
//! The reference checks here are written apart from `world.rs`, so a bug
//! there can't hide in a shared helper.

use maze_core::wfc::SplitMix64;
use maze_core::world::{Anchor, Endpoint, Pos, Settings, World, step};
use maze_core::{Direction, SideKind, Table, TileSet};
use std::collections::{BTreeMap, BTreeSet};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    let set = TileSet::from_toml(CORE).expect("core.toml loads");
    let table = Table::calculate(&set);
    (set, table)
}

type Snapshot = BTreeMap<Pos, usize>;

fn snapshot(world: &World) -> Snapshot {
    world.cells().collect()
}

/// Grows around `player` one patch at a time, as the tool does, calling
/// `each` after every patch with the world, the cells stored before it
/// and its discards.
fn grow_around_with(
    world: &mut World,
    player: Pos,
    each: &mut impl FnMut(&World, &Snapshot, &[Pos]),
) {
    while let Some(anchor) = world.nearest_anchor(player) {
        let before = snapshot(world);
        assert!(world.is_anchor(anchor));
        let discards = world.grow_with_discards(anchor);
        each(world, &before, &discards);
    }
}

/// A new world grown patch by patch the tool's way, then walked with
/// `random_step`, calling `each` after every patch.
fn walk_with(
    set: &TileSet,
    table: &Table,
    seed: u32,
    walk_seed: u64,
    steps: usize,
    each: &mut impl FnMut(&World, &Snapshot, &[Pos]),
) -> (World, Vec<Pos>) {
    let mut world = World::replay(set, table, seed, Settings::default(), &[]).unwrap();
    let origin = Anchor {
        cell: (0, 0),
        from: None,
    };
    assert!(world.is_anchor(origin));
    let discards = world.grow_with_discards(origin);
    each(&world, &Snapshot::new(), &discards);
    grow_around_with(&mut world, (0, 0), each);
    let mut rng = SplitMix64::new(walk_seed);
    let mut at = (0, 0);
    let mut path = Vec::new();
    for _ in 0..steps {
        at = world.random_step(at, &mut rng);
        path.push(at);
        grow_around_with(&mut world, at, each);
    }
    (world, path)
}

#[test]
fn growing_patch_by_patch_equals_world_new() {
    let (set, table) = core();
    for seed in 0..20 {
        let (grown, _) = walk_with(&set, &table, seed, 0, 0, &mut |_, _, _| {});
        let new = World::new(&set, &table, seed, Settings::default()).unwrap();
        assert_eq!(snapshot(&grown), snapshot(&new), "seed {seed}");
        assert_eq!(grown.log(), new.log(), "seed {seed}");
    }
}

#[test]
fn random_step_walks_as_random_walk_does() {
    let (set, table) = core();
    for seed in 0..20 {
        let walk_seed = u64::from(seed) * 7 + 3;
        let (stepped, path) = walk_with(&set, &table, seed, walk_seed, 500, &mut |_, _, _| {});
        let mut walked = World::new(&set, &table, seed, Settings::default()).unwrap();
        assert_eq!(
            walked.random_walk((0, 0), walk_seed, 500),
            path,
            "seed {seed}"
        );
        assert_eq!(snapshot(&stepped), snapshot(&walked), "seed {seed}");
        assert_eq!(stepped.log(), walked.log(), "seed {seed}");
    }
}

fn walkable(side: SideKind) -> bool {
    side != SideKind::Hedge
}

/// Whether unsettled `anchor` reaches the rim of the stored cells' box,
/// grown by one, through unsettled cells: a flood of its own.
fn reaches_out(cells: &Snapshot, anchor: Pos) -> bool {
    let (lo, hi) = cells.keys().fold(((0, 0), (0, 0)), |(lo, hi), &(x, y)| {
        ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)))
    });
    let (lo, hi) = ((lo.0 - 1, lo.1 - 1), (hi.0 + 1, hi.1 + 1));
    let mut seen = BTreeSet::from([anchor]);
    let mut stack = vec![anchor];
    while let Some((x, y)) = stack.pop() {
        if x == lo.0 || x == hi.0 || y == lo.1 || y == hi.1 {
            return true;
        }
        for d in Direction::ALL {
            let q = step((x, y), d);
            if !cells.contains_key(&q) && seen.insert(q) {
                stack.push(q);
            }
        }
    }
    false
}

/// Every stored cell's walkable side facing an unsettled cell, in (y, x)
/// then N, E, S, W order, each with its own flood.
fn brute_frontier(world: &World) -> Vec<Endpoint> {
    let set = world.tile_set();
    let cells = snapshot(world);
    let mut sorted: Vec<(Pos, usize)> = cells.iter().map(|(&p, &v)| (p, v)).collect();
    sorted.sort_by_key(|&((x, y), _)| (y, x));
    let mut out = Vec::new();
    for (cell, v) in sorted {
        for facing in Direction::ALL {
            let anchor = step(cell, facing);
            if !cells.contains_key(&anchor) && walkable(set.variants[v].side(facing)) {
                out.push(Endpoint {
                    cell,
                    facing,
                    outward: reaches_out(&cells, anchor),
                });
            }
        }
    }
    out
}

#[test]
fn frontier_matches_a_brute_force_list_after_every_patch() {
    let (set, table) = core();
    let mut patches = 0;
    for seed in 0..6 {
        walk_with(&set, &table, seed, 11, 300, &mut |world, _, _| {
            patches += 1;
            assert_eq!(world.frontier(), brute_frontier(world), "seed {seed}");
        });
    }
    assert!(patches > 6);
}

/// DR-0036 rule 6 as the frontier reports it, after every patch of 100
/// walks of 1,000 steps.
#[test]
fn every_patch_of_a_hundred_walks_leaves_an_outward_endpoint() {
    let (set, table) = core();
    std::thread::scope(|scope| {
        for t in 0..4u32 {
            let (set, table) = (&set, &table);
            scope.spawn(move || {
                for seed in (t..100).step_by(4) {
                    walk_with(
                        set,
                        table,
                        seed,
                        u64::from(seed) + 1000,
                        1000,
                        &mut |w, _, _| {
                            assert!(
                                w.frontier().iter().any(|e| e.outward),
                                "seed {seed}: no outward endpoint"
                            );
                        },
                    );
                }
            });
        }
    });
}

/// `joined` against the definition: both facing sides walkable.
#[test]
fn bounds_and_joined_agree_with_the_cells() {
    let (set, table) = core();
    let empty = World::replay(&set, &table, 1, Settings::default(), &[]).unwrap();
    assert_eq!(empty.bounds(), None);
    assert!(empty.frontier().is_empty());
    assert!(!empty.joined((0, 0), Direction::North));
    for seed in 0..5 {
        let (world, _) = walk_with(&set, &table, seed, 5, 200, &mut |_, _, _| {});
        let cells = snapshot(&world);
        let xs = cells.keys().map(|p| p.0);
        let ys = cells.keys().map(|p| p.1);
        let lo = (xs.clone().min().unwrap(), ys.clone().min().unwrap());
        let hi = (xs.max().unwrap(), ys.max().unwrap());
        assert_eq!(world.bounds(), Some((lo, hi)));
        for (&p, &v) in &cells {
            for d in Direction::ALL {
                let expected = cells.get(&step(p, d)).is_some_and(|&u| {
                    walkable(set.variants[v].side(d))
                        && walkable(set.variants[u].side(d.opposite()))
                });
                assert_eq!(world.joined(p, d), expected, "{p:?} {d:?}");
            }
        }
        // An unsettled cell is never joined.
        let outside = (hi.0 + 1, hi.1);
        assert!(!world.joined(outside, Direction::West));
    }
}

#[test]
fn discards_and_stored_cells_fill_each_window_exactly() {
    let (set, table) = core();
    let mut patches = 0;
    for seed in 0..6 {
        walk_with(
            &set,
            &table,
            seed,
            21,
            300,
            &mut |world, before, discards| {
                patches += 1;
                let patch = world.log().last().unwrap();
                let size = world.settings().patch_size as i64;
                let (nx, ny) = patch.window;
                let window: BTreeSet<Pos> = (ny..ny + size)
                    .flat_map(|y| (nx..nx + size).map(move |x| (x, y)))
                    .collect();
                let stored: BTreeSet<Pos> = patch.stored.iter().copied().collect();
                let earlier: BTreeSet<Pos> = window
                    .iter()
                    .copied()
                    .filter(|p| before.contains_key(p))
                    .collect();
                let thrown: BTreeSet<Pos> = discards.iter().copied().collect();
                assert_eq!(thrown.len(), discards.len(), "a discard repeats");
                // Row-major order.
                let mut ordered = discards.to_vec();
                ordered.sort_by_key(|&(x, y)| (y, x));
                assert_eq!(ordered, discards);
                for p in discards {
                    assert!(world.get(*p).is_none(), "discard {p:?} is stored");
                }
                assert!(thrown.is_disjoint(&stored) && thrown.is_disjoint(&earlier));
                assert!(stored.is_disjoint(&earlier));
                let all: BTreeSet<Pos> = thrown.union(&stored).chain(&earlier).copied().collect();
                assert_eq!(all, window, "seed {seed}");
            },
        );
    }
    assert!(patches > 6);
}

#[test]
fn is_anchor_accepts_only_what_grow_at_would() {
    let (set, table) = core();
    let world = World::new(&set, &table, 3, Settings::default()).unwrap();
    // Not empty, so no second origin.
    assert!(!world.is_anchor(Anchor {
        cell: (0, 0),
        from: None
    }));
    // A stored cell is never an anchor.
    assert!(!world.is_anchor(Anchor {
        cell: (0, 0),
        from: Some(Direction::North)
    }));
    // Each frontier anchor is an anchor in exactly one direction: the first
    // stored neighbour, N, E, S, W from it, showing a walkable side.
    for e in world.frontier() {
        let cell = step(e.cell, e.facing);
        let accepted: Vec<Direction> = Direction::ALL
            .into_iter()
            .filter(|&d| {
                world.is_anchor(Anchor {
                    cell,
                    from: Some(d),
                })
            })
            .collect();
        assert_eq!(accepted.len(), 1, "{cell:?}");
    }
}
