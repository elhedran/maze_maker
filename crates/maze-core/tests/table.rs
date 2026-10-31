//! Tests of the tile set loader and the table, against the real core file
//! and small fixtures (DR-0015, DR-0022, DR-0024; Phase 1 plan "done when").

use maze_core::{Direction, LoadError, SideKind, Table, TileKind, TileSet};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

fn core() -> TileSet {
    TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml failed to load: {e:?}"))
}

const BASES: &str = r#"
[bases]
hedge_hedge = 1.0
passage_passage = 2.0
passage_door = 1.0
door_door = 0.2
floor_floor = 1.0
floor_wall_floor_wall = 1.0
"#;

const STRAIGHT: &str = r#"
[[tile]]
id = "corridor_straight"
name = "Corridor straight"
kind = "corridor"
tags = []
sides = ["passage", "hedge", "passage", "hedge"]
orientations = 2
weight = 1.5
"#;

const CORNER: &str = r#"
[[tile]]
id = "corridor_corner"
name = "Corridor corner"
kind = "corridor"
tags = ["bend"]
sides = ["passage", "passage", "hedge", "hedge"]
orientations = 4
weight = 3.0
"#;

const HEDGE: &str = r#"
[[tile]]
id = "hedge"
name = "Hedge"
kind = "hedge"
sides = ["hedge", "hedge", "hedge", "hedge"]
orientations = 1
weight = 4.0
"#;

/// A small set: straight, corner, hedge; no noise unless `extra` adds it.
fn fixture(extra: &str) -> String {
    format!("format = 1\nname = \"fixture\"\n{BASES}{STRAIGHT}{CORNER}{HEDGE}{extra}")
}

fn load_err(text: &str) -> Vec<LoadError> {
    match TileSet::from_toml(text) {
        Ok(_) => panic!("expected the tile set to be rejected"),
        Err(errors) => errors,
    }
}

fn messages(errors: &[LoadError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

fn variant_index(set: &TileSet, id: &str, orientation: u32) -> usize {
    set.variants
        .iter()
        .position(|v| set.tiles[v.tile].id == id && v.orientation == orientation)
        .unwrap_or_else(|| panic!("no variant {id}/{orientation}"))
}

// ---- The core file ----

#[test]
fn core_file_has_29_pairs_by_kind() {
    let set = core();
    let count = |kind| {
        set.variants
            .iter()
            .filter(|v| set.tiles[v.tile].kind == kind)
            .count()
    };
    assert_eq!(set.variants.len(), 29);
    assert_eq!(count(TileKind::Corridor), 15);
    assert_eq!(count(TileKind::Room), 13);
    assert_eq!(count(TileKind::Hedge), 1);
}

#[test]
fn core_file_allows_916_entries_229_per_direction() {
    let set = core();
    let table = Table::calculate(&set);
    let n = table.size();
    for d in Direction::ALL {
        let allowed = (0..n)
            .flat_map(|r| (0..n).map(move |c| (r, c)))
            .filter(|&(r, c)| table.entry(d, r, c).allowed)
            .count();
        assert_eq!(allowed, 229, "{d:?}");
    }
}

#[test]
fn core_file_is_in_kind_order_with_fallbacks_last() {
    let set = core();
    let rank = |t: &maze_core::Tile| {
        let kind = match t.kind {
            TileKind::Corridor => 0,
            TileKind::Room => 1,
            TileKind::Hedge => 2,
        };
        (kind, t.tags.iter().any(|tag| tag == "fallback") as u8)
    };
    let ranks: Vec<_> = set.tiles.iter().map(rank).collect();
    let mut sorted = ranks.clone();
    sorted.sort();
    assert_eq!(ranks, sorted);
}

#[test]
fn orientations_follow_dr_0017_order() {
    use SideKind::*;
    let set = core();
    let sides = |id, k| set.variants[variant_index(&set, id, k)].sides;
    // Corner: open N+E, E+S, S+W, W+N.
    assert_eq!(
        sides("corridor_corner", 1),
        [Hedge, Passage, Passage, Hedge]
    );
    // T: closed S, W, N, E.
    assert_eq!(sides("corridor_t_intersection", 1)[3], Hedge);
    // Room wall: wall N, E, S, W; the wall end turns with the tile.
    assert_eq!(
        sides("room_wall", 1),
        [FloorWallE, Hedge, FloorWallE, Floor]
    );
    // Room corner: NE, SE, SW, NW.
    assert_eq!(
        sides("room_corner", 2),
        [FloorWallW, FloorWallS, Hedge, Hedge]
    );
    // Straight has two orientations: N-S then E-W.
    assert_eq!(
        sides("corridor_straight", 1),
        [Hedge, Passage, Hedge, Passage]
    );
}

// ---- The model ----

#[test]
fn symmetric_a_east_of_b_equals_b_west_of_a() {
    for text in [CORE.to_string(), fixture(EXCEPTION)] {
        let set = TileSet::from_toml(&text).unwrap();
        let table = Table::calculate(&set);
        let n = table.size();
        for d in Direction::ALL {
            for r in 0..n {
                for c in 0..n {
                    let here = table.entry(d, r, c);
                    let back = table.entry(d.opposite(), c, r);
                    assert_eq!(here.weight, back.weight, "{d:?} {r} {c}");
                    assert_eq!(here.allowed, back.allowed);
                    assert_eq!(here.exception, back.exception);
                }
            }
        }
    }
}

#[test]
fn rows_sum_to_one_and_never_nan() {
    let set = core();
    let table = Table::calculate(&set);
    for d in Direction::ALL {
        for r in 0..table.size() {
            let sum: f64 = (0..table.size()).map(|c| table.probability(d, r, c)).sum();
            if table.row_total(d, r) > 0.0 {
                assert!((sum - 1.0).abs() < 1e-9, "{d:?} row {r} sums to {sum}");
            }
        }
    }
}

#[test]
fn a_row_with_zero_total_has_probability_zero_not_nan() {
    // A hedge tile of weight 0: every entry in its rows is 0.
    let text = fixture("").replace("weight = 4.0", "weight = 0.0");
    let set = TileSet::from_toml(&text).unwrap();
    let table = Table::calculate(&set);
    let hedge = variant_index(&set, "hedge", 0);
    assert_eq!(table.row_total(Direction::East, hedge), 0.0);
    for c in 0..table.size() {
        assert_eq!(table.probability(Direction::East, hedge, c), 0.0);
    }
}

#[test]
fn side_rules_always_win() {
    // Large weights, a large exception and noise: every entry whose sides
    // may not meet stays exactly 0, and every allowed one is above 0.
    let text = fixture(&format!("{EXCEPTION}\n[noise]\nseed = 3\namount = 0.9\n"))
        .replace("weight = 1.5", "weight = 1000.0")
        .replace("multiplier = 0.5", "multiplier = 1e6");
    let set = TileSet::from_toml(&text).unwrap();
    let table = Table::calculate(&set);
    for d in Direction::ALL {
        for r in 0..table.size() {
            for c in 0..table.size() {
                let e = table.entry(d, r, c);
                assert_eq!(e.allowed, e.row_side.may_meet(e.column_side));
                if e.allowed {
                    assert!(e.weight > 0.0);
                } else {
                    assert_eq!(e.weight, 0.0);
                    assert_eq!(e.base, 0.0);
                    assert_eq!(e.noise, 1.0);
                    assert_eq!(e.exception, None);
                }
            }
        }
    }
}

#[test]
fn same_data_gives_same_table() {
    let a = Table::calculate(&core());
    let b = Table::calculate(&core());
    assert_eq!(a, b);
    // Bit for bit, not just within tolerance.
    for d in Direction::ALL {
        for r in 0..a.size() {
            for c in 0..a.size() {
                assert_eq!(
                    a.entry(d, r, c).weight.to_bits(),
                    b.entry(d, r, c).weight.to_bits()
                );
            }
        }
    }
}

const EXCEPTION: &str = r#"
[[exception]]
name = "Straights rarely run into bends"
a = { tile = "corridor_straight", side = "passage" }
b = { tag = "bend", side = "passage" }
multiplier = 0.5
"#;

#[test]
fn known_entries_by_hand() {
    let set = TileSet::from_toml(&fixture(EXCEPTION)).unwrap();
    let table = Table::calculate(&set);
    let straight_ew = variant_index(&set, "corridor_straight", 1);
    let straight_ns = variant_index(&set, "corridor_straight", 0);
    let corner_sw = variant_index(&set, "corridor_corner", 2); // open S and W
    let hedge = variant_index(&set, "hedge", 0);

    // Corner (open S+W) east of an E-W straight: passage meets passage.
    // 2.0 (base) x 1.5 x 3.0 x 0.5 (exception) = 4.5.
    let e = table.entry(Direction::East, straight_ew, corner_sw);
    assert!(e.allowed);
    assert_eq!(
        (e.row_side, e.column_side),
        (SideKind::Passage, SideKind::Passage)
    );
    assert_eq!(e.base, 2.0);
    assert_eq!((e.row_weight, e.column_weight), (1.5, 3.0));
    assert_eq!(e.exception, Some(0));
    assert_eq!(e.noise, 1.0);
    assert_eq!(e.weight, 4.5);

    // East of the E-W straight, neighbours with a passage on their W side:
    // straight E-W (2 x 1.5 x 1.5 = 4.5), corner open S+W (4.5),
    // corner open W+N (4.5). Total 13.5.
    assert_eq!(table.row_total(Direction::East, straight_ew), 13.5);
    assert!((table.probability(Direction::East, straight_ew, corner_sw) - 1.0 / 3.0).abs() < 1e-12);

    // Straight E-W east of the hedge: hedge meets passage, a clash.
    let zero = table.entry(Direction::East, hedge, straight_ew);
    assert!(!zero.allowed);
    assert_eq!(
        (zero.row_side, zero.column_side),
        (SideKind::Hedge, SideKind::Passage)
    );
    assert_eq!(zero.weight, 0.0);

    // Hedge east of an N-S straight: hedge meets hedge, 1.0 x 1.5 x 4.0.
    assert_eq!(table.entry(Direction::East, straight_ns, hedge).weight, 6.0);
}

#[test]
fn noise_changes_allowed_entries_within_amount() {
    let plain = Table::calculate(&TileSet::from_toml(&fixture("")).unwrap());
    let noisy_set = TileSet::from_toml(&fixture("\n[noise]\nseed = 42\namount = 0.2\n")).unwrap();
    let noisy = Table::calculate(&noisy_set);
    let mut changed = 0;
    for d in Direction::ALL {
        for r in 0..plain.size() {
            for c in 0..plain.size() {
                let (p, q) = (plain.entry(d, r, c), noisy.entry(d, r, c));
                assert!((0.8..=1.2).contains(&q.noise));
                assert!((q.weight - p.weight * q.noise).abs() < 1e-12);
                if q.noise != 1.0 {
                    changed += 1;
                }
            }
        }
    }
    assert!(changed > 0);
}

// ---- Validation: every DR-0022 condition 3 case, plus DR-0024's ----

fn assert_rejects(text: &str, culprit: &str) {
    let errors = load_err(text);
    let all = messages(&errors);
    assert!(
        all.contains(culprit),
        "expected an error naming {culprit:?}, got:\n{all}"
    );
}

#[test]
fn rejects_unknown_tile_in_exception() {
    let text =
        fixture(&EXCEPTION.replace("tile = \"corridor_straight\"", "tile = \"corridor_strait\""));
    assert_rejects(&text, "corridor_strait");
}

#[test]
fn rejects_unknown_tag_in_exception() {
    assert_rejects(
        &fixture(&EXCEPTION.replace("tag = \"bend\"", "tag = \"bent\"")),
        "bent",
    );
}

#[test]
fn rejects_negative_and_non_finite_numbers() {
    assert_rejects(
        &fixture("").replace("weight = 3.0", "weight = -1.0"),
        "corridor_corner",
    );
    assert_rejects(
        &fixture("").replace("door_door = 0.2", "door_door = nan"),
        "door_door",
    );
    assert_rejects(
        &fixture("").replace("passage_passage = 2.0", "passage_passage = inf"),
        "passage_passage",
    );
    assert_rejects(
        &fixture(&EXCEPTION.replace("multiplier = 0.5", "multiplier = -0.5")),
        "Straights rarely run into bends",
    );
}

#[test]
fn rejects_exception_matching_no_allowed_pair() {
    // A straight's passage never meets a hedge side.
    let text = fixture(&EXCEPTION.replace(
        "tag = \"bend\", side = \"passage\"",
        "tile = \"hedge\", side = \"hedge\"",
    ));
    let errors = load_err(&text);
    assert!(
        matches!(&errors[..], [LoadError::NoAllowedPair { exception }] if exception == "Straights rarely run into bends")
    );
}

#[test]
fn rejects_two_exceptions_on_one_pair() {
    let second = EXCEPTION
        .replace("Straights rarely run into bends", "Second rule")
        .replace("multiplier = 0.5", "multiplier = 2.0");
    let all = messages(&load_err(&fixture(&format!("{EXCEPTION}{second}"))));
    assert!(
        all.contains("Straights rarely run into bends") && all.contains("Second rule"),
        "{all}"
    );
}

#[test]
fn rejects_exception_end_with_both_or_neither_selector() {
    let both = EXCEPTION.replace("tag = \"bend\",", "tag = \"bend\", tile = \"hedge\",");
    assert_rejects(&fixture(&both), "exactly one of");
}

#[test]
fn rejects_unknown_keys_and_missing_bases() {
    assert_rejects(
        &fixture("").replace("weight = 4.0", "weight = 4.0\ncolour = \"green\""),
        "colour",
    );
    assert_rejects(&fixture("").replace("door_door = 0.2\n", ""), "door_door");
    // A base for a pair that may not meet.
    assert_rejects(
        &fixture("").replace("[bases]", "[bases]\npassage_floor = 1.0"),
        "passage_floor",
    );
}

#[test]
fn rejects_wrong_orientation_count_and_duplicate_ids() {
    assert_rejects(
        &fixture("").replace("orientations = 2", "orientations = 4"),
        "corridor_straight",
    );
    assert_rejects(&format!("{}{HEDGE}", fixture("")), "defined more than once");
}

#[test]
fn rejects_bad_noise_and_format() {
    assert_rejects(
        &fixture("\n[noise]\nseed = 1\namount = 1.0\n"),
        "Noise amount",
    );
    assert_rejects(
        &fixture("").replace("format = 1", "format = 2"),
        "format = 2",
    );
    assert_rejects(&fixture("\n[noise]\nseed = -1\namount = 0.1\n"), "seed");
}

#[test]
fn reports_every_error_at_once() {
    let text = fixture("")
        .replace("weight = 3.0", "weight = -1.0")
        .replace("orientations = 2", "orientations = 3");
    assert_eq!(load_err(&text).len(), 2);
}

#[test]
fn rejects_bad_toml_with_a_readable_message() {
    let all = messages(&load_err("format = 1\nname = "));
    assert!(all.starts_with("The file doesn't parse"), "{all}");
}
