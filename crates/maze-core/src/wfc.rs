//! Wave Function Collapse over the adjacency table (DR-0007, DR-0028).
//!
//! Each cell holds the variants (table order) it could still be. A candidate
//! survives only while every in-grid neighbour has some candidate it pairs
//! with at weight above 0. We repeatedly take the cell with the fewest
//! candidates (seeded tie-break), draw one candidate weighted by
//!
//! > score(c) = weight of c × Π over settled neighbours of
//! > (side-pair base × exception multiplier × noise factor)
//!
//! and propagate. A cell with no candidates left ends the attempt; the whole
//! maze restarts with the next attempt's seed, up to `MAX_RESTARTS` times.
//!
//! Determinism (DR-0028 rule 5): a hand-written SplitMix64, and every loop
//! runs in a fixed order (row-major cells, table-order candidates, FIFO
//! propagation). No hash map decides anything.

use crate::side::Direction;
use crate::table::Table;
use crate::tile_set::TileSet;
use std::collections::VecDeque;
use std::fmt;

/// Restarts after the first attempt before giving up.
pub const MAX_RESTARTS: u32 = 20;
/// Smallest and largest width or height `generate` accepts.
pub const MIN_SIZE: usize = 1;
pub const MAX_SIZE: usize = 64;
/// The first sample maze's size: one screen at about 40 px a cell.
pub const DEFAULT_WIDTH: usize = 24;
pub const DEFAULT_HEIGHT: usize = 16;

/// SplitMix64 (Steele, Lea and Flood, 2014). Written out so a seed means
/// the same maze on any toolchain; the `rand` crate promises no stable
/// value streams.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// The top 53 bits as a fraction in [0, 1): exact in an f64.
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `0..n` (multiply-high; the bias is below 2^-50 for any
    /// `n` we use). Panics if `n` is 0.
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "below(0)");
        ((u128::from(self.next_u64()) * n as u128) >> 64) as usize
    }
}

/// The seed of attempt `k` (0 for the first) of a maze with `seed`.
pub fn attempt_seed(seed: u32, attempt: u32) -> u64 {
    (u64::from(seed) << 32) | u64::from(attempt)
}

/// A generated maze.
#[derive(Clone, Debug, PartialEq)]
pub struct Generation {
    pub width: usize,
    pub height: usize,
    /// Row-major from the north-west; each a variant index (table order).
    /// `None` only in a failed maze, where the last attempt left it open.
    pub cells: Vec<Option<usize>>,
    /// 0 for a clean first attempt.
    pub restarts: u32,
    /// True when every attempt hit a contradiction.
    pub failed: bool,
}

/// Why `generate` refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GenerateError {
    BadSize { width: usize, height: usize },
}

impl fmt::Display for GenerateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerateError::BadSize { width, height } => write!(
                f,
                "A maze of {width} × {height} can't be made; width and height must each be from {MIN_SIZE} to {MAX_SIZE}."
            ),
        }
    }
}

impl std::error::Error for GenerateError {}

/// One collapse as the solver made it; borrowed from its own working
/// (DR-0034 rule 5).
pub struct CollapseStep<'a> {
    pub cell: usize,
    /// Neighbours already collapsed, N, E, S, W order: (direction from
    /// `cell`, variant).
    pub settled: &'a [(Direction, usize)],
    /// Candidates left after propagation, table order, with the score each
    /// was drawn by.
    pub scores: &'a [(usize, f64)],
    /// The sum of `scores`, in table order, as the draw used it.
    pub total: f64,
    pub chosen: usize,
}

/// Watches generation without being able to change it: it only ever gets
/// shared borrows, never the wave or the PRNG (DR-0034 rule 5).
pub trait Observer {
    /// Attempt `attempt` (0 for the first) is about to start from scratch.
    fn attempt_started(&mut self, attempt: u32);
    /// A cell was just collapsed. Called once per collapse, after the draw.
    fn collapsed(&mut self, step: &CollapseStep<'_>);
}

/// Watches nothing: what `generate` uses.
impl Observer for () {
    fn attempt_started(&mut self, _attempt: u32) {}
    fn collapsed(&mut self, _step: &CollapseStep<'_>) {}
}

/// The neighbour of cell `i` in direction `d`, if it is inside the grid.
/// Past the edge nothing is constrained (DR-0028 rule 6).
pub fn neighbour(width: usize, height: usize, i: usize, d: Direction) -> Option<usize> {
    let (x, y) = (i % width, i / width);
    match d {
        Direction::North if y > 0 => Some(i - width),
        Direction::South if y + 1 < height => Some(i + width),
        Direction::West if x > 0 => Some(i - 1),
        Direction::East if x + 1 < width => Some(i + 1),
        _ => None,
    }
}

/// What the attempts share: the table read once into the shapes WFC needs.
struct Rules {
    size: usize,
    /// Per variant: the tile's weight.
    weight: Vec<f64>,
    /// `pair[(d * size + c) * size + n]`: base × exception multiplier ×
    /// noise for `n` lying `d` of `c`; 0 where the sides may not meet.
    pair: Vec<f64>,
    /// `supported[(d * size + c) * size + n]`: `n` may lie `d` of `c`,
    /// that is the entry's weight is above 0 (DR-0028 rule 1).
    supported: Vec<bool>,
}

impl Rules {
    fn new(set: &TileSet, table: &Table) -> Rules {
        let size = table.size();
        let weight = set.variants.iter().map(|v| set.tile_of(v).weight).collect();
        let mut pair = Vec::with_capacity(4 * size * size);
        let mut supported = Vec::with_capacity(4 * size * size);
        for d in Direction::ALL {
            for c in 0..size {
                for n in 0..size {
                    let e = table.entry(d, c, n);
                    let multiplier = e.exception.map_or(1.0, |i| set.exceptions[i].multiplier);
                    // Read from the parts, not weight / (row × column), so a
                    // zero tile weight can't divide.
                    pair.push(if e.allowed {
                        e.base * multiplier * e.noise
                    } else {
                        0.0
                    });
                    supported.push(e.weight > 0.0);
                }
            }
        }
        Rules {
            size,
            weight,
            pair,
            supported,
        }
    }

    fn at(&self, d: Direction, c: usize, n: usize) -> usize {
        (d.index() * self.size + c) * self.size + n
    }
}

/// One attempt's state.
struct Wave<'r> {
    rules: &'r Rules,
    width: usize,
    height: usize,
    /// `possible[cell * size + variant]`.
    possible: Vec<bool>,
    count: Vec<usize>,
    collapsed: Vec<Option<usize>>,
}

/// A cell ran out of candidates.
struct Contradiction;

impl<'r> Wave<'r> {
    fn new(rules: &'r Rules, width: usize, height: usize) -> Wave<'r> {
        let cells = width * height;
        // A zero-weight tile can never be drawn, so it is never a candidate
        // (DR-0028 rule 1; Rowan's note on the record).
        let row: Vec<bool> = rules.weight.iter().map(|&w| w > 0.0).collect();
        let live = row.iter().filter(|&&p| p).count();
        Wave {
            rules,
            width,
            height,
            possible: row.repeat(cells),
            count: vec![live; cells],
            collapsed: vec![None; cells],
        }
    }

    /// As `new`, but each cell starts from `mask[cell * size + variant]`,
    /// still without zero-weight tiles: a chunk's border labels, pinned
    /// spine and fallback tiles (DR-0031 rule 3). `run` fails on an empty
    /// cell and propagates every cell first, so the masks spread with no
    /// other change. Panics if `mask` doesn't fit the grid.
    fn with_mask(rules: &'r Rules, width: usize, height: usize, mask: &[bool]) -> Wave<'r> {
        let mut wave = Wave::new(rules, width, height);
        assert_eq!(mask.len(), wave.possible.len(), "mask doesn't fit the grid");
        for (p, &m) in wave.possible.iter_mut().zip(mask) {
            *p &= m;
        }
        for (cell, count) in wave.count.iter_mut().enumerate() {
            let row = &wave.possible[cell * rules.size..(cell + 1) * rules.size];
            *count = row.iter().filter(|&&p| p).count();
        }
        wave
    }

    fn is_possible(&self, cell: usize, v: usize) -> bool {
        self.possible[cell * self.rules.size + v]
    }

    /// Removes from `cell` every candidate with no support in `other`, which
    /// lies `d` of it. Returns whether anything was removed.
    fn restrict(&mut self, cell: usize, d: Direction, other: usize) -> Result<bool, Contradiction> {
        let size = self.rules.size;
        let mut changed = false;
        for c in 0..size {
            if !self.is_possible(cell, c) {
                continue;
            }
            let supported = (0..size).any(|n| {
                self.is_possible(other, n) && self.rules.supported[self.rules.at(d, c, n)]
            });
            if !supported {
                self.possible[cell * size + c] = false;
                self.count[cell] -= 1;
                changed = true;
            }
        }
        if self.count[cell] == 0 {
            Err(Contradiction)
        } else {
            Ok(changed)
        }
    }

    /// FIFO propagation from the cells in `queue` until nothing changes.
    fn propagate(&mut self, mut queue: VecDeque<usize>) -> Result<(), Contradiction> {
        let mut queued = vec![false; self.count.len()];
        for &c in &queue {
            queued[c] = true;
        }
        while let Some(changed) = queue.pop_front() {
            queued[changed] = false;
            for d in Direction::ALL {
                let Some(next) = neighbour(self.width, self.height, changed, d) else {
                    continue;
                };
                // `changed` lies opposite(d) of `next`.
                if self.restrict(next, d.opposite(), changed)? && !queued[next] {
                    queued[next] = true;
                    queue.push_back(next);
                }
            }
        }
        Ok(())
    }

    /// The uncollapsed cell with the fewest candidates, ties picked
    /// uniformly from row-major order; `None` when every cell is collapsed.
    fn choose(&self, rng: &mut SplitMix64) -> Option<usize> {
        let mut fewest = usize::MAX;
        let mut tied = Vec::new();
        for (i, &n) in self.count.iter().enumerate() {
            if self.collapsed[i].is_some() {
                continue;
            }
            if n < fewest {
                fewest = n;
                tied.clear();
            }
            if n == fewest {
                tied.push(i);
            }
        }
        if tied.is_empty() {
            None
        } else {
            Some(tied[rng.below(tied.len())])
        }
    }

    /// Draws a candidate for `cell` by score and settles it there, then
    /// tells `observer` what it drew from.
    fn collapse(
        &mut self,
        cell: usize,
        rng: &mut SplitMix64,
        observer: &mut dyn Observer,
    ) -> Result<(), Contradiction> {
        let rules = self.rules;
        let settled: Vec<(Direction, usize)> = Direction::ALL
            .into_iter()
            .filter_map(|d| {
                let n = neighbour(self.width, self.height, cell, d)?;
                Some((d, self.collapsed[n]?))
            })
            .collect();
        let scores: Vec<(usize, f64)> = (0..rules.size)
            .filter(|&c| self.is_possible(cell, c))
            .map(|c| {
                let score = settled.iter().fold(rules.weight[c], |s, &(d, n)| {
                    s * rules.pair[rules.at(d, c, n)]
                });
                (c, score)
            })
            .collect();
        let total: f64 = scores.iter().map(|&(_, s)| s).sum();
        if !(total > 0.0 && total.is_finite()) {
            return Err(Contradiction);
        }
        let target = rng.unit() * total;
        let mut cumulative = 0.0;
        // Rounding can leave `target` just past the last sum: fall back to
        // the last candidate that could be drawn at all.
        let mut chosen = scores
            .iter()
            .rev()
            .find(|&&(_, s)| s > 0.0)
            .map(|&(c, _)| c);
        for &(c, s) in &scores {
            cumulative += s;
            if s > 0.0 && target < cumulative {
                chosen = Some(c);
                break;
            }
        }
        let chosen = chosen.ok_or(Contradiction)?;
        let size = rules.size;
        for v in 0..size {
            self.possible[cell * size + v] = v == chosen;
        }
        self.count[cell] = 1;
        self.collapsed[cell] = Some(chosen);
        observer.collapsed(&CollapseStep {
            cell,
            settled: &settled,
            scores: &scores,
            total,
            chosen,
        });
        Ok(())
    }

    /// Runs the attempt to the end. Each step collapses one cell and
    /// propagation only removes candidates, so this always terminates.
    fn run(
        &mut self,
        rng: &mut SplitMix64,
        observer: &mut dyn Observer,
    ) -> Result<(), Contradiction> {
        if self.count.contains(&0) {
            return Err(Contradiction);
        }
        self.propagate((0..self.count.len()).collect())?;
        while let Some(cell) = self.choose(rng) {
            self.collapse(cell, rng, observer)?;
            self.propagate(VecDeque::from([cell]))?;
        }
        Ok(())
    }
}

/// Generates a `width` × `height` maze from `table` (calculated from `set`).
/// The same seed, tile set and size always give the same maze.
pub fn generate(
    set: &TileSet,
    table: &Table,
    seed: u32,
    width: usize,
    height: usize,
) -> Result<Generation, GenerateError> {
    generate_observed(set, table, seed, width, height, &mut ())
}

/// `generate`, with `observer` told as each attempt starts and as each cell
/// collapses. The one code path: `generate` is this with `&mut ()`, so an
/// observer can't make the maze differ (DR-0034 rule 5).
pub fn generate_observed(
    set: &TileSet,
    table: &Table,
    seed: u32,
    width: usize,
    height: usize,
    observer: &mut dyn Observer,
) -> Result<Generation, GenerateError> {
    let range = MIN_SIZE..=MAX_SIZE;
    if !range.contains(&width) || !range.contains(&height) {
        return Err(GenerateError::BadSize { width, height });
    }
    let rules = Rules::new(set, table);
    let mut attempt = 0;
    loop {
        observer.attempt_started(attempt);
        let mut rng = SplitMix64::new(attempt_seed(seed, attempt));
        let mut wave = Wave::new(&rules, width, height);
        let outcome = wave.run(&mut rng, observer);
        if outcome.is_ok() || attempt == MAX_RESTARTS {
            return Ok(Generation {
                width,
                height,
                cells: wave.collapsed,
                restarts: attempt,
                failed: outcome.is_err(),
            });
        }
        attempt += 1;
    }
}

/// Single WFC runs that start from per-cell candidate masks, for the chunk
/// generator (DR-0031 rule 3). It reads the table once, so a chunk's many
/// runs share that work; the restarts and seeds are the caller's.
pub struct Solver {
    rules: Rules,
}

impl Solver {
    /// `table` must be calculated from `set`.
    pub fn new(set: &TileSet, table: &Table) -> Solver {
        Solver {
            rules: Rules::new(set, table),
        }
    }

    /// One attempt over `width` × `height` cells seeded with `seed`, each
    /// cell starting from the variants `mask[cell * variants + variant]`
    /// allows (never a zero-weight tile). Every cell's variant, row-major,
    /// or `None` on a contradiction. Panics if `mask` doesn't have
    /// `width × height × variants` entries.
    pub fn run(&self, width: usize, height: usize, mask: &[bool], seed: u64) -> Option<Vec<usize>> {
        let mut rng = SplitMix64::new(seed);
        let mut wave = Wave::with_mask(&self.rules, width, height, mask);
        wave.run(&mut rng).ok()?;
        let cells = wave.collapsed.into_iter();
        Some(
            cells
                .map(|c| c.expect("a finished run collapses every cell"))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned against the reference SplitMix64 (seed 1234567): if this
    /// fails, every seed's maze changed.
    #[test]
    fn splitmix64_matches_reference() {
        let mut rng = SplitMix64::new(1234567);
        assert_eq!(rng.next_u64(), 6457827717110365317);
        assert_eq!(rng.next_u64(), 3203168211198807973);
        assert_eq!(rng.next_u64(), 9817491932198370423);
    }

    #[test]
    fn unit_and_below_stay_in_range() {
        let mut rng = SplitMix64::new(7);
        for _ in 0..10_000 {
            let u = rng.unit();
            assert!((0.0..1.0).contains(&u));
            assert!(rng.below(3) < 3);
        }
        assert_eq!(rng.below(1), 0);
    }

    #[test]
    fn neighbours_stop_at_the_edge() {
        // 3 × 2 grid: 0 1 2 / 3 4 5.
        assert_eq!(neighbour(3, 2, 0, Direction::North), None);
        assert_eq!(neighbour(3, 2, 0, Direction::West), None);
        assert_eq!(neighbour(3, 2, 0, Direction::East), Some(1));
        assert_eq!(neighbour(3, 2, 0, Direction::South), Some(3));
        assert_eq!(neighbour(3, 2, 5, Direction::East), None);
        assert_eq!(neighbour(3, 2, 5, Direction::South), None);
        assert_eq!(neighbour(3, 2, 5, Direction::North), Some(2));
    }

    #[test]
    fn attempt_seeds_differ() {
        assert_ne!(attempt_seed(1, 0), attempt_seed(1, 1));
        assert_ne!(attempt_seed(1, 0), attempt_seed(0, 1));
        assert_eq!(attempt_seed(u32::MAX, 20), 0xffff_ffff_0000_0014);
    }
}
