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
    let cells: Vec<(Pos, usize)> = world.cells().collect();
    let Some(((lx, ly), (hx, hy))) = bounds(cells.iter().map(|(p, _)| p)) else {
        return Frontier::default();
    };
    // A dense grid over the box one bigger than the stored cells' box, so
    // its rim is all unsettled; flood unsettled cells inward from the rim.
    let (lx, ly) = (lx - 1, ly - 1);
    let (w, h) = ((hx + 2 - lx) as usize, (hy + 2 - ly) as usize);
    let at = |p: Pos| (p.1 - ly) as usize * w + (p.0 - lx) as usize;
    let mut stored = vec![false; w * h];
    for &(p, _) in &cells {
        stored[at(p)] = true;
    }
    let mut reached = vec![false; w * h];
    let mut stack: Vec<usize> = (0..w * h)
        .filter(|&i| i % w == 0 || i / w == 0 || i % w == w - 1 || i / w == h - 1)
        .collect();
    for &i in &stack {
        reached[i] = true;
    }
    while let Some(i) = stack.pop() {
        let (x, y) = (i % w, i / w);
        let next = [
            (y > 0).then(|| i - w),
            (x + 1 < w).then(|| i + 1),
            (y + 1 < h).then(|| i + w),
            (x > 0).then(|| i - 1),
        ];
        for j in next.into_iter().flatten() {
            if !stored[j] && !reached[j] {
                reached[j] = true;
                stack.push(j);
            }
        }
    }
    let mut out = Frontier::default();
    for &(p, v) in &cells {
        for d in Direction::ALL {
            let q = at(step(p, d));
            if walkable(set.variants[v].side(d)) && !stored[q] {
                out.open += 1;
                out.outward += usize::from(reached[q]);
            }
        }
    }
    out
}

/// Where stored cells lie in the window of the patch that stored them:
/// `[d]` counts cells `d` cells in from the window's outer ring (the ring
/// is 0; the last entry gathers everything further in), as (room cells,
/// all cells). Only the cells runs kept (`Patch::stored`): sealed hedge
/// (DR-0047) lies anywhere and isn't counted. If rooms avoid patch edges, room cells are rarer near the
/// ring than stored cells are in general.
pub fn edge_profile(world: &World) -> [(usize, usize); 4] {
    let set = world.tile_set();
    let s = world.settings().patch_size as i64;
    let mut out = [(0, 0); 4];
    for patch in world.log() {
        for &p in &patch.stored {
            let (x, y) = (p.0 - patch.window.0, p.1 - patch.window.1);
            let d = x.min(y).min(s - 1 - x).min(s - 1 - y).clamp(0, 3) as usize;
            let v = world.get(p).expect("logged cells are stored");
            out[d].0 += usize::from(set.tile_of(&set.variants[v]).kind == TileKind::Room);
            out[d].1 += 1;
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

/// An explorer's walk, standing in for a player who heads for the
/// unknown: each step the player jumps to the stored cell of an open
/// endpoint drawn from `seed` (any endpoint, outward or not), and the
/// world grows around it with `grow`; `after` sees the world after each
/// step. Grows far faster than `walk`, whose random walk mostly stays near
/// where it has been.
pub fn explore(
    world: &mut World,
    seed: u64,
    steps: usize,
    mut grow: impl FnMut(&mut World, Anchor),
    mut after: impl FnMut(&World),
) {
    let mut rng = SplitMix64::new(seed);
    for _ in 0..steps {
        let set = world.tile_set();
        let ends: Vec<Pos> = world
            .cells()
            .filter(|&(p, v)| {
                Direction::ALL
                    .into_iter()
                    .any(|d| walkable(set.variants[v].side(d)) && world.get(step(p, d)).is_none())
            })
            .map(|(p, _)| p)
            .collect();
        if ends.is_empty() {
            // Rule 6 says this can't happen; stop rather than spin.
            return;
        }
        let at = ends[rng.below(ends.len())];
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

    #[test]
    fn exploring_grows_and_keeps_the_guarantees_measured_here() {
        let (set, table) = core();
        let mut world = a_world(&set, &table, 6);
        let start = world.len();
        let mut steps = 0;
        explore(
            &mut world,
            11,
            20,
            |w, anchor| {
                w.grow_at(anchor);
            },
            |w| {
                steps += 1;
                assert!(frontier(w).outward >= 1);
            },
        );
        assert_eq!(steps, 20);
        assert!(world.len() > start);
        let cells: BTreeMap<Pos, usize> = world.cells().collect();
        assert_eq!(connectivity(&set, &cells, (0, 0), true).areas, 1);
    }

    #[test]
    fn edge_profile_counts_every_stored_cell_once() {
        let (set, table) = core();
        let mut world = a_world(&set, &table, 4);
        world.random_walk((0, 0), 8, 150);
        let profile = edge_profile(&world);
        // Sealed hedge (DR-0047) isn't placed by a run, so isn't counted.
        let sealed: usize = world.log().iter().map(|p| p.sealed.len()).sum();
        assert_eq!(
            profile.iter().map(|x| x.1).sum::<usize>(),
            world.len() - sealed
        );
        let room_cells: usize = rooms(&world).iter().map(|r| r.cells).sum();
        assert_eq!(profile.iter().map(|x| x.0).sum::<usize>(), room_cells);
        // Every patch but the first has its anchor on its window's ring.
        assert!(profile[0].1 >= world.log().len() - 1);
    }
}
