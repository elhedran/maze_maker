//! A seeded walker that avoids going back over its own path (2.6 step 2b).
//!
//! Ian (`human:ian`), 2026-11-01: "keep a list of where it has walked and
//! make it a much lower chance (not impossible) for it to walk over a spot
//! it walked before rather than a spot it hasn't walked to."
//!
//! Each step the walker moves to a joined stored neighbour, drawn by
//! weight. A neighbour it hasn't stood on has weight 1; one it has stood on
//! once has `revisit_weight` (w); one it has stood on c times has
//! 1 / (1 + c (1/w − 1)), which falls about as w / c. Every weight is above
//! 0, so a walker in a dead end, or with only walked cells around it, still
//! goes back. At w = 1 every weight is 1: the plain random walk, draw for
//! draw (`World::random_step`).
//!
//! **Why count visits, not just mark them (Rowan, a two-way door).** With
//! only a mark, once every cell round the walker is walked all weights are
//! alike again and it wanders as the plain walk does. In a room it has
//! covered, with one or two ways out, that can last a long time: seed 94's
//! walk at w = 0.1 grew nothing in 1,000 steps. With counts, the
//! well-trodden middle of such a room is less likely to be crossed again
//! than its less walked edges, so the walker drifts out. Over step 1's 100
//! test walks of 1,000 steps (world seeds 0 to 99) at w = 0.1, the fewest
//! patches a walk grew went from 0 (marks) to 5 (counts), and the median
//! from 27 to 34. The first revisit weighs w either way, as Ian asked.
//!
//! The walker owns its random stream and its visit counts, so a walk is a
//! pure function of (world, start, walker seed, revisit weight, steps) and
//! replays exactly. It holds no globals and is `Send`, so it can run on its
//! own thread behind a plain API (Ian's long-term intent: an engine that can
//! sit beside a game on its own thread).

use crate::wfc::SplitMix64;
use crate::world::{Pos, World};
use std::collections::BTreeMap;

/// The revisit weight `Walker::new` uses. An engineers' starting value
/// (a two-way door); Wren picks the value from 2.6 step 2b's sweep.
pub const REVISIT_WEIGHT: f64 = 0.1;

/// A seeded walk over a world's joined stored cells.
#[derive(Clone, Debug)]
pub struct Walker {
    at: Pos,
    /// How many times the walker has stood on each cell, the start
    /// included; the keys are the visited cells.
    visits: BTreeMap<Pos, u32>,
    rng: SplitMix64,
    revisit_weight: f64,
}

impl Walker {
    /// A walker at `start` with its own random stream from `seed`, at the
    /// default revisit weight.
    pub fn new(start: Pos, seed: u64) -> Walker {
        Walker {
            at: start,
            visits: BTreeMap::from([(start, 1)]),
            rng: SplitMix64::new(seed),
            revisit_weight: REVISIT_WEIGHT,
        }
    }

    /// As `new`, with the weight of an already-walked neighbour against 1
    /// for an unwalked one. `None` unless 0 < `revisit_weight` ≤ 1: above 0
    /// so a walker is never stuck, and at most 1 since more would make it
    /// prefer its own path, which nobody has asked for.
    pub fn with_revisit_weight(start: Pos, seed: u64, revisit_weight: f64) -> Option<Walker> {
        (revisit_weight > 0.0 && revisit_weight <= 1.0).then(|| Walker {
            revisit_weight,
            ..Walker::new(start, seed)
        })
    }

    /// Where the walker stands.
    pub fn at(&self) -> Pos {
        self.at
    }

    /// How many times the walker has stood on each cell, the start
    /// included, in key order. The keys are the cells it has walked.
    pub fn visits(&self) -> &BTreeMap<Pos, u32> {
        &self.visits
    }

    pub fn revisit_weight(&self) -> f64 {
        self.revisit_weight
    }

    /// One step, then the world grows around the walker (DR-0036 rule 2).
    /// Returns the new position. Panics if the walker isn't on a stored
    /// cell of `world`.
    pub fn step(&mut self, world: &mut World) -> Pos {
        let at = self.advance(world);
        world.grow_around(at);
        at
    }

    /// One step without growing the world, for a caller that grows it
    /// itself (the measures time each patch). `step` is `advance` then
    /// `World::grow_around`. Stays put only if no stored neighbour is
    /// joined, which a grown world never has: rule 4 joins every stored
    /// cell to the origin.
    pub fn advance(&mut self, world: &World) -> Pos {
        let ways = world.joined_neighbours(self.at);
        if !ways.is_empty() {
            let weights: Vec<f64> = ways
                .iter()
                .map(|p| {
                    let c = self.visits.get(p).copied().unwrap_or(0);
                    weight(c, self.revisit_weight)
                })
                .collect();
            let i = if weights.iter().all(|&w| w == weights[0]) {
                // All alike: the plain random walk's draw, so weight 1 walks
                // exactly as `World::random_step` does.
                self.rng.below(ways.len())
            } else {
                pick(&weights, self.rng.unit())
            };
            self.at = ways[i];
        }
        let c = self.visits.entry(self.at).or_insert(0);
        *c = c.saturating_add(1);
        self.at
    }
}

// Cheap proof that a walker can move to another thread.
const _: fn() = || {
    fn send<T: Send>() {}
    send::<Walker>();
};

/// The draw weight of a cell walked `visits` times: 1 unwalked, `w` once,
/// then 1 / (1 + visits (1/w − 1)). Above 0 for any count.
fn weight(visits: u32, w: f64) -> f64 {
    1.0 / (1.0 + f64::from(visits) * (1.0 / w - 1.0))
}

/// The index `u` (in [0, 1)) falls on when `weights` (all above 0) are laid
/// end to end and scaled to 1.
fn pick(weights: &[f64], u: f64) -> usize {
    let mut left = u * weights.iter().sum::<f64>();
    for (i, &w) in weights.iter().enumerate() {
        if left < w {
            return i;
        }
        left -= w;
    }
    // Rounding can leave `left` a hair past the last weight.
    weights.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_lays_weights_end_to_end() {
        let w = [1.0, 0.1, 1.0];
        assert_eq!(pick(&w, 0.0), 0);
        assert_eq!(pick(&w, 0.47), 0);
        assert_eq!(pick(&w, 0.49), 1);
        assert_eq!(pick(&w, 0.53), 2);
        assert_eq!(pick(&w, 0.999_999_999), 2);
    }

    #[test]
    fn weights_fall_with_visits_and_never_reach_0() {
        let close = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert_eq!(weight(0, 0.1), 1.0);
        assert!(close(weight(1, 0.1), 0.1));
        assert!(close(weight(2, 0.1), 1.0 / 19.0));
        assert!(weight(2, 0.1) < weight(1, 0.1));
        assert!(weight(u32::MAX, 1e-9) > 0.0);
        // At w = 1 every cell is alike: the plain random walk.
        assert_eq!(weight(0, 1.0), 1.0);
        assert_eq!(weight(1000, 1.0), 1.0);
    }

    #[test]
    fn the_weight_must_be_above_0_and_at_most_1() {
        for bad in [0.0, -0.5, 1.5, f64::NAN, f64::INFINITY] {
            assert!(
                Walker::with_revisit_weight((0, 0), 1, bad).is_none(),
                "{bad}"
            );
        }
        for good in [1e-9, 0.02, 1.0] {
            let w = Walker::with_revisit_weight((0, 0), 1, good).expect("in range");
            assert_eq!(w.revisit_weight(), good);
        }
        assert_eq!(Walker::new((3, 4), 1).revisit_weight(), REVISIT_WEIGHT);
        assert_eq!(Walker::new((3, 4), 1).visits().len(), 1);
    }
}
