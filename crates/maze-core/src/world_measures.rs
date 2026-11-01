//! 2.6 step 2's measures of a grown world (DR-0036's list, re-cut for
//! patches). Plain functions of a `World`, so they are tested here on small
//! worlds; `examples/measures.rs` runs the sweep and prints the report.
//!
//! Definitions (engineers' choices, two-way doors):
//!
//! - **Open endpoint:** a stored cell's walkable side facing an unsettled
//!   cell, counted per side (DR-0036's terms). **Outward:** its anchor is
//!   reached by a flood over unsettled cells from just outside the box
//!   around the stored cells (rule 6's reference method).
//! - **Room:** a group of stored room-kind cells joined through floor
//!   sides: two side-sharing room cells are in the same room when both
//!   facing sides are `floor` or `floor_wall_*`. A door joins a room to
//!   what lies outside it, so two doorways facing each other join two
//!   rooms, not one. The room's patch is the patch that stored its cells.
//! - **Touches its patch's edge:** some room cell lies on the outer ring of
//!   that patch's window.
//! - **Connectivity**, comparable with DR-0030 and Wren's 2.1 review:
//!   walkable cells are those whose tile isn't hedge-kind; reachable cells
//!   are walkable cells joined to the start through walkable sides; dead
//!   ends are walkable cells with exactly one joined neighbour and nothing
//!   still to grow from them (no open endpoint); loops are the independent
//!   cycles of the reachable area (joins − cells + 1).

use crate::measures::connected;
use crate::side::{Direction, SideKind};
use crate::tile_set::{TileKind, TileSet};
use crate::wfc::SplitMix64;
use crate::world::{Anchor, Pos, World, step};
use std::collections::{BTreeMap, BTreeSet};

fn walkable(side: SideKind) -> bool {
    side != SideKind::Hedge
}

fn floor(side: SideKind) -> bool {
    !matches!(side, SideKind::Hedge | SideKind::Passage | SideKind::Door)
}

/// The frontier as it stands.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Frontier {
    pub open: usize,
    pub outward: usize,
}

/// Open and outward endpoints of `world`, counted per side.
pub fn frontier(world: &World) -> Frontier {
    let set = world.tile_set();
    let cells: BTreeMap<Pos, usize> = world.cells().collect();
    let Some(((lx, ly), (hx, hy))) = bounds(cells.keys()) else {
        return Frontier::default();
    };
    // Unsettled cells reached from the rim of the box one bigger than the
    // stored cells' box.
    let (lx, ly, hx, hy) = (lx - 1, ly - 1, hx + 1, hy + 1);
    let inside = |p: Pos| (lx..=hx).contains(&p.0) && (ly..=hy).contains(&p.1);
    let mut reached = BTreeSet::new();
    let mut stack = Vec::new();
    for x in lx..=hx {
        stack.extend([(x, ly), (x, hy)]);
    }
    for y in ly..=hy {
        stack.extend([(lx, y), (hx, y)]);
    }
    stack.retain(|p| !cells.contains_key(p));
    reached.extend(stack.iter().copied());
    while let Some(p) = stack.pop() {
        for d in Direction::ALL {
            let q = step(p, d);
            if inside(q) && !cells.contains_key(&q) && reached.insert(q) {
                stack.push(q);
            }
        }
    }
    let mut out = Frontier::default();
    for (&p, &v) in &cells {
        for d in Direction::ALL {
            let q = step(p, d);
            if walkable(set.variants[v].side(d)) && !cells.contains_key(&q) {
                out.open += 1;
                out.outward += usize::from(reached.contains(&q) || !inside(q));
            }
        }
    }
    out
}

fn bounds<'a>(cells: impl Iterator<Item = &'a Pos>) -> Option<(Pos, Pos)> {
    cells.fold(None, |b, &p| {
        Some(match b {
            None => (p, p),
            Some((lo, hi)) => (
                (p.0.min(lo.0), p.1.min(lo.1)),
                (p.0.max(hi.0), p.1.max(hi.1)),
            ),
        })
    })
}

/// One room of a grown world.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    pub cells: usize,
    /// The bounding box's width and height.
    pub width: usize,
    pub height: usize,
    /// The patch (index into the growth log) that stored it; the first
    /// cell's, if the room somehow spans patches (see `spans_patches`).
    pub patch: usize,
    /// Some cell lies on the outer ring of its patch's window.
    pub touches_edge: bool,
    /// Its cells came from more than one patch. Rule 1 says this can't
    /// happen; the report counts it as a check.
    pub spans_patches: bool,
    /// Room cells on the outer ring of the window.
    pub edge_cells: usize,
}

/// Every room of `world`, in key order of its first cell.
pub fn rooms(world: &World) -> Vec<Room> {
    let set = world.tile_set();
    let size = world.settings().patch_size as i64;
    let mut patch_of: BTreeMap<Pos, usize> = BTreeMap::new();
    for (k, patch) in world.log().iter().enumerate() {
        for &p in &patch.stored {
            patch_of.insert(p, k);
        }
    }
    let room: BTreeMap<Pos, usize> = world
        .cells()
        .filter(|&(_, v)| set.tile_of(&set.variants[v]).kind == TileKind::Room)
        .collect();
    let on_ring = |p: Pos, k: usize| {
        let (wx, wy) = world.log()[k].window;
        let (x, y) = (p.0 - wx, p.1 - wy);
        x == 0 || y == 0 || x == size - 1 || y == size - 1
    };
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for &start in room.keys() {
        if !seen.insert(start) {
            continue;
        }
        let mut members = vec![start];
        let mut stack = vec![start];
        while let Some(p) = stack.pop() {
            for d in Direction::ALL {
                let q = step(p, d);
                if let Some(&u) = room.get(&q)
                    && floor(set.variants[room[&p]].side(d))
                    && floor(set.variants[u].side(d.opposite()))
                    && seen.insert(q)
                {
                    members.push(q);
                    stack.push(q);
                }
            }
        }
        let ((lx, ly), (hx, hy)) = bounds(members.iter()).expect("a room has a cell");
        let patch = patch_of[&start];
        let edge_cells = members.iter().filter(|&&p| on_ring(p, patch)).count();
        out.push(Room {
            cells: members.len(),
            width: (hx - lx + 1) as usize,
            height: (hy - ly + 1) as usize,
            patch,
            touches_edge: edge_cells > 0,
            spans_patches: members.iter().any(|p| patch_of[p] != patch),
            edge_cells,
        });
    }
    out
}

/// The chance that a `width` × `height` box placed at random inside a
/// `size` × `size` window touches the window's outer ring: the baseline for
/// "rooms avoid patch edges".
pub fn edge_chance(width: usize, height: usize, size: usize) -> f64 {
    if width > size || height > size {
        return 1.0;
    }
    let places = ((size - width + 1) * (size - height + 1)) as f64;
    let clear = (size.saturating_sub(width + 1) * size.saturating_sub(height + 1)) as f64;
    1.0 - clear / places
}

/// Connectivity of a set of cells, from the player's side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Connectivity {
    pub walkable: usize,
    /// Walkable cells joined to the start, counting it.
    pub reachable: usize,
    /// Groups of joined walkable cells.
    pub areas: usize,
    /// Reachable cells with one joined neighbour and no open endpoint.
    pub dead_ends: usize,
    /// Independent cycles of the reachable area.
    pub loops: usize,
}

/// Connectivity of `cells` (variant indices by position) from `start`.
/// With `grown`, a missing neighbour is unsettled, so a walkable side
/// facing it is an open endpoint; without, it is past a fixed maze's border
/// and leads nowhere (DR-0030).
pub fn connectivity(
    set: &TileSet,
    cells: &BTreeMap<Pos, usize>,
    start: Pos,
    grown: bool,
) -> Connectivity {
    let is_walkable = |v: usize| set.tile_of(&set.variants[v]).kind != TileKind::Hedge;
    let walkable_cells: Vec<Pos> = cells
        .iter()
        .filter(|&(_, &v)| is_walkable(v))
        .map(|(&p, _)| p)
        .collect();
    let joined = |p: Pos, d: Direction| -> Option<Pos> {
        let q = step(p, d);
        let (&a, &b) = (cells.get(&p)?, cells.get(&q)?);
        (is_walkable(b) && connected(set, a, d, b)).then_some(q)
    };
    let mut area: BTreeMap<Pos, usize> = BTreeMap::new();
    let mut out = Connectivity {
        walkable: walkable_cells.len(),
        ..Connectivity::default()
    };
    for &first in &walkable_cells {
        if area.contains_key(&first) {
            continue;
        }
        area.insert(first, out.areas);
        let mut stack = vec![first];
        while let Some(p) = stack.pop() {
            for d in Direction::ALL {
                if let Some(q) = joined(p, d)
                    && !area.contains_key(&q)
                {
                    area.insert(q, out.areas);
                    stack.push(q);
                }
            }
        }
        out.areas += 1;
    }
    let Some(&home) = area.get(&start) else {
        return out;
    };
    let mut joins = 0;
    for (&p, _) in area.iter().filter(|&(_, &a)| a == home) {
        out.reachable += 1;
        let degree = Direction::ALL
            .into_iter()
            .filter(|&d| joined(p, d).is_some())
            .count();
        joins += degree;
        let open = grown
            && Direction::ALL.into_iter().any(|d| {
                walkable(set.variants[cells[&p]].side(d)) && !cells.contains_key(&step(p, d))
            });
        out.dead_ends += usize::from(degree == 1 && !open);
    }
    // Each join was counted from both ends.
    out.loops = (joins / 2 + 1).saturating_sub(out.reachable);
    out
}

/// A fixed maze's cells, row-major as `generate` gives them, by position;
/// cells a failed attempt left open are left out.
pub fn grid_cells(cells: &[Option<usize>], width: usize) -> BTreeMap<Pos, usize> {
    cells
        .iter()
        .enumerate()
        .filter_map(|(i, c)| Some((((i % width) as i64, (i / width) as i64), (*c)?)))
        .collect()
}

/// `World::random_walk`'s walk, step for step, with each patch grown by
/// `grow` (so the caller can time it), and `after` told the world after
/// each step's growth. Gives the same world as `random_walk` when `grow`
/// calls `grow_at`.
pub fn walk(
    world: &mut World,
    from: Pos,
    seed: u64,
    steps: usize,
    mut grow: impl FnMut(&mut World, Anchor),
    mut after: impl FnMut(&World),
) {
    let mut rng = SplitMix64::new(seed);
    let mut at = from;
    for _ in 0..steps {
        at = world.random_step(at, &mut rng);
        while let Some(anchor) = world.nearest_anchor(at) {
            grow(world, anchor);
        }
        after(world);
    }
}

/// The value at `q` (0 to 1) of `sorted`, nearest rank.
pub fn quantile<T: Copy>(sorted: &[T], q: f64) -> Option<T> {
    if sorted.is_empty() {
        return None;
    }
    let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len());
    Some(sorted[rank - 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Table;
    use crate::world::Settings;

    const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

    fn core() -> (TileSet, Table) {
        let set = TileSet::from_toml(CORE).expect("core loads");
        let table = Table::calculate(&set);
        (set, table)
    }

    fn variant(set: &TileSet, id: &str, orientation: u32) -> usize {
        set.variants
            .iter()
            .position(|v| set.tile_of(v).id == id && v.orientation == orientation)
            .unwrap_or_else(|| panic!("no {id} at {orientation}"))
    }

    #[test]
    fn edge_chance_matches_counting() {
        // A 1 × 1 box in 4 × 4: 12 of 16 places are on the ring.
        assert!((edge_chance(1, 1, 4) - 12.0 / 16.0).abs() < 1e-12);
        // 2 × 2 in 4 × 4: only the centre place of 9 is clear.
        assert!((edge_chance(2, 2, 4) - 8.0 / 9.0).abs() < 1e-12);
        assert_eq!(edge_chance(3, 3, 4), 1.0);
        assert_eq!(edge_chance(5, 1, 4), 1.0);
    }

    #[test]
    fn quantiles_use_nearest_rank() {
        let v = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(quantile(&v, 0.5), Some(5));
        assert_eq!(quantile(&v, 0.99), Some(10));
        assert_eq!(quantile(&v, 0.0), Some(1));
        assert_eq!(quantile::<u32>(&[], 0.5), None);
    }

    #[test]
    fn connectivity_counts_a_ring_and_a_dead_end() {
        let (set, _) = core();
        // A 2 × 2 ring of corners, with a dead end hanging off nothing:
        //   ┌┐
        //   └┘  and, apart, a lone straight between two dead ends.
        let corner = |o| variant(&set, "corridor_corner", o);
        // corridor_corner at 0 is open N and E; turn clockwise.
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), corner(1)); // E, S
        cells.insert((1, 0), corner(2)); // S, W
        cells.insert((1, 1), corner(3)); // W, N
        cells.insert((0, 1), corner(0)); // N, E
        let dead = |o| variant(&set, "corridor_dead_end", o);
        cells.insert((5, 0), dead(2)); // open S
        cells.insert((5, 1), variant(&set, "corridor_straight", 0));
        cells.insert((5, 2), dead(0)); // open N
        cells.insert((9, 9), variant(&set, "hedge", 0));
        let c = connectivity(&set, &cells, (0, 0), false);
        assert_eq!(c.walkable, 7);
        assert_eq!(c.areas, 2);
        assert_eq!(c.reachable, 4);
        assert_eq!(c.loops, 1);
        assert_eq!(c.dead_ends, 0);
        let c = connectivity(&set, &cells, (5, 1), false);
        assert_eq!((c.reachable, c.loops, c.dead_ends), (3, 0, 2));
        // From a hedge cell nothing is reachable.
        assert_eq!(connectivity(&set, &cells, (9, 9), false).reachable, 0);
    }

    #[test]
    fn a_dead_end_facing_the_unsettled_is_an_endpoint_not_a_dead_end() {
        let (set, _) = core();
        let mut cells = BTreeMap::new();
        cells.insert((0, 0), variant(&set, "corridor_straight", 0)); // N, S
        cells.insert((0, 1), variant(&set, "corridor_dead_end", 0)); // N
        let fixed = connectivity(&set, &cells, (0, 0), false);
        let grown = connectivity(&set, &cells, (0, 0), true);
        assert_eq!(fixed.dead_ends, 2);
        // In a grown world the straight still faces the unsettled north.
        assert_eq!(grown.dead_ends, 1);
    }

    #[test]
    fn grid_cells_skip_open_cells() {
        let cells = grid_cells(&[Some(3), None, Some(4), Some(5)], 2);
        assert_eq!(
            cells.into_iter().collect::<Vec<_>>(),
            vec![((0, 0), 3), ((0, 1), 4), ((1, 1), 5)]
        );
    }

    #[test]
    fn the_measured_walk_is_random_walk() {
        let (set, table) = core();
        let mut a = World::new(&set, &table, 5, Settings::default()).expect("loads");
        let mut b = a_world(&set, &table, 5);
        a.random_walk((0, 0), 77, 60);
        let mut steps = 0;
        walk(
            &mut b,
            (0, 0),
            77,
            60,
            |w, anchor| {
                w.grow_at(anchor);
            },
            |_| steps += 1,
        );
        assert_eq!(steps, 60);
        assert_eq!(a.log(), b.log());
    }

    fn a_world(set: &TileSet, table: &Table, seed: u32) -> World {
        World::new(set, table, seed, Settings::default()).expect("loads")
    }

    #[test]
    fn a_grown_world_is_one_area_with_an_outward_frontier() {
        let (set, table) = core();
        let mut world = a_world(&set, &table, 9);
        world.random_walk((0, 0), 3, 80);
        let cells: BTreeMap<Pos, usize> = world.cells().collect();
        let c = connectivity(&set, &cells, (0, 0), true);
        assert_eq!(c.areas, 1);
        assert_eq!(c.reachable, c.walkable);
        let f = frontier(&world);
        assert!(f.outward >= 1 && f.outward <= f.open);
        assert!(world.has_way_onward());
    }

    #[test]
    fn frontier_counts_sides_and_finds_enclosed_ones() {
        let (set, table) = core();
        let world = a_world(&set, &table, 9);
        // Count open sides independently of the flood.
        let cells: BTreeMap<Pos, usize> = world.cells().collect();
        let open: usize = cells
            .iter()
            .map(|(&p, &v)| {
                Direction::ALL
                    .into_iter()
                    .filter(|&d| {
                        walkable(set.variants[v].side(d)) && !cells.contains_key(&step(p, d))
                    })
                    .count()
            })
            .sum();
        assert_eq!(frontier(&world).open, open);
        assert!(frontier(&world).outward >= 1);
    }

    #[test]
    fn rooms_never_span_patches_and_count_their_cells() {
        let (set, table) = core();
        let mut world = a_world(&set, &table, 4);
        world.random_walk((0, 0), 8, 150);
        let rooms = rooms(&world);
        assert!(!rooms.is_empty(), "core.toml grows rooms");
        let room_cells = world
            .cells()
            .filter(|&(_, v)| set.tile_of(&set.variants[v]).kind == TileKind::Room)
            .count();
        assert_eq!(rooms.iter().map(|r| r.cells).sum::<usize>(), room_cells);
        for r in &rooms {
            assert!(!r.spans_patches);
            assert!(r.cells <= r.width * r.height);
            assert_eq!(r.touches_edge, r.edge_cells > 0);
        }
    }

    #[test]
    fn memory_grows_with_the_world() {
        let (set, table) = core();
        let mut world = a_world(&set, &table, 2);
        let before = world.memory();
        world.random_walk((0, 0), 1, 200);
        let after = world.memory();
        assert!(before.pages >= 1);
        assert!(after.pages >= before.pages);
        assert!(after.table >= after.pages * 2048);
        assert!(after.log > before.log);
    }
}
