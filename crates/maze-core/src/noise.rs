//! Seeded noise for table entries (DR-0024).
//!
//! The hash is written out here rather than taken from `std` (whose hashers
//! aren't promised stable across Rust versions) or a crate, so the same file
//! gives the same table on any machine and any toolchain. Tests pin values.
//!
//! The input is the seed plus the unordered pair of meeting sides, each as
//! (tile id, orientation, the side's direction). The two sides are put in a
//! fixed order before hashing, so the factor is the same whichever tile is
//! the row: that is what makes noise symmetric.

use crate::side::Direction;
use crate::tile_set::Noise;

/// One side of one (tile, orientation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideRef<'a> {
    pub tile_id: &'a str,
    pub orientation: u32,
    pub direction: Direction,
}

impl SideRef<'_> {
    /// Bytes: the id, a 0 byte (ids are snake_case, so never contain one),
    /// the orientation as 4 little-endian bytes, the direction index.
    fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.tile_id.len() + 6);
        out.extend_from_slice(self.tile_id.as_bytes());
        out.push(0);
        out.extend_from_slice(&self.orientation.to_le_bytes());
        out.push(self.direction.index() as u8);
        out
    }
}

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a(state: u64, bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .fold(state, |h, &b| (h ^ u64::from(b)).wrapping_mul(FNV_PRIME))
}

/// SplitMix64's finaliser: spreads FNV's weak low bits across the word.
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// The stable hash of a seed and an unordered pair of sides.
pub fn pair_hash(seed: u32, a: SideRef<'_>, b: SideRef<'_>) -> u64 {
    let (a, b) = (a.encode(), b.encode());
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    let mut h = fnv1a(FNV_OFFSET, &seed.to_le_bytes());
    h = fnv1a(h, &lo);
    h = fnv1a(h, &[0xff]);
    h = fnv1a(h, &hi);
    mix(h)
}

/// The noise factor for two meeting sides: in [1 - amount, 1 + amount).
pub fn factor(noise: &Noise, a: SideRef<'_>, b: SideRef<'_>) -> f64 {
    // The top 53 bits as a fraction in [0, 1): exact in an f64.
    let unit = (pair_hash(noise.seed, a, b) >> 11) as f64 / (1u64 << 53) as f64;
    1.0 + noise.amount * (2.0 * unit - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn side(tile_id: &str, orientation: u32, direction: Direction) -> SideRef<'_> {
        SideRef {
            tile_id,
            orientation,
            direction,
        }
    }

    /// Pinned values. If this test fails, the hash changed and every table
    /// with noise changed with it: that needs a decision, not a test update.
    #[test]
    fn hash_is_pinned() {
        let straight_east = side("corridor_straight", 1, Direction::East);
        let corner_west = side("corridor_corner", 2, Direction::West);
        // Both ways round give the same hash.
        assert_eq!(
            pair_hash(1234, straight_east, corner_west),
            0x9af2_ba46_1a39_bc96
        );
        assert_eq!(
            pair_hash(1234, corner_west, straight_east),
            0x9af2_ba46_1a39_bc96
        );
        let hedge_north = side("hedge", 0, Direction::North);
        let hedge_south = side("hedge", 0, Direction::South);
        assert_eq!(
            pair_hash(1, hedge_north, hedge_south),
            0x5c9c_2337_63e7_e576
        );
        assert_eq!(
            pair_hash(1, hedge_south, hedge_north),
            0x5c9c_2337_63e7_e576
        );
        // And the factor that comes out of it.
        let noise = Noise {
            seed: 1234,
            amount: 0.5,
        };
        let f = factor(&noise, straight_east, corner_west);
        let unit = (0x9af2_ba46_1a39_bc96_u64 >> 11) as f64 / (1u64 << 53) as f64;
        assert_eq!(f, 1.0 + 0.5 * (2.0 * unit - 1.0));
    }

    #[test]
    fn factor_stays_in_range_and_is_symmetric() {
        let noise = Noise {
            seed: 7,
            amount: 0.25,
        };
        let ids = ["a", "bb", "corridor_corner", "room_wall"];
        for id_a in ids {
            for id_b in ids {
                for k in 0..4 {
                    for d in Direction::ALL {
                        let a = side(id_a, k, d);
                        let b = side(id_b, (k + 1) % 4, d.opposite());
                        let f = factor(&noise, a, b);
                        assert!((0.75..1.25).contains(&f), "{f}");
                        assert_eq!(f, factor(&noise, b, a));
                    }
                }
            }
        }
    }

    #[test]
    fn direction_tells_pairs_apart() {
        // "B east of A" and "B west of A" are different pairs of sides.
        let a_east = side("t", 0, Direction::East);
        let b_west = side("u", 0, Direction::West);
        let a_west = side("t", 0, Direction::West);
        let b_east = side("u", 0, Direction::East);
        assert_ne!(pair_hash(9, a_east, b_west), pair_hash(9, a_west, b_east));
    }

    #[test]
    fn zero_amount_is_exactly_one() {
        let noise = Noise {
            seed: 99,
            amount: 0.0,
        };
        let f = factor(
            &noise,
            side("x", 0, Direction::North),
            side("y", 0, Direction::South),
        );
        assert_eq!(f, 1.0);
    }
}
