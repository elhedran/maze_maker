//! The endless world: stored (DR-0035) and grown in patches from open
//! endpoints near the player (DR-0036).
//!
//! Terms (DR-0036): a *stored* cell is in the world's table, an *unsettled*
//! one isn't. A side is *walkable* unless it is hedge, and two cells are
//! *joined* when both facing sides are walkable. An *open endpoint* is a
//! stored cell's walkable side facing an unsettled cell, its *anchor*. It
//! is *outward* when the anchor connects through unsettled cells to the
//! unbounded unsettled region.
//!
//! A patch is one WFC run over a P × P window beyond its anchor, with
//! stored cells pinned and the window's edge kept to hedge, passage or door
//! where it faces unsettled cells. Only the new cells joined to the anchor
//! are stored. The attempt order (DR-0036 rule 8), with one counter across
//! every run that goes into its seed:
//!
//! 1. tries 0 to R, full tile set, each with up to 20 contradiction
//!    restarts; the first run that completes and leaves an outward endpoint
//!    is the patch;
//! 2. the spine (a path pinned open from the anchor out of the window)
//!    with the full tile set, up to 20 restarts;
//! 3. the fallback: the spine, fallback tiles only, one run, which can't
//!    contradict because the tile set passed `completeness::check`.
//!
//! **Keep (Rowan's reading of rules 4 and 6).** A patch keeps the new cells
//! joined to its own anchor, not every new cell joined to some stored cell.
//! Rule 1 holds either way (a kept cell faces a discarded one with hedge),
//! and other endpoints in the window stay open for their own patches. It
//! makes the spine's guarantee provable: if the anchor is outward, its way
//! out leaves its own pocket of the window's unsettled cells for the last
//! time at some edge, so a spine inside that pocket exists, and nothing
//! the patch stores can block what lies beyond. If the anchor isn't
//! outward, everything kept lies in a bounded pocket and no outward
//! endpoint can be lost.

use crate::completeness::{self, is_fallback};
use crate::measures::connected;
use crate::side::{Direction, SideKind};
use crate::table::Table;
use crate::tile_set::{LoadError, TileSet};
use crate::wfc::{MAX_RESTARTS, MAX_SIZE, Solver, SplitMix64, Start};
use std::collections::{BTreeMap, VecDeque};

/// P, the patch window's width and height (DR-0036 rule 3).
pub const PATCH_SIZE: usize = 16;
/// r, how far from the player (Chebyshev) an anchor makes the world grow
/// (DR-0036 rule 2). Must be more than the player can see.
pub const RADIUS: i64 = 8;
/// R: tries after the first before a spine is pinned (DR-0036 rule 8).
pub const CHECK_RETRIES: u32 = 3;
/// Cells per side of a page of the sparse table (DR-0035's proposal).
const PAGE: i64 = 32;
const EMPTY: u16 = u16::MAX;

/// A world cell, (x, y): x grows east, y grows south.
pub type Pos = (i64, i64);

/// The cell one step from `p` in direction `d`.
pub fn step(p: Pos, d: Direction) -> Pos {
    let (x, y) = p;
    match d {
        Direction::North => (x, y - 1),
        Direction::East => (x + 1, y),
        Direction::South => (x, y + 1),
        Direction::West => (x - 1, y),
    }
}

/// Where in the attempt order a patch starts. Always `Checked` for the
/// world; the others force the spine or the fallback in tests and step 2's
/// measures, and give a different world.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Checked,
    Spine,
    Fallback,
}

/// The growth settings. Changing any of them changes new growth only;
/// stored cells stay as they are.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// P, 1 to 64.
    pub patch_size: usize,
    /// r, 1 or more.
    pub radius: i64,
    /// R.
    pub retries: u32,
    pub first_stage: Stage,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            patch_size: PATCH_SIZE,
            radius: RADIUS,
            retries: CHECK_RETRIES,
            first_stage: Stage::Checked,
        }
    }
}

/// One growth step: the anchor, and the direction its endpoint faces (from
/// the stored cell to the anchor). `None` only for a new world's first
/// patch, at the origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Anchor {
    pub cell: Pos,
    pub from: Option<Direction>,
}

/// What one patch did: an entry in the growth log (DR-0035).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patch {
    pub anchor: Anchor,
    /// The window's north-west cell; it is `patch_size` square.
    pub window: Pos,
    /// WFC runs made, counting the one that made the patch: 1 for a clean
    /// first run.
    pub attempts: u32,
    /// A spine was pinned (stages 2 and 3).
    pub spine: bool,
    /// The fallback run made the patch (stage 3).
    pub fallback: bool,
    /// The cells it stored, in the window's row-major order.
    pub stored: Vec<Pos>,
    /// The spine's cells from the anchor out; empty without a spine.
    pub spine_cells: Vec<Pos>,
}

/// The sparse table (DR-0035): square pages of palette indices in a sorted
/// map, so every walk over it is in key order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Cells {
    /// Keyed (page y, page x); each page row-major, a palette index or
    /// `EMPTY` where unsettled.
    pages: BTreeMap<Pos, Vec<u16>>,
    count: usize,
    /// The stored cells' bounding box, (min, max), once anything is stored.
    bounds: Option<(Pos, Pos)>,
}

impl Cells {
    fn split(p: Pos) -> (Pos, usize) {
        let (x, y) = p;
        let key = (y.div_euclid(PAGE), x.div_euclid(PAGE));
        let at = y.rem_euclid(PAGE) * PAGE + x.rem_euclid(PAGE);
        (key, at as usize)
    }

    fn get(&self, p: Pos) -> Option<usize> {
        let (key, at) = Cells::split(p);
        let v = *self.pages.get(&key)?.get(at)?;
        (v != EMPTY).then_some(usize::from(v))
    }

    /// Stores `v` at `p`. Panics if `p` is already stored: stored cells
    /// never change (DR-0036 rule 5).
    fn store(&mut self, p: Pos, v: usize) {
        let (key, at) = Cells::split(p);
        let page = self
            .pages
            .entry(key)
            .or_insert_with(|| vec![EMPTY; (PAGE * PAGE) as usize]);
        assert_eq!(page[at], EMPTY, "stored cell {p:?} would change");
        page[at] = u16::try_from(v).expect("palette index fits a page");
        self.count += 1;
        self.bounds = Some(match self.bounds {
            None => (p, p),
            Some((lo, hi)) => (
                (lo.0.min(p.0), lo.1.min(p.1)),
                (hi.0.max(p.0), hi.1.max(p.1)),
            ),
        });
    }

    fn iter(&self) -> impl Iterator<Item = (Pos, usize)> + '_ {
        self.pages.iter().flat_map(|(&(py, px), page)| {
            page.iter()
                .enumerate()
                .filter(|(_, v)| **v != EMPTY)
                .map(move |(i, &v)| {
                    let (lx, ly) = (i as i64 % PAGE, i as i64 / PAGE);
                    ((px * PAGE + lx, py * PAGE + ly), usize::from(v))
                })
        })
    }
}

const TAG_RUN: u64 = 1;
const TAG_SPINE: u64 = 2;

/// Mixes `words` into one hash, each through SplitMix64's step (add the
/// golden gamma, then the finaliser), as DR-0036 rule 8 says. Never `std`'s
/// hashers or `rand`, whose values aren't promised stable.
pub fn hash(words: &[u64]) -> u64 {
    words
        .iter()
        .fold(0, |h, &w| SplitMix64::new(h ^ w).next_u64())
}

fn walkable(side: SideKind) -> bool {
    side != SideKind::Hedge
}

fn open_side(side: SideKind) -> bool {
    matches!(side, SideKind::Passage | SideKind::Door)
}

/// One patch's window, in world and local coordinates.
struct Window {
    nw: Pos,
    size: usize,
}

impl Window {
    fn pos(&self, i: usize) -> Pos {
        let s = self.size;
        (self.nw.0 + (i % s) as i64, self.nw.1 + (i / s) as i64)
    }

    fn index(&self, p: Pos) -> Option<usize> {
        let s = self.size as i64;
        let (x, y) = (p.0 - self.nw.0, p.1 - self.nw.1);
        ((0..s).contains(&x) && (0..s).contains(&y)).then(|| (y * s + x) as usize)
    }

    fn corners(&self) -> (Pos, Pos) {
        let s = self.size as i64 - 1;
        (self.nw, (self.nw.0 + s, self.nw.1 + s))
    }
}

/// A box of cells, (min, max) inclusive, for the floods.
struct Area {
    lo: Pos,
    width: usize,
    height: usize,
}

impl Area {
    /// The box around `boxes`, one cell bigger on every side, so its rim is
    /// all unsettled and joined to the unbounded region.
    fn around(boxes: &[(Pos, Pos)]) -> Area {
        let lo = boxes.iter().fold((i64::MAX, i64::MAX), |a, (l, _)| {
            (a.0.min(l.0), a.1.min(l.1))
        });
        let hi = boxes.iter().fold((i64::MIN, i64::MIN), |a, (_, h)| {
            (a.0.max(h.0), a.1.max(h.1))
        });
        let lo = (lo.0 - 1, lo.1 - 1);
        Area {
            lo,
            width: (hi.0 + 1 - lo.0 + 1) as usize,
            height: (hi.1 + 1 - lo.1 + 1) as usize,
        }
    }

    fn index(&self, p: Pos) -> Option<usize> {
        let (x, y) = (p.0 - self.lo.0, p.1 - self.lo.1);
        let inside = (0..self.width as i64).contains(&x) && (0..self.height as i64).contains(&y);
        inside.then(|| y as usize * self.width + x as usize)
    }

    fn pos(&self, i: usize) -> Pos {
        (
            self.lo.0 + (i % self.width) as i64,
            self.lo.1 + (i / self.width) as i64,
        )
    }

    /// Floods from the rim over the cells `open` allows; `reached[i]`.
    fn flood_from_rim(&self, open: impl Fn(Pos) -> bool) -> Vec<bool> {
        let mut reached = vec![false; self.width * self.height];
        let mut stack: Vec<usize> = (0..reached.len())
            .filter(|&i| {
                let (x, y) = (i % self.width, i / self.width);
                x == 0 || y == 0 || x + 1 == self.width || y + 1 == self.height
            })
            .collect();
        for &i in &stack {
            reached[i] = true;
        }
        while let Some(i) = stack.pop() {
            for d in Direction::ALL {
                let Some(j) = self.index(step(self.pos(i), d)) else {
                    continue;
                };
                if !reached[j] && open(self.pos(j)) {
                    reached[j] = true;
                    stack.push(j);
                }
            }
        }
        reached
    }
}

/// The endless world for one tile set and world seed.
pub struct World {
    set: TileSet,
    solver: Solver,
    /// Per variant: may the fallback run use it?
    fallback: Vec<bool>,
    settings: Settings,
    seed: u32,
    /// DR-0035: cells hold an index into this palette of (tile id,
    /// orientation), never a table index, since table order shifts when the
    /// tile set changes. Entries are added as tiles are first stored.
    palette: Vec<(String, u32)>,
    /// Per palette entry: its variant in `set`.
    palette_variant: Vec<usize>,
    cells: Cells,
    log: Vec<Patch>,
}

impl World {
    /// An empty world. Refuses a set that fails the completeness check
    /// (DR-0036 rule 7), since the fallback's guarantee rests on it; a set
    /// loaded with `TileSet::from_toml` has passed. Panics on settings out
    /// of range.
    fn empty(
        set: &TileSet,
        table: &Table,
        seed: u32,
        settings: Settings,
    ) -> Result<World, Vec<LoadError>> {
        assert!(
            (1..=MAX_SIZE).contains(&settings.patch_size),
            "patch size must be 1 to {MAX_SIZE}"
        );
        assert!(settings.radius >= 1, "radius must be 1 or more");
        let errors = completeness::check(set, table);
        if !errors.is_empty() {
            return Err(errors);
        }
        Ok(World {
            set: set.clone(),
            solver: Solver::new(set, table),
            fallback: set.variants.iter().map(|v| is_fallback(set, v)).collect(),
            settings,
            seed,
            palette: Vec::new(),
            palette_variant: Vec::new(),
            cells: Cells::default(),
            log: Vec::new(),
        })
    }

    /// A new world: one patch at the origin, where the player spawns, then
    /// growth around it (DR-0036 rule 2).
    pub fn new(
        set: &TileSet,
        table: &Table,
        seed: u32,
        settings: Settings,
    ) -> Result<World, Vec<LoadError>> {
        let mut world = World::empty(set, table, seed, settings)?;
        world.grow_at(Anchor {
            cell: (0, 0),
            from: None,
        });
        world.grow_around((0, 0));
        Ok(world)
    }

    /// The world a growth log made: each anchor's patch, in order
    /// (DR-0036 rule 10). Panics if an entry isn't an anchor of the world
    /// before it, as happens if the log came from a different tile set.
    pub fn replay(
        set: &TileSet,
        table: &Table,
        seed: u32,
        settings: Settings,
        anchors: &[Anchor],
    ) -> Result<World, Vec<LoadError>> {
        let mut world = World::empty(set, table, seed, settings)?;
        for &anchor in anchors {
            world.grow_at(anchor);
        }
        Ok(world)
    }

    pub fn seed(&self) -> u32 {
        self.seed
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn tile_set(&self) -> &TileSet {
        &self.set
    }

    /// The stored tile at `p`, as a variant index into the world's tile
    /// set, or `None` if unsettled.
    pub fn get(&self, p: Pos) -> Option<usize> {
        self.cells.get(p).map(|k| self.palette_variant[k])
    }

    /// The palette stored cells index into (DR-0035), in first-stored order.
    pub fn palette(&self) -> &[(String, u32)] {
        &self.palette
    }

    fn store(&mut self, p: Pos, v: usize) {
        let k = match self.palette_variant.iter().position(|&x| x == v) {
            Some(k) => k,
            None => {
                let variant = &self.set.variants[v];
                let id = self.set.tile_of(variant).id.clone();
                self.palette.push((id, variant.orientation));
                self.palette_variant.push(v);
                self.palette.len() - 1
            }
        };
        self.cells.store(p, k);
    }

    /// How many cells are stored.
    pub fn len(&self) -> usize {
        self.cells.count
    }

    pub fn is_empty(&self) -> bool {
        self.cells.count == 0
    }

    /// Every stored cell, in key order.
    pub fn cells(&self) -> impl Iterator<Item = (Pos, usize)> + '_ {
        self.cells.iter().map(|(p, k)| (p, self.palette_variant[k]))
    }

    /// The growth log, oldest first.
    pub fn log(&self) -> &[Patch] {
        &self.log
    }

    /// The side of stored `p` facing `d`, if `p` is stored.
    fn side(&self, p: Pos, d: Direction) -> Option<SideKind> {
        self.get(p).map(|v| self.set.variants[v].side(d))
    }

    /// `cell` as an anchor: the first direction, N, E, S, W from it, in
    /// which a stored cell shows a walkable side to it.
    fn anchor_at(&self, cell: Pos) -> Option<Anchor> {
        if self.get(cell).is_some() {
            return None;
        }
        Direction::ALL.into_iter().find_map(|d| {
            let side = self.side(step(cell, d), d.opposite())?;
            walkable(side).then_some(Anchor {
                cell,
                from: Some(d.opposite()),
            })
        })
    }

    /// The anchor nearest `player` within r: Chebyshev distance, ties in
    /// (y, x) order (DR-0036 rule 2).
    pub fn nearest_anchor(&self, player: Pos) -> Option<Anchor> {
        let r = self.settings.radius;
        let mut best: Option<(i64, Pos, Anchor)> = None;
        for dy in -r..=r {
            for dx in -r..=r {
                let cell = (player.0 + dx, player.1 + dy);
                let Some(anchor) = self.anchor_at(cell) else {
                    continue;
                };
                let key = (dx.abs().max(dy.abs()), (cell.1, cell.0));
                if best.is_none_or(|(d, at, _)| key < (d, at)) {
                    best = Some((key.0, key.1, anchor));
                }
            }
        }
        best.map(|(_, _, a)| a)
    }

    /// Grows patches at the nearest anchor until none is within r of the
    /// player (DR-0036 rule 2). Each patch stores at least its anchor, so
    /// this ends.
    pub fn grow_around(&mut self, player: Pos) {
        while let Some(anchor) = self.nearest_anchor(player) {
            self.grow_at(anchor);
        }
    }

    /// Walks `path`, growing around each cell of it.
    pub fn walk(&mut self, path: &[Pos]) {
        for &p in path {
            self.grow_around(p);
        }
    }

    /// A seeded random walk of `steps` steps from `from` over joined stored
    /// cells, growing as it goes. Returns the cells walked, after `from`.
    pub fn random_walk(&mut self, from: Pos, seed: u64, steps: usize) -> Vec<Pos> {
        let mut rng = SplitMix64::new(seed);
        let mut at = from;
        let mut path = Vec::with_capacity(steps);
        for _ in 0..steps {
            let here = self.get(at).expect("the walker stands on a stored cell");
            let ways: Vec<Pos> = Direction::ALL
                .into_iter()
                .filter_map(|d| {
                    let next = step(at, d);
                    let there = self.get(next)?;
                    connected(&self.set, here, d, there).then_some(next)
                })
                .collect();
            if !ways.is_empty() {
                at = ways[rng.below(ways.len())];
            }
            path.push(at);
            self.grow_around(at);
        }
        path
    }

    /// Grows one patch at `anchor` and stores it. Panics if `anchor` isn't
    /// an anchor of this world (or, for `from: None`, the world isn't
    /// empty).
    pub fn grow_at(&mut self, anchor: Anchor) -> &Patch {
        match anchor.from {
            None => assert!(self.is_empty(), "only a new world grows from no endpoint"),
            Some(_) => assert_eq!(
                self.anchor_at(anchor.cell),
                Some(anchor),
                "not an anchor of this world"
            ),
        }
        let (patch, tiles) = self.patch(anchor);
        for (&p, &v) in patch.stored.iter().zip(&tiles) {
            self.store(p, v);
        }
        self.log.push(patch);
        self.log.last().expect("just pushed")
    }

    fn window(&self, anchor: Anchor) -> Window {
        let size = self.settings.patch_size;
        let (p, half) = (size as i64, size as i64 / 2);
        let (x, y) = anchor.cell;
        let nw = match anchor.from {
            None => (x - half, y - half),
            Some(Direction::East) => (x, y - half),
            Some(Direction::West) => (x - p + 1, y - half),
            Some(Direction::South) => (x - half, y),
            Some(Direction::North) => (x - half, y - p + 1),
        };
        Window { nw, size }
    }

    /// The patch at `anchor` against the world as it is, without storing
    /// it: the log entry and the stored cells' tiles, in step. A pure
    /// function of the world, the anchor, the tile set and the code.
    pub fn patch(&self, anchor: Anchor) -> (Patch, Vec<usize>) {
        let window = self.window(anchor);
        let (s, n) = (window.size, self.set.variants.len());
        let cells = s * s;
        let a = window
            .index(anchor.cell)
            .expect("the anchor is in its window");
        let variants = &self.set.variants;

        let mut start = Start::open(cells, n);
        for i in 0..cells {
            let p = window.pos(i);
            start.settled[i] = self.get(p);
            if start.settled[i].is_some() {
                continue;
            }
            for d in Direction::ALL {
                let q = step(p, d);
                if window.index(q).is_some() {
                    continue;
                }
                match self.get(q) {
                    Some(t) => start.outside[i * 4 + d.index()] = Some(t),
                    // DR-0036 rule 1: never floor towards the unsettled.
                    None => {
                        for (v, variant) in variants.iter().enumerate() {
                            let side = variant.side(d);
                            start.mask[i * n + v] &= side == SideKind::Hedge || open_side(side);
                        }
                    }
                }
            }
        }
        if anchor.from.is_none() {
            for (v, variant) in variants.iter().enumerate() {
                start.mask[a * n + v] &= variant.sides.iter().any(|&side| walkable(side));
            }
        }

        let mut attempt = 0u32;
        let (ax, ay) = anchor.cell;
        let mut run = |start: &Start| {
            let seed = hash(&[
                TAG_RUN,
                u64::from(self.seed),
                ax as u64,
                ay as u64,
                u64::from(attempt),
            ]);
            attempt += 1;
            self.solver.run(s, s, start, seed)
        };
        let done = |tiles: Vec<usize>, attempts, spine: &[usize], fallback| {
            let kept = self.keep(&window, &start.settled, &tiles, a);
            let patch = Patch {
                anchor,
                window: window.nw,
                attempts,
                spine: !spine.is_empty(),
                fallback,
                stored: (0..cells)
                    .filter(|&i| kept[i])
                    .map(|i| window.pos(i))
                    .collect(),
                spine_cells: spine.iter().map(|&i| window.pos(i)).collect(),
            };
            let tiles = (0..cells).filter(|&i| kept[i]).map(|i| tiles[i]).collect();
            (patch, tiles)
        };

        if self.settings.first_stage == Stage::Checked {
            'tries: for _ in 0..=self.settings.retries {
                for _ in 0..=MAX_RESTARTS {
                    let Some(tiles) = run(&start) else {
                        continue;
                    };
                    let kept = self.keep(&window, &start.settled, &tiles, a);
                    if self.has_outward(&window, &kept, &tiles) {
                        return done(tiles, attempt, &[], false);
                    }
                    continue 'tries;
                }
            }
        }

        // The spine. With no route (the anchor isn't outward), nothing the
        // patch stores can close the frontier, so the stages run unpinned.
        let mut spined = start.clone();
        let spine = match self.spine(anchor, &window) {
            Some((path, exit)) => {
                for pair in path.windows(2) {
                    let d = Direction::ALL
                        .into_iter()
                        .find(|&d| window.index(step(window.pos(pair[0]), d)) == Some(pair[1]))
                        .expect("spine cells are neighbours");
                    pin_open(&mut spined.mask, n, variants, pair[0], d);
                    pin_open(&mut spined.mask, n, variants, pair[1], d.opposite());
                }
                pin_open(
                    &mut spined.mask,
                    n,
                    variants,
                    *path.last().expect("not empty"),
                    exit,
                );
                path
            }
            None => Vec::new(),
        };
        if self.settings.first_stage <= Stage::Spine {
            for _ in 0..=MAX_RESTARTS {
                if let Some(tiles) = run(&spined) {
                    return done(tiles, attempt, &spine, false);
                }
            }
        }

        let mut fallback = spined;
        for i in 0..cells {
            if fallback.settled[i].is_none() {
                for v in 0..n {
                    fallback.mask[i * n + v] &= self.fallback[v];
                }
            }
        }
        let tiles = run(&fallback).unwrap_or_else(|| {
            panic!(
                "the fallback run at {:?} contradicted, which the completeness check (DR-0036 rule 7) should make impossible",
                anchor.cell
            )
        });
        done(tiles, attempt, &spine, true)
    }

    /// The new cells joined to the anchor through walkable sides (DR-0036
    /// rule 4, Rowan's reading in the module notes).
    fn keep(
        &self,
        window: &Window,
        settled: &[Option<usize>],
        tiles: &[usize],
        a: usize,
    ) -> Vec<bool> {
        let s = window.size;
        let mut kept = vec![false; s * s];
        kept[a] = true;
        let mut stack = vec![a];
        while let Some(i) = stack.pop() {
            for d in Direction::ALL {
                let Some(j) = window.index(step(window.pos(i), d)) else {
                    continue;
                };
                if !kept[j] && settled[j].is_none() && connected(&self.set, tiles[i], d, tiles[j]) {
                    kept[j] = true;
                    stack.push(j);
                }
            }
        }
        kept
    }

    /// DR-0036 rule 6's reference check: with the kept cells stored, some
    /// stored cell shows a walkable side to an unsettled cell joined to the
    /// unbounded unsettled region.
    fn has_outward(&self, window: &Window, kept: &[bool], tiles: &[usize]) -> bool {
        let new = |p: Pos| window.index(p).filter(|&i| kept[i]).map(|i| tiles[i]);
        let tile = |p: Pos| self.get(p).or_else(|| new(p));
        let mut boxes = vec![window.corners()];
        boxes.extend(self.cells.bounds);
        let area = Area::around(&boxes);
        let reached = area.flood_from_rim(|p| tile(p).is_none());
        (0..reached.len()).filter(|&i| reached[i]).any(|i| {
            let u = area.pos(i);
            Direction::ALL.into_iter().any(|d| {
                tile(step(u, d)).is_some_and(|v| walkable(self.set.variants[v].side(d.opposite())))
            })
        })
    }

    /// The spine (DR-0036 rule 6): local cells from the anchor through its
    /// pocket of the window's unsettled cells to an edge cell, and the
    /// direction out of the window from it, to an unsettled cell joined to
    /// the unbounded region without crossing that pocket. The route is a
    /// breadth-first search with the direction order shuffled from the
    /// patch's seed. `None` when there is no route: the anchor isn't
    /// outward.
    fn spine(&self, anchor: Anchor, window: &Window) -> Option<(Vec<usize>, Direction)> {
        let s = window.size;
        let a = window
            .index(anchor.cell)
            .expect("the anchor is in its window");
        // The anchor's pocket.
        let mut pocket = vec![false; s * s];
        pocket[a] = true;
        let mut stack = vec![a];
        while let Some(i) = stack.pop() {
            for d in Direction::ALL {
                let Some(j) = window.index(step(window.pos(i), d)) else {
                    continue;
                };
                if !pocket[j] && self.get(window.pos(j)).is_none() {
                    pocket[j] = true;
                    stack.push(j);
                }
            }
        }
        let mut boxes = vec![window.corners()];
        boxes.extend(self.cells.bounds);
        let area = Area::around(&boxes);
        let in_pocket = |p: Pos| window.index(p).is_some_and(|i| pocket[i]);
        let free = area.flood_from_rim(|p| self.get(p).is_none() && !in_pocket(p));
        let free = |p: Pos| area.index(p).is_none_or(|i| free[i]);

        let (ax, ay) = anchor.cell;
        let mut rng = SplitMix64::new(hash(&[
            TAG_SPINE,
            u64::from(self.seed),
            ax as u64,
            ay as u64,
        ]));
        let mut parent: Vec<Option<usize>> = vec![None; s * s];
        let mut seen = vec![false; s * s];
        seen[a] = true;
        let mut queue = VecDeque::from([a]);
        while let Some(i) = queue.pop_front() {
            let mut order = Direction::ALL;
            for k in (1..4).rev() {
                order.swap(k, rng.below(k + 1));
            }
            for d in order {
                let q = step(window.pos(i), d);
                match window.index(q) {
                    None if self.get(q).is_none() && free(q) => {
                        let mut path = vec![i];
                        while let Some(p) = parent[*path.last().expect("not empty")] {
                            path.push(p);
                        }
                        path.reverse();
                        return Some((path, d));
                    }
                    Some(j) if pocket[j] && !seen[j] => {
                        seen[j] = true;
                        parent[j] = Some(i);
                        queue.push_back(j);
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

/// Keeps cell `i` to variants showing passage or door towards `d`.
fn pin_open(
    mask: &mut [bool],
    n: usize,
    variants: &[crate::tile_set::Variant],
    i: usize,
    d: Direction,
) {
    for (v, variant) in variants.iter().enumerate() {
        mask[i * n + v] &= open_side(variant.side(d));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_pinned() {
        // If this fails, every seed's growth changed.
        assert_eq!(hash(&[]), 0);
        assert_eq!(hash(&[1]), SplitMix64::new(1).next_u64());
        assert_ne!(hash(&[1, 2]), hash(&[2, 1]));
    }

    #[test]
    fn pages_split_negative_coordinates() {
        let mut cells = Cells::default();
        for (i, p) in [(0, 0), (-1, -1), (31, 32), (-33, 5)]
            .into_iter()
            .enumerate()
        {
            cells.store(p, i);
        }
        assert_eq!(cells.get((-1, -1)), Some(1));
        assert_eq!(cells.get((31, 32)), Some(2));
        assert_eq!(cells.get((-33, 5)), Some(3));
        assert_eq!(cells.get((1, 0)), None);
        assert_eq!(cells.count, 4);
        assert_eq!(cells.bounds, Some(((-33, -1), (31, 32))));
        let mut all: Vec<(Pos, usize)> = cells.iter().collect();
        all.sort();
        assert_eq!(
            all,
            vec![((-33, 5), 3), ((-1, -1), 1), ((0, 0), 0), ((31, 32), 2)]
        );
    }

    #[test]
    #[should_panic(expected = "would change")]
    fn a_stored_cell_never_changes() {
        let mut cells = Cells::default();
        cells.store((2, 3), 1);
        cells.store((2, 3), 2);
    }
}
