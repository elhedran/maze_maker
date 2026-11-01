//! The stored world grown from open endpoints (DR-0035, DR-0036): its
//! guarantees after every patch, over seeded random walks, and replay.
//!
//! The checks here are written apart from `world.rs`, so a bug there
//! can't hide in a shared helper.

use maze_core::completeness::is_fallback;
use maze_core::world::{Anchor, Pos, Settings, Stage, World, step};
use maze_core::{Direction, SideKind, Table, TileSet};
use std::collections::{BTreeMap, BTreeSet};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> (TileSet, Table) {
    core_with("")
}

/// `core.toml` with `extra` top-level keys, after taking out any
/// `opening_pressure` it has: these tests (and the pinned world) are at no
/// pressure unless `extra` sets one.
fn core_with(extra: &str) -> (TileSet, Table) {
    let text: String = CORE
        .lines()
        .filter(|l| !l.trim_start().starts_with("opening_pressure"))
        .map(|l| format!("{l}\n"))
        .collect();
    let text = text.replace("name = \"core\"\n", &format!("name = \"core\"\n{extra}"));
    let set = TileSet::from_toml(&text).unwrap_or_else(|e| panic!("refused: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

type Snapshot = BTreeMap<Pos, usize>;

fn snapshot(world: &World) -> Snapshot {
    world.cells().collect()
}

fn walkable(side: SideKind) -> bool {
    side != SideKind::Hedge
}

/// DR-0047's seal tile, found here apart from `world.rs`: the first
/// fallback variant with hedge on all four sides.
fn seal_tile(set: &TileSet) -> usize {
    (0..set.variants.len())
        .find(|&v| {
            is_fallback(set, &set.variants[v])
                && set.variants[v].sides.iter().all(|&s| s == SideKind::Hedge)
        })
        .expect("an all-hedge fallback tile")
}

/// The 4-connected regions of unsettled cells inside the box (lo, hi)
/// that don't reach its rim: the bounded regions, when the box is one
/// cell bigger than the stored cells' bounding box.
fn bounded_regions(now: &Snapshot, lo: Pos, hi: Pos) -> Vec<Vec<Pos>> {
    let inside = |(x, y): Pos| (lo.0..=hi.0).contains(&x) && (lo.1..=hi.1).contains(&y);
    let on_rim = |(x, y): Pos| x == lo.0 || x == hi.0 || y == lo.1 || y == hi.1;
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for y in lo.1..=hi.1 {
        for x in lo.0..=hi.0 {
            if now.contains_key(&(x, y)) || !seen.insert((x, y)) {
                continue;
            }
            let mut region = vec![(x, y)];
            let mut at = 0;
            while at < region.len() {
                let p = region[at];
                at += 1;
                for d in Direction::ALL {
                    let q = step(p, d);
                    if inside(q) && !now.contains_key(&q) && seen.insert(q) {
                        region.push(q);
                    }
                }
            }
            if !region.iter().any(|&p| on_rim(p)) {
                out.push(region);
            }
        }
    }
    out
}

/// DR-0036's guarantees for the world as it stands, against `before`, the
/// world before the last patch, with DR-0047's sealed hedge: `sealed` is
/// every cell some patch so far has sealed. Returns the world now.
fn assert_guarantees(
    world: &World,
    table: &Table,
    before: &Snapshot,
    sealed: &BTreeSet<Pos>,
) -> Snapshot {
    let set = world.tile_set();
    let side = |v: usize, d: Direction| set.variants[v].side(d);
    let now = snapshot(world);
    assert_eq!(now.len(), world.len());
    // Rule 5: stored cells never change.
    for (p, v) in before {
        assert_eq!(now.get(p), Some(v), "stored cell {p:?} changed");
    }
    for (&p, &v) in &now {
        for d in Direction::ALL {
            match now.get(&step(p, d)) {
                // Neighbouring stored cells are a pair the table allows.
                Some(&u) => assert!(
                    table.entry(d, v, u).weight > 0.0,
                    "{p:?} ({v}) has {u} to the {d:?} at weight 0"
                ),
                // Rule 1: only hedge, passage or door towards the unsettled.
                None => assert!(
                    matches!(
                        side(v, d),
                        SideKind::Hedge | SideKind::Passage | SideKind::Door
                    ),
                    "{p:?} shows {:?} to the unsettled {d:?}",
                    side(v, d)
                ),
            }
        }
    }
    // Every stored cell is joined to the origin.
    let mut seen = BTreeMap::from([((0, 0), ())]);
    let mut stack = vec![(0, 0)];
    assert!(now.contains_key(&(0, 0)), "the origin isn't stored");
    while let Some(p) = stack.pop() {
        for d in Direction::ALL {
            let q = step(p, d);
            if let Some(&u) = now.get(&q)
                && walkable(side(now[&p], d))
                && walkable(side(u, d.opposite()))
                && seen.insert(q, ()).is_none()
            {
                stack.push(q);
            }
        }
    }
    // DR-0047: every stored cell is joined to the origin, or is sealed
    // and the seal tile.
    let seal = seal_tile(set);
    for (p, &v) in &now {
        if !seen.contains_key(p) {
            assert!(
                sealed.contains(p),
                "{p:?} isn't joined to the origin, nor sealed"
            );
        }
        if sealed.contains(p) {
            assert_eq!(v, seal, "sealed {p:?} isn't the seal tile");
        }
    }
    assert!(
        sealed.iter().all(|p| now.contains_key(p)),
        "a sealed cell isn't stored"
    );
    // Rule 6: an outward endpoint. Flood the unsettled cells from just
    // outside the stored cells' bounding box.
    let (lo, hi) = now.keys().fold(((0, 0), (0, 0)), |(lo, hi), &(x, y)| {
        ((lo.0.min(x), lo.1.min(y)), (hi.0.max(x), hi.1.max(y)))
    });
    let (lo, hi) = ((lo.0 - 1, lo.1 - 1), (hi.0 + 1, hi.1 + 1));
    let inside = |(x, y): Pos| (lo.0..=hi.0).contains(&x) && (lo.1..=hi.1).contains(&y);
    let mut outside: Vec<Pos> = Vec::new();
    for x in lo.0..=hi.0 {
        outside.extend([(x, lo.1), (x, hi.1)]);
    }
    for y in lo.1..=hi.1 {
        outside.extend([(lo.0, y), (hi.0, y)]);
    }
    let mut reached: BTreeMap<Pos, ()> = outside.iter().map(|&p| (p, ())).collect();
    let mut outward = false;
    while let Some(p) = outside.pop() {
        for d in Direction::ALL {
            let q = step(p, d);
            match now.get(&q) {
                Some(&u) => outward |= walkable(side(u, d.opposite())),
                None if inside(q) && reached.insert(q, ()).is_none() => outside.push(q),
                None => {}
            }
        }
    }
    assert!(outward, "no outward endpoint: the player could be boxed in");
    // DR-0047: nothing sealed is left unstored. Every bounded unsettled
    // region has a stored cell showing a walkable side into it.
    for region in bounded_regions(&now, lo, hi) {
        let faced = region.iter().any(|&p| {
            Direction::ALL.into_iter().any(|d| {
                now.get(&step(p, d))
                    .is_some_and(|&u| walkable(side(u, d.opposite())))
            })
        });
        assert!(faced, "bounded region at {:?} was left unsealed", region[0]);
    }
    now
}

/// Rebuilds `walked` patch by patch from its growth log, checking every
/// guarantee after each patch, and that each patch, regenerated from the
/// world before it, comes out as logged. Returns the rebuilt world.
fn replay_checking(walked: &World, set: &TileSet, table: &Table) -> World {
    let settings = walked.settings();
    let mut world = World::replay(set, table, walked.seed(), settings, &[]).unwrap();
    let mut before = Snapshot::new();
    let mut sealed = BTreeSet::new();
    for logged in walked.log() {
        let (again, _) = world.patch(logged.anchor);
        assert_eq!(
            &again, logged,
            "patch at {:?} came out differently",
            logged.anchor
        );
        let patch = world.grow_at(logged.anchor).clone();
        assert!(
            patch.stored.contains(&patch.anchor.cell),
            "the anchor wasn't stored"
        );
        for p in &patch.stored {
            assert!(!before.contains_key(p), "{p:?} was stored twice");
        }
        for p in &patch.spine_cells {
            assert!(patch.stored.contains(p), "spine cell {p:?} wasn't stored");
        }
        if patch.fallback {
            for p in &patch.stored {
                let v = world.get(*p).unwrap();
                assert!(
                    is_fallback(set, &set.variants[v]),
                    "fallback stored {v} at {p:?}"
                );
            }
        }
        // DR-0047: each sealed cell was unsettled before, isn't in
        // `stored`, and lies in a region (of this patch's sealed cells)
        // that touches a kept cell. The list is in (y, x) order.
        let stored: BTreeSet<Pos> = patch.stored.iter().copied().collect();
        let this: BTreeSet<Pos> = patch.sealed.iter().copied().collect();
        let mut ordered = patch.sealed.clone();
        ordered.sort_by_key(|&(x, y)| (y, x));
        assert_eq!(ordered, patch.sealed, "sealed isn't in (y, x) order");
        assert_eq!(this.len(), patch.sealed.len(), "a sealed cell repeats");
        for &p in &patch.sealed {
            assert!(!before.contains_key(&p), "sealed {p:?} was stored before");
            assert!(!stored.contains(&p), "sealed {p:?} is also in stored");
            let mut region = vec![p];
            let mut seen = BTreeSet::from([p]);
            let mut touches = false;
            while let Some(q) = region.pop() {
                for d in Direction::ALL {
                    let r = step(q, d);
                    touches |= stored.contains(&r);
                    if this.contains(&r) && seen.insert(r) {
                        region.push(r);
                    }
                }
            }
            assert!(touches, "sealed {p:?} doesn't touch the patch's kept cells");
        }
        sealed.extend(this);
        before = assert_guarantees(&world, table, &before, &sealed);
    }
    world
}

/// Runs `walks` seeded random walks of `steps` steps on 4 threads, checks
/// each patch by patch, and returns each walk's log of
/// (patches, spines, fallbacks, sealed cells).
fn check_walks(
    settings: Settings,
    extra: &str,
    walks: u32,
    steps: usize,
) -> Vec<(usize, usize, usize, usize)> {
    let (set, table) = core_with(extra);
    let threads = 4;
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (set, table) = (&set, &table);
                scope.spawn(move || {
                    let mut out = Vec::new();
                    for seed in (t..walks).step_by(threads as usize) {
                        let mut walked = World::new(set, table, seed, settings).unwrap();
                        walked.random_walk((0, 0), u64::from(seed) + 1000, steps);
                        let rebuilt = replay_checking(&walked, set, table);
                        assert_eq!(snapshot(&rebuilt), snapshot(&walked), "seed {seed}");
                        let log = walked.log();
                        out.push((
                            log.len(),
                            log.iter().filter(|p| p.spine).count(),
                            log.iter().filter(|p| p.fallback).count(),
                            log.iter().map(|p| p.sealed.len()).sum(),
                        ));
                    }
                    out
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect()
    })
}

/// DR-0036's step 1 test: over 100 seeded random walks of 1,000 steps,
/// after every patch: rule 1 holds, stored cells never change, neighbours
/// are allowed pairs, everything is joined to the origin, and there is an
/// outward endpoint. Each patch also replays as logged.
#[test]
fn a_hundred_walks_keep_every_guarantee_after_every_patch() {
    let stats = check_walks(Settings::default(), "", 100, 1000);
    let patches: usize = stats.iter().map(|s| s.0).sum();
    let spines: usize = stats.iter().map(|s| s.1).sum();
    let fallbacks: usize = stats.iter().map(|s| s.2).sum();
    let sealed: usize = stats.iter().map(|s| s.3).sum();
    println!(
        "100 walks: {patches} patches, {spines} spines, {fallbacks} fallbacks, {sealed} sealed cells"
    );
    assert_eq!(stats.len(), 100);
    // Otherwise the DR-0047 checks above never ran on a sealed cell.
    assert!(sealed > 0, "no walk ever sealed a cell");
}

#[test]
fn a_forced_spine_keeps_every_guarantee() {
    let settings = Settings {
        first_stage: Stage::Spine,
        ..Settings::default()
    };
    let stats = check_walks(settings, "", 12, 300);
    assert!(stats.iter().any(|s| s.1 > 0), "no spine was ever pinned");
}

#[test]
fn a_forced_fallback_keeps_every_guarantee() {
    let settings = Settings {
        first_stage: Stage::Fallback,
        ..Settings::default()
    };
    let stats = check_walks(settings, "", 12, 300);
    assert!(
        stats.iter().all(|s| s.2 == s.0),
        "every patch is a fallback run"
    );
    assert!(stats.iter().any(|s| s.1 > 0), "no spine was ever pinned");
}

#[test]
fn opening_pressure_keeps_every_guarantee_and_changes_growth() {
    check_walks(Settings::default(), "opening_pressure = 0.3\n", 12, 300);
    let grown = |extra| {
        let (set, table) = core_with(extra);
        let mut world = World::new(&set, &table, 5, Settings::default()).unwrap();
        world.random_walk((0, 0), 5, 300);
        snapshot(&world)
    };
    assert_ne!(grown(""), grown("opening_pressure = 0.3\n"));
}

/// DR-0036 rule 10: the same seed and the same walk give the same world,
/// whether walked again, scripted, or replayed from the growth log.
#[test]
fn the_same_seed_and_walk_give_the_same_world() {
    let (set, table) = core();
    let settings = Settings::default();
    let mut first = World::new(&set, &table, 77, settings).unwrap();
    let path = first.random_walk((0, 0), 9, 600);
    let mut second = World::new(&set, &table, 77, settings).unwrap();
    assert_eq!(second.random_walk((0, 0), 9, 600), path);
    assert_eq!(snapshot(&second), snapshot(&first));
    assert_eq!(second.log(), first.log());
    // A scripted walk along the same path.
    let mut scripted = World::new(&set, &table, 77, settings).unwrap();
    scripted.walk(&path);
    assert_eq!(snapshot(&scripted), snapshot(&first));
    // The growth log alone.
    let anchors: Vec<Anchor> = first.log().iter().map(|p| p.anchor).collect();
    let replayed = World::replay(&set, &table, 77, settings, &anchors).unwrap();
    assert_eq!(snapshot(&replayed), snapshot(&first));
    assert_eq!(replayed.log(), first.log());
    // A different world seed grows a different world.
    let other = World::new(&set, &table, 78, settings).unwrap();
    let start = World::new(&set, &table, 77, settings).unwrap();
    assert_ne!(snapshot(&other), snapshot(&start));
}

/// FNV-1a over every stored cell in key order, as (x, y, tile id,
/// orientation), so it doesn't depend on table order.
fn fingerprint(world: &World) -> u64 {
    let set = world.tile_set();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for &b in bytes {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for ((x, y), v) in world.cells() {
        let variant = &set.variants[v];
        eat(&x.to_le_bytes());
        eat(&y.to_le_bytes());
        eat(set.tile_of(variant).id.as_bytes());
        eat(&variant.orientation.to_le_bytes());
    }
    h
}

/// One world pinned. If this fails, the generator changed what a seed and
/// walk grow: fine while nothing is saved (DR-0036 is a two-way door), but
/// it should be on purpose.
#[test]
fn one_world_is_pinned() {
    let (set, table) = core();
    let mut world = World::new(&set, &table, 2026, Settings::default()).unwrap();
    world.random_walk((0, 0), 2026, 400);
    println!(
        "pinned world: {} cells, {} patches, {:#018x}",
        world.len(),
        world.log().len(),
        fingerprint(&world)
    );
    assert_eq!(
        (world.len(), world.log().len(), fingerprint(&world)),
        (253, 9, 0x57b5_920b_f7fe_922a)
    );
}

#[test]
fn a_new_world_starts_with_one_patch_at_the_origin() {
    let (set, table) = core();
    let world = World::new(&set, &table, 1, Settings::default()).unwrap();
    let first = &world.log()[0];
    assert_eq!(
        first.anchor,
        Anchor {
            cell: (0, 0),
            from: None
        }
    );
    let origin = world.get((0, 0)).expect("the origin is stored");
    assert!(set.variants[origin].sides.iter().any(|&s| walkable(s)));
    // Growth stopped because no anchor is within r of the player.
    assert_eq!(world.nearest_anchor((0, 0)), None);
    // The palette names every stored tile by id and orientation.
    for (_, v) in world.cells() {
        let variant = &set.variants[v];
        let entry = (set.tile_of(variant).id.clone(), variant.orientation);
        assert!(world.palette().contains(&entry));
    }
}

#[test]
fn an_incomplete_tile_set_is_refused() {
    let text = CORE.replace("passage_passage = 1.0", "passage_passage = 0.0");
    let set = TileSet::from_toml_allow_incomplete(&text).unwrap();
    let table = Table::calculate(&set);
    let err = World::new(&set, &table, 1, Settings::default())
        .err()
        .expect("refused");
    assert!(err[0].to_string().contains("DR-0036 rule 7"), "{err:?}");
}
