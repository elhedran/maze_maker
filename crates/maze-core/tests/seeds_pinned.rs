//! Slice 2.1's seeds keep giving the same mazes (roadmap 2.6 step 1: "2.1
//! is unchanged"). The fingerprints were first taken from `main` at
//! 0f2001b, before the chunk generator was added, and retaken on
//! 2026-11-01 when core.toml's numbers were retuned for the game (Wren's
//! G.1 tuning, variant v5; docs/plans/g1-tuning.md). If this fails, a seed
//! the tool shows now draws a different maze: that needs a decision, not a
//! test update.

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

/// (seed, width, height, restarts, fingerprint). Seed 692 at 64 × 64 is the
/// first that restarts with the retuned numbers (it was 56 before), so the
/// restart path is pinned too.
const PINNED: [(u32, usize, usize, u32, u64); 6] = [
    (0, DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0x2c5d_0c3a_a601_ee02),
    (1, DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0xf52e_51fc_8895_0c46),
    (42, DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0x32e4_eab0_87db_e164),
    (
        2026,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        0,
        0xa1de_2bca_26cb_4c08,
    ),
    (
        u32::MAX,
        DEFAULT_WIDTH,
        DEFAULT_HEIGHT,
        0,
        0xb10f_0dc2_6c79_9bf2,
    ),
    (692, MAX_SIZE, MAX_SIZE, 1, 0xf374_b895_34af_d1d8),
];

fn check_pinned(text: &str) {
    let set = TileSet::from_toml(text).unwrap_or_else(|e| panic!("core.toml failed: {e:?}"));
    let table = Table::calculate(&set);
    for (seed, width, height, restarts, print) in PINNED {
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

/// `core.toml` with no `opening_pressure` key, whatever value Wren sets in
/// it (roadmap 2.3): the pins are of 2.1's numbers, before pressure.
fn core_without_pressure() -> String {
    CORE.lines()
        .filter(|l| !l.trim_start().starts_with("opening_pressure"))
        .map(|l| format!("{l}\n"))
        .collect()
}

#[test]
fn two_one_seeds_give_the_mazes_they_gave_on_main() {
    check_pinned(&core_without_pressure());
}

/// Opening pressure at 1 is off (DR-0036 rule 9), so stating it changes
/// nothing (roadmap 2.3).
#[test]
fn opening_pressure_of_1_gives_the_same_mazes() {
    let text = core_without_pressure().replace(
        "name = \"core\"\n",
        "name = \"core\"\nopening_pressure = 1.0\n",
    );
    assert!(text.contains("opening_pressure = 1.0"));
    check_pinned(&text);
}
