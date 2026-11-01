//! "Why is this here?" for one cell of a generated maze (DR-0034 rules 4
//! to 7): replay the seed through the one WFC code path, watch the cell's
//! collapse, and lay out what the draw saw.
//!
//! Nothing is stored. The same seed, tile set and size give the same maze
//! (DR-0028 rule 5), so replaying is enough; if generation ever came to
//! depend on anything else, the trace would describe a different maze, and
//! the identity tests in `tests/trace.rs` would fail.

use crate::side::Direction;
use crate::table::Table;
use crate::tile_set::TileSet;
use crate::wfc::{
    CollapseStep, GenerateError, Generation, MAX_SIZE, MIN_SIZE, Observer, generate_observed,
    neighbour,
};
use std::fmt;

/// What the replay found for one cell.
#[derive(Clone, Debug, PartialEq)]
pub struct CellTrace {
    /// As `generate` returns them for the same request.
    pub restarts: u32,
    pub failed: bool,
    /// Collapses the shown attempt made: width × height when it succeeded.
    pub steps: usize,
    /// `None` only when a failed maze's last attempt left the cell open.
    pub collapse: Option<CollapseTrace>,
}

/// The cell's draw, in the attempt whose grid is shown.
#[derive(Clone, Debug, PartialEq)]
pub struct CollapseTrace {
    /// When it collapsed in that attempt, from 1.
    pub step: usize,
    /// N, E, S, W, by `Direction::index`.
    pub neighbours: [NeighbourAtDraw; 4],
    /// Those left after propagation, by share (highest first), then table
    /// order (DR-0034 rule 6). The draw itself walked table order.
    pub candidates: Vec<CandidateScore>,
    /// Variants that weren't possible at the draw, zero-weight tiles
    /// included: `candidates.len() + ruled_out` is the variant count.
    pub ruled_out: usize,
    /// The solver's sum of the scores.
    pub total: f64,
    /// The variant drawn.
    pub chosen: usize,
}

/// One side of the cell, as the draw saw it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeighbourAtDraw {
    /// Collapsed before the draw, to this variant; it has a factor.
    Counted(usize),
    /// Collapsed after the draw, to this variant, or `None` if a failed
    /// maze left it open. It shaped the cell only through propagation.
    Later(Option<usize>),
    /// Past the edge of the maze: nothing there counts (DR-0028 rule 6).
    Edge,
}

/// One candidate's score, with its parts.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateScore {
    pub variant: usize,
    pub tile_weight: f64,
    /// One per counted neighbour, N, E, S, W order.
    pub factors: Vec<PairFactor>,
    /// The opening-pressure factor (DR-0036 rule 9), passed through from
    /// the solver: the tile set's `opening_pressure` when this candidate
    /// opens nothing new and the pressure isn't 1, else 1.
    pub pressure: f64,
    /// The solver's own score, passed through. It equals `tile_weight`
    /// times each factor in turn, left to right, then `pressure`, bit for
    /// bit.
    pub score: f64,
    /// `score / total`.
    pub share: f64,
}

/// The pair factor one counted neighbour gave a candidate (DR-0028 rule 3).
#[derive(Clone, Debug, PartialEq)]
pub struct PairFactor {
    /// From the cell to the neighbour.
    pub direction: Direction,
    pub neighbour: usize,
    pub base: f64,
    /// Index into `TileSet::exceptions`; at most one applies (DR-0022).
    pub exception: Option<usize>,
    /// The exception's multiplier, or 1 when there is none.
    pub multiplier: f64,
    pub noise: f64,
    /// `base × multiplier × noise`, 0 where the sides may not meet.
    pub factor: f64,
}

/// Why `trace_cell` refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceError {
    /// The size is one `generate` refuses.
    Generate(GenerateError),
    /// The cell (column `x`, row `y`, from 0) isn't in the maze.
    CellOutside {
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    },
}

impl fmt::Display for TraceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraceError::Generate(e) => e.fmt(f),
            TraceError::CellOutside {
                x,
                y,
                width,
                height,
            } => write!(
                f,
                "Cell ({x}, {y}) is outside the {width} × {height} maze; x must be below {width} and y below {height}."
            ),
        }
    }
}

impl std::error::Error for TraceError {}

/// Replays the maze `generate` makes from (`seed`, `width`, `height`) and
/// explains the draw at `cell`, given as (column, row) from the north-west.
/// Both checks come before the replay; a failed maze is not an error.
pub fn trace_cell(
    set: &TileSet,
    table: &Table,
    seed: u32,
    width: usize,
    height: usize,
    cell: (usize, usize),
) -> Result<CellTrace, TraceError> {
    // `generate` checks the size too, but only once it starts; checking it
    // first means a bad size reads as one rather than as a cell outside.
    let range = MIN_SIZE..=MAX_SIZE;
    if !range.contains(&width) || !range.contains(&height) {
        return Err(TraceError::Generate(GenerateError::BadSize {
            width,
            height,
        }));
    }
    let (x, y) = cell;
    if x >= width || y >= height {
        return Err(TraceError::CellOutside {
            x,
            y,
            width,
            height,
        });
    }
    let target = y * width + x;
    let mut watch = Watch {
        target,
        steps: 0,
        seen: None,
    };
    let generation = generate_observed(set, table, seed, width, height, &mut watch)
        .map_err(TraceError::Generate)?;
    Ok(CellTrace {
        restarts: generation.restarts,
        failed: generation.failed,
        steps: watch.steps,
        collapse: watch
            .seen
            .map(|seen| collapse_trace(set, table, &generation, target, &seen)),
    })
}

/// What the observer copies out of one collapse: the borrowed parts of a
/// `CollapseStep`, owned, plus when it happened.
#[derive(Clone, Debug)]
struct Seen {
    step: usize,
    settled: Vec<(Direction, usize)>,
    scores: Vec<(usize, f64)>,
    pressure: Vec<f64>,
    total: f64,
    chosen: usize,
}

impl Seen {
    fn new(step: usize, collapse: &CollapseStep<'_>) -> Seen {
        Seen {
            step,
            settled: collapse.settled.to_vec(),
            scores: collapse.scores.to_vec(),
            pressure: collapse.pressure.to_vec(),
            total: collapse.total,
            chosen: collapse.chosen,
        }
    }
}

/// Keeps the target cell's collapse from the latest attempt only, so the
/// trace is of the attempt whose grid is shown (DR-0034 rule 6).
struct Watch {
    target: usize,
    /// Collapses so far in the current attempt.
    steps: usize,
    seen: Option<Seen>,
}

impl Observer for Watch {
    fn attempt_started(&mut self, _attempt: u32) {
        self.steps = 0;
        self.seen = None;
    }

    fn collapsed(&mut self, step: &CollapseStep<'_>) {
        self.steps += 1;
        if step.cell == self.target {
            self.seen = Some(Seen::new(self.steps, step));
        }
    }
}

/// Lays out one observed collapse against the finished grid.
fn collapse_trace(
    set: &TileSet,
    table: &Table,
    generation: &Generation,
    cell: usize,
    seen: &Seen,
) -> CollapseTrace {
    let side = |d: Direction| {
        let Some(j) = neighbour(generation.width, generation.height, cell, d) else {
            return NeighbourAtDraw::Edge;
        };
        match seen.settled.iter().find(|&&(sd, _)| sd == d) {
            Some(&(_, n)) => NeighbourAtDraw::Counted(n),
            None => NeighbourAtDraw::Later(generation.cells[j]),
        }
    };
    let neighbours = Direction::ALL.map(side);
    let mut candidates: Vec<CandidateScore> = seen
        .scores
        .iter()
        .zip(&seen.pressure)
        .map(|(&(c, score), &pressure)| CandidateScore {
            variant: c,
            tile_weight: set.tile_of(&set.variants[c]).weight,
            factors: seen
                .settled
                .iter()
                .map(|&(d, n)| pair_factor(set, table, d, c, n))
                .collect(),
            pressure,
            score,
            // The solver only draws when `total` is finite and above 0, and
            // every score is then finite and at least 0, so no NaN here.
            share: score / seen.total,
        })
        .collect();
    candidates.sort_by(|a, b| b.share.total_cmp(&a.share).then(a.variant.cmp(&b.variant)));
    CollapseTrace {
        step: seen.step,
        neighbours,
        ruled_out: table.size() - candidates.len(),
        candidates,
        total: seen.total,
        chosen: seen.chosen,
    }
}

/// The factor `n`, lying `d` of candidate `c`, gives it. The same
/// expression as the solver's `Rules::new`, so `tile weight × factors` is
/// the solver's score exactly; the factor test checks that it stays so.
fn pair_factor(set: &TileSet, table: &Table, d: Direction, c: usize, n: usize) -> PairFactor {
    let e = table.entry(d, c, n);
    let multiplier = e.exception.map_or(1.0, |i| set.exceptions[i].multiplier);
    PairFactor {
        direction: d,
        neighbour: n,
        base: e.base,
        exception: e.exception,
        multiplier,
        noise: e.noise,
        factor: if e.allowed {
            e.base * multiplier * e.noise
        } else {
            0.0
        },
    }
}

#[cfg(test)]
mod tests {
    //! Per-cell checks. Tracing every cell one by one would replay the maze
    //! once per cell, so these record every collapse in one pass and lay
    //! each out with `collapse_trace`, the function `trace_cell` uses; the
    //! last test checks `trace_cell` agrees with that for whole mazes.

    use super::*;
    use crate::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH};

    const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

    fn core() -> (TileSet, Table) {
        let set = TileSet::from_toml(CORE).expect("core.toml loads");
        let table = Table::calculate(&set);
        (set, table)
    }

    /// Every cell's collapse in the latest attempt.
    struct All {
        steps: usize,
        seen: Vec<Option<Seen>>,
    }

    impl Observer for All {
        fn attempt_started(&mut self, _attempt: u32) {
            self.steps = 0;
            self.seen.iter_mut().for_each(|s| *s = None);
        }

        fn collapsed(&mut self, step: &CollapseStep<'_>) {
            self.steps += 1;
            self.seen[step.cell] = Some(Seen::new(self.steps, step));
        }
    }

    /// The maze and each cell's trace, from one replay.
    fn all_traces(
        set: &TileSet,
        table: &Table,
        seed: u32,
        width: usize,
        height: usize,
    ) -> (Generation, Vec<Option<CollapseTrace>>) {
        let mut all = All {
            steps: 0,
            seen: vec![None; width * height],
        };
        let generation = generate_observed(set, table, seed, width, height, &mut all).unwrap();
        let traces = all
            .seen
            .iter()
            .enumerate()
            .map(|(i, s)| {
                s.as_ref()
                    .map(|s| collapse_trace(set, table, &generation, i, s))
            })
            .collect();
        (generation, traces)
    }

    /// DR-0034's factor test, for every cell of several seeds.
    #[test]
    fn factors_multiply_to_the_solvers_score() {
        let (set, table) = core();
        let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        for seed in [0, 1, 7, 42, 2026] {
            let (maze, traces) = all_traces(&set, &table, seed, w, h);
            assert!(!maze.failed, "seed {seed}");
            for (i, trace) in traces.iter().enumerate() {
                let t = trace.as_ref().expect("a full maze traces every cell");
                assert_eq!(Some(t.chosen), maze.cells[i], "seed {seed} cell {i}");
                assert_eq!(t.candidates.len() + t.ruled_out, table.size());
                assert!(t.total > 0.0 && t.total.is_finite());

                let counted: Vec<(Direction, usize)> = Direction::ALL
                    .into_iter()
                    .filter_map(|d| match t.neighbours[d.index()] {
                        NeighbourAtDraw::Counted(n) => Some((d, n)),
                        _ => None,
                    })
                    .collect();
                for c in &t.candidates {
                    // Exact: the same multiplications in the same order.
                    let product =
                        c.factors.iter().fold(c.tile_weight, |s, f| s * f.factor) * c.pressure;
                    assert_eq!(c.pressure, 1.0, "no opening_pressure key, no pressure");
                    assert_eq!(
                        product, c.score,
                        "seed {seed} cell {i} variant {}",
                        c.variant
                    );
                    assert_eq!(c.share, c.score / t.total);
                    assert!(c.share.is_finite() && c.share >= 0.0);
                    let directions: Vec<(Direction, usize)> = c
                        .factors
                        .iter()
                        .map(|f| (f.direction, f.neighbour))
                        .collect();
                    assert_eq!(directions, counted, "factors follow the counted neighbours");
                    for f in &c.factors {
                        let e = table.entry(f.direction, c.variant, f.neighbour);
                        assert_eq!(
                            (f.base, f.noise, f.exception),
                            (e.base, e.noise, e.exception)
                        );
                        let product = f.base * f.multiplier * f.noise;
                        assert_eq!(f.factor, product);
                    }
                }
                let sum: f64 = t.candidates.iter().map(|c| c.share).sum();
                assert!((sum - 1.0).abs() < 1e-9, "shares sum to {sum}");
                assert_eq!(
                    t.candidates
                        .iter()
                        .filter(|c| c.variant == t.chosen)
                        .count(),
                    1,
                    "the drawn tile is exactly one candidate"
                );
                for pair in t.candidates.windows(2) {
                    let (a, b) = (&pair[0], &pair[1]);
                    assert!(
                        a.share > b.share || (a.share == b.share && a.variant < b.variant),
                        "candidates out of order: {} then {}",
                        a.variant,
                        b.variant
                    );
                }
            }
        }
    }

    /// DR-0034's factor test with opening pressure on (roadmap 2.3):
    /// `weight × factors × pressure` is the solver's score exactly, over 200
    /// seeds, and the pressure lands exactly on candidates whose walkable
    /// sides all face counted neighbours; a side past the edge is new.
    #[test]
    fn pressure_is_the_last_factor_over_200_seeds() {
        let text = CORE.replace(
            "name = \"core\"\n",
            "name = \"core\"\nopening_pressure = 0.5\n",
        );
        let set = TileSet::from_toml(&text).expect("core.toml with pressure loads");
        let table = Table::calculate(&set);
        let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let (mut pressed, mut edge_kept_open) = (0usize, 0usize);
        for seed in 0..200 {
            let (maze, traces) = all_traces(&set, &table, seed, w, h);
            assert!(!maze.failed, "seed {seed}");
            for (i, trace) in traces.iter().enumerate() {
                let t = trace.as_ref().expect("a full maze traces every cell");
                for c in &t.candidates {
                    let product =
                        c.factors.iter().fold(c.tile_weight, |s, f| s * f.factor) * c.pressure;
                    assert_eq!(
                        product, c.score,
                        "seed {seed} cell {i} variant {}",
                        c.variant
                    );
                    assert_eq!(c.share, c.score / t.total);
                    // What rule 9 says, worked out again from the trace.
                    let sides = set.variants[c.variant].sides;
                    let walks: Vec<Direction> = Direction::ALL
                        .into_iter()
                        .filter(|d| sides[d.index()] != crate::side::SideKind::Hedge)
                        .collect();
                    let faces_counted = |d: &Direction| {
                        matches!(t.neighbours[d.index()], NeighbourAtDraw::Counted(_))
                    };
                    let closed = !walks.is_empty() && walks.iter().all(faces_counted);
                    assert_eq!(
                        c.pressure,
                        if closed { 0.5 } else { 1.0 },
                        "seed {seed} cell {i} variant {}",
                        c.variant
                    );
                    pressed += usize::from(closed);
                    let only_edge_open = !closed
                        && walks.iter().all(|d| {
                            faces_counted(d)
                                || matches!(t.neighbours[d.index()], NeighbourAtDraw::Edge)
                        });
                    edge_kept_open += usize::from(only_edge_open);
                }
            }
        }
        assert!(pressed > 0, "the pressure never applied");
        assert!(
            edge_kept_open > 0,
            "no candidate was kept open by the edge alone"
        );
    }

    /// Counted neighbours came earlier, later ones later, both with the
    /// final tile; edges are exactly the border.
    #[test]
    fn neighbours_say_when_they_collapsed() {
        let (set, table) = core();
        let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let mut counted_any = false;
        for seed in [3, 11, 99] {
            let (maze, traces) = all_traces(&set, &table, seed, w, h);
            let step = |i: usize| traces[i].as_ref().unwrap().step;
            let mut steps: Vec<usize> = (0..w * h).map(step).collect();
            steps.sort_unstable();
            assert_eq!(steps, (1..=w * h).collect::<Vec<_>>(), "each cell once");
            for (i, t) in traces.iter().enumerate() {
                let t = t.as_ref().unwrap();
                for d in Direction::ALL {
                    match (t.neighbours[d.index()], neighbour(w, h, i, d)) {
                        (NeighbourAtDraw::Edge, None) => {}
                        (NeighbourAtDraw::Counted(n), Some(j)) => {
                            counted_any = true;
                            assert_eq!(Some(n), maze.cells[j]);
                            assert!(step(j) < t.step);
                        }
                        (NeighbourAtDraw::Later(n), Some(j)) => {
                            assert_eq!(n, maze.cells[j]);
                            assert!(n.is_some());
                            assert!(step(j) > t.step);
                        }
                        (n, j) => panic!("seed {seed} cell {i} {d:?}: {n:?} but neighbour {j:?}"),
                    }
                }
            }
            // The first cell drawn had nothing settled round it.
            let first = (0..w * h).find(|&i| step(i) == 1).unwrap();
            let t = traces[first].as_ref().unwrap();
            assert!(t.candidates.iter().all(|c| c.factors.is_empty()));
        }
        assert!(counted_any);
    }

    #[test]
    fn trace_cell_matches_the_one_pass_record() {
        let (set, table) = core();
        for (seed, w, h) in [(5, 6, 4), (8, 1, 1), (13, 3, 7)] {
            let (maze, traces) = all_traces(&set, &table, seed, w, h);
            for (i, expected) in traces.iter().enumerate() {
                let t = trace_cell(&set, &table, seed, w, h, (i % w, i / w)).unwrap();
                assert_eq!(&t.collapse, expected);
                assert_eq!((t.restarts, t.failed), (maze.restarts, maze.failed));
                assert_eq!(t.steps, w * h);
            }
        }
        // A few cells of the default size, corners included.
        let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let (_, traces) = all_traces(&set, &table, 48213, w, h);
        for (x, y) in [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1), (14, 2)] {
            let t = trace_cell(&set, &table, 48213, w, h, (x, y)).unwrap();
            assert_eq!(t.collapse, traces[y * w + x]);
        }
    }

    #[test]
    fn a_lone_candidate_has_share_one() {
        let (set, table) = core();
        let (_, traces) = all_traces(&set, &table, 1, DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let lone: Vec<&CollapseTrace> = traces
            .iter()
            .flatten()
            .filter(|t| t.candidates.len() == 1)
            .collect();
        assert!(!lone.is_empty(), "no cell had one candidate left");
        for t in lone {
            assert_eq!(t.candidates[0].share, 1.0);
            assert_eq!(t.ruled_out, table.size() - 1);
        }
    }

    #[test]
    fn bad_sizes_and_cells_outside_are_refused() {
        let (set, table) = core();
        for (w, h) in [(0, 16), (65, 16), (24, 0), (24, 65)] {
            let err = trace_cell(&set, &table, 1, w, h, (0, 0)).unwrap_err();
            assert_eq!(
                err,
                TraceError::Generate(GenerateError::BadSize {
                    width: w,
                    height: h
                })
            );
            assert!(err.to_string().contains("from 1 to 64"), "{err}");
        }
        // A bad size wins over a cell that would be outside it.
        let err = trace_cell(&set, &table, 1, 0, 0, (5, 5)).unwrap_err();
        assert!(matches!(err, TraceError::Generate(_)));
        for cell in [(24, 0), (0, 16), (usize::MAX, usize::MAX)] {
            let err = trace_cell(&set, &table, 1, 24, 16, cell).unwrap_err();
            assert_eq!(
                err,
                TraceError::CellOutside {
                    x: cell.0,
                    y: cell.1,
                    width: 24,
                    height: 16
                }
            );
            assert!(
                err.to_string().contains("outside the 24 × 16 maze"),
                "{err}"
            );
        }
    }
}
