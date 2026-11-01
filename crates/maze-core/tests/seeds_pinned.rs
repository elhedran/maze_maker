//! Slice 2.1's seeds keep giving the same mazes (roadmap 2.6 step 1: "2.1
//! is unchanged"). The fingerprints were taken from `main` at 0f2001b,
//! before the chunk generator was added. If this fails, a seed the tool
//! shows now draws a different maze: that needs a decision, not a test
//! update.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH, MAX_SIZE};
use maze_core::{Generation, Table, TileSet, generate};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

/// FNV-1a over every cell (an open cell as `u64::MAX`), then the restart
/// count and the failed flag. Written here, apart from `maze-core`'s own
/// hashes, so a change there can't hide a change in the mazes.
fn fingerprint(maze: &Generation) -> u64 {
    let words = maze
        .cells
        .iter()
        .map(|c| c.map_or(u64::MAX, |v| v as u64))
        .chain([u64::from(maze.restarts), u64::from(maze.failed)]);
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for word in words {
        for byte in word.to_le_bytes() {
            h = (h ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

#[test]
fn two_one_seeds_give_the_mazes_they_gave_on_main() {
    let set = TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml failed: {e:?}"));
    let table = Table::calculate(&set);
    // (seed, width, height, restarts, fingerprint). Seed 56 at 64 × 64 is
    // the first that restarts, so the restart path is pinned too.
    let (w, h) = (DEFAULT_WIDTH, DEFAULT_HEIGHT);
    let pinned: [(u32, usize, usize, u32, u64); 6] = [
        (0, w, h, 0, 0x1b04_8e15_7d42_d308),
        (1, w, h, 0, 0xa30d_31c9_b4e5_6de7),
        (42, w, h, 0, 0x62c3_fdbe_5ea3_148f),
        (2026, w, h, 0, 0xcc59_bfaf_c82d_45df),
        (u32::MAX, w, h, 0, 0x05d8_01c8_4287_eafc),
        (56, MAX_SIZE, MAX_SIZE, 1, 0x457c_724f_3769_c3f2),
    ];
    for (seed, width, height, restarts, print) in pinned {
        let maze = generate(&set, &table, seed, width, height).unwrap();
        assert!(!maze.failed, "seed {seed}");
        assert_eq!(maze.restarts, restarts, "seed {seed}");
        assert_eq!(
            fingerprint(&maze),
            print,
            "seed {seed} at {width} × {height} now gives a different maze"
        );
    }
}
