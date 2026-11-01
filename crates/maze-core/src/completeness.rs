//! The completeness check (DR-0036 rule 7, carrying DR-0031 rule 6), run
//! when a tile set loads.
//!
//! A patch's last-resort run can't contradict. It uses only the *fallback
//! tiles*: those of weight above 0 whose four sides are each hedge or
//! passage (in the core set: hedge, dead ends, corners, straights, Ts and
//! the crossroads), and it meets stored tiles of every kind. It can't
//! contradict when
//!
//! 1. each of the 16 ways to put hedge or passage on a cell's four sides is
//!    some fallback tile's sides, and
//! 2. in every direction, every tile of weight above 0 has an entry weight
//!    above 0 with every fallback tile that faces its hedge side with hedge,
//!    or its passage or door side with passage.
//!
//! Clause 2 covers fallback tiles against each other, and stored room
//! hedge and doorway doors against the fallback across the frontier
//! (stored cells show only hedge, passage or door there, DR-0036 rule 1).
//! Then each cell edge settles on its own, and propagation never empties a
//! cell (the research note's "Why the fallback can't fail"). Clause 2 reads
//! entry weights, the weight the solver's support reads (DR-0028 rule 1),
//! not tile weights: a `passage_passage` or `hedge_hedge` base of 0, or an
//! exception multiplier of 0, breaks the proof while every tile weight is
//! still above 0.

use crate::side::{Direction, SideKind};
use crate::table::{Entry, Table};
use crate::tile_set::{LoadError, TileSet, Variant};

/// Whether `variant` is a fallback tile: weight above 0, and every side
/// hedge or passage. The fallback run may use only these.
pub fn is_fallback(set: &TileSet, variant: &Variant) -> bool {
    set.tile_of(variant).weight > 0.0
        && variant
            .sides
            .iter()
            .all(|&s| matches!(s, SideKind::Hedge | SideKind::Passage))
}

/// The 16 mixes of hedge and passage as N, E, S, W sides: bit `d` of the
/// count is passage facing direction `d`.
fn mixes() -> impl Iterator<Item = [SideKind; 4]> {
    (0..16u32).map(|m| {
        Direction::ALL.map(|d| {
            if m & (1 << d.index()) != 0 {
                SideKind::Passage
            } else {
                SideKind::Hedge
            }
        })
    })
}

/// Every way `set` (with `table` calculated from it) falls short of DR-0036
/// rule 7: one error per missing mix of sides, and one per cause of a zero
/// pairing, with the first such pair in table order as its example. Empty
/// when the set can make endless mazes.
pub fn check(set: &TileSet, table: &Table) -> Vec<LoadError> {
    let fallback: Vec<usize> = (0..set.variants.len())
        .filter(|&v| is_fallback(set, &set.variants[v]))
        .collect();
    let mut errors: Vec<LoadError> = mixes()
        .filter(|sides| !fallback.iter().any(|&v| set.variants[v].sides == *sides))
        .map(|sides| LoadError::NoFallbackTile { sides })
        .collect();
    let live: Vec<usize> = (0..set.variants.len())
        .filter(|&v| set.tile_of(&set.variants[v]).weight > 0.0)
        .collect();
    let mut causes: Vec<String> = Vec::new();
    for d in Direction::ALL {
        for &row in &live {
            for &column in &fallback {
                let entry = table.entry(d, row, column);
                let must_meet = matches!(
                    (entry.row_side, entry.column_side),
                    (SideKind::Hedge, SideKind::Hedge)
                        | (SideKind::Passage | SideKind::Door, SideKind::Passage)
                );
                if !must_meet || entry.weight > 0.0 {
                    continue;
                }
                let cause = zero_cause(set, entry);
                if causes.contains(&cause) {
                    continue;
                }
                errors.push(LoadError::FallbackPairIsZero {
                    cause: cause.clone(),
                    row: set.tile_of(&set.variants[row]).id.clone(),
                    row_side: entry.row_side,
                    column: set.tile_of(&set.variants[column]).id.clone(),
                    column_side: entry.column_side,
                });
                causes.push(cause);
            }
        }
    }
    errors
}

/// Why an allowed entry between two tiles of weight above 0 has weight 0.
/// Noise is above 0 (its amount is below 1), so it is the base, the
/// exception, or the product rounding to 0.
fn zero_cause(set: &TileSet, entry: &Entry) -> String {
    let exception = entry
        .exception
        .filter(|&i| set.exceptions[i].multiplier == 0.0);
    if entry.base == 0.0 {
        let pair = entry
            .row_side
            .base_pair(entry.column_side)
            .expect("hedge meets hedge, and passage or door meets passage");
        format!("Base {} is 0", pair.key())
    } else if let Some(i) = exception {
        format!("Exception \"{}\" has multiplier 0", set.exceptions[i].name)
    } else {
        "The weights are so small they multiply to 0".to_string()
    }
}

/// A mix of hedge and passage in words, with its corridor shape, e.g.
/// "passage on N and S, hedge on E and W (a straight)".
pub(crate) fn describe_sides(sides: [SideKind; 4]) -> String {
    let (passage, hedge): (Vec<Direction>, Vec<Direction>) = Direction::ALL
        .into_iter()
        .partition(|d| sides[d.index()] == SideKind::Passage);
    let shape = match passage.len() {
        0 => "solid hedge",
        1 => "a dead end",
        2 if passage[0].opposite() == passage[1] => "a straight",
        2 => "a corner",
        3 => "a T",
        _ => "a crossroads",
    };
    match passage.len() {
        0 => format!("hedge on all four sides ({shape})"),
        4 => format!("passage on all four sides ({shape})"),
        _ => format!(
            "passage on {}, hedge on {} ({shape})",
            letters(&passage),
            letters(&hedge)
        ),
    }
}

/// "N", "N and S", "N, E and S".
fn letters(directions: &[Direction]) -> String {
    let names: Vec<String> = directions.iter().map(|d| d.letter().to_string()).collect();
    match names.split_last() {
        Some((last, rest)) if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => names.concat(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SideKind::{Hedge as H, Passage as P};

    #[test]
    fn sixteen_distinct_mixes() {
        let all: Vec<[SideKind; 4]> = mixes().collect();
        assert_eq!(all.len(), 16);
        for (i, a) in all.iter().enumerate() {
            assert!(!all[i + 1..].contains(a));
        }
    }

    #[test]
    fn mixes_are_described_by_shape() {
        assert_eq!(
            describe_sides([H, H, H, H]),
            "hedge on all four sides (solid hedge)"
        );
        assert_eq!(
            describe_sides([P, P, P, P]),
            "passage on all four sides (a crossroads)"
        );
        assert_eq!(
            describe_sides([P, H, H, H]),
            "passage on N, hedge on E, S and W (a dead end)"
        );
        assert_eq!(
            describe_sides([P, H, P, H]),
            "passage on N and S, hedge on E and W (a straight)"
        );
        assert_eq!(
            describe_sides([H, P, P, H]),
            "passage on E and S, hedge on N and W (a corner)"
        );
        assert_eq!(
            describe_sides([P, P, H, P]),
            "passage on N, E and W, hedge on S (a T)"
        );
    }
}
