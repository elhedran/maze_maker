//! DR-0036 rule 7 (carrying DR-0031 rule 6), the completeness check, as an error when a tile set
//! loads (roadmap 2.6 step 1). The tool's Reload calls `TileSet::from_toml`,
//! so these are the messages its banner shows.

use maze_core::completeness::{check, is_fallback};
use maze_core::{LoadError, SideKind, Table, TileKind, TileSet};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

/// `CORE` with `from` replaced by `to`, which must be there exactly once.
fn edit(from: &str, to: &str) -> String {
    assert_eq!(
        CORE.matches(from).count(),
        1,
        "{from:?} isn't in core.toml once"
    );
    CORE.replace(from, to)
}

/// `CORE` without the tile `id`: from its `[[tile]]` to the next one.
fn without_tile(id: &str) -> String {
    let start = CORE
        .find(&format!("[[tile]]\nid = \"{id}\""))
        .unwrap_or_else(|| panic!("no tile {id}"));
    let end = start
        + 1
        + CORE[start + 1..]
            .find("[[tile]]")
            .expect("not the last tile");
    format!("{}{}", &CORE[..start], &CORE[end..])
}

/// `CORE` with tile `id`'s weight set to `weight`.
fn with_weight(id: &str, weight: &str) -> String {
    let start = CORE.find(&format!("id = \"{id}\"")).expect("tile is there");
    let line = start
        + CORE[start..]
            .find("\nweight = ")
            .expect("tile has a weight")
        + 1;
    let end = line + CORE[line..].find('\n').unwrap();
    format!("{}weight = {weight}{}", &CORE[..line], &CORE[end..])
}

fn errors(text: &str) -> Vec<LoadError> {
    match TileSet::from_toml(text) {
        Ok(_) => panic!("expected the tile set to be refused"),
        Err(errors) => errors,
    }
}

fn messages(errors: &[LoadError]) -> Vec<String> {
    errors.iter().map(|e| e.to_string()).collect()
}

#[test]
fn core_toml_can_make_endless_mazes() {
    let set = TileSet::from_toml(CORE).unwrap_or_else(|e| panic!("core.toml refused: {e:?}"));
    assert_eq!(check(&set, &Table::calculate(&set)), vec![]);
    // Its fallback tiles are its 16 corridor and hedge variants: hedge, 4
    // dead ends, 4 corners, 2 straights, 4 Ts and the crossroads.
    let fallback: Vec<&str> = set
        .variants
        .iter()
        .filter(|v| is_fallback(&set, v))
        .map(|v| set.tile_of(v).id.as_str())
        .collect();
    assert_eq!(fallback.len(), 16);
    for v in &set.variants {
        let kind = set.tile_of(v).kind;
        assert_eq!(is_fallback(&set, v), kind != TileKind::Room, "{v:?}");
    }
}

#[test]
fn a_missing_crossroads_is_refused() {
    let errors = errors(&without_tile("corridor_crossroads"));
    assert_eq!(
        errors,
        vec![LoadError::NoFallbackTile {
            sides: [SideKind::Passage; 4]
        }]
    );
    assert_eq!(
        errors[0].to_string(),
        "No tile of weight above 0 has passage on all four sides (a crossroads). Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs one for every mix of hedge and passage on a cell's four sides (DR-0036 rule 7)."
    );
}

/// Tile weights count too: a dead end of weight 0 is never a candidate, so
/// its four mixes have no tile.
#[test]
fn a_dead_end_of_weight_zero_is_refused() {
    let all = messages(&errors(&with_weight("corridor_dead_end", "0.0")));
    assert_eq!(all.len(), 4, "{all:#?}");
    for side in ["N", "E", "S", "W"] {
        let want = format!("has passage on {side}, hedge on ");
        assert!(
            all.iter()
                .any(|m| m.contains(&want) && m.contains("(a dead end)")),
            "no error for the dead end open to {side}: {all:#?}"
        );
    }
}

/// The demo: Ian sets `passage_passage` to 0 and presses Reload. This is
/// the whole of what the banner says.
#[test]
fn a_passage_passage_base_of_zero_is_refused() {
    let errors = errors(&edit("passage_passage = 1.0", "passage_passage = 0.0"));
    assert_eq!(
        messages(&errors),
        vec![
            "Base passage_passage is 0, so corridor_straight and corridor_straight can't meet passage to passage. Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs every tile's hedge to meet their hedge, and every passage or door to meet their passage, above 0 (DR-0036 rule 7)."
        ]
    );
}

#[test]
fn a_hedge_hedge_base_of_zero_is_refused() {
    let errors = errors(&edit("hedge_hedge = 1.0", "hedge_hedge = 0.0"));
    assert_eq!(errors.len(), 1, "{:#?}", messages(&errors));
    assert!(matches!(
        &errors[0],
        LoadError::FallbackPairIsZero { cause, row_side: SideKind::Hedge, .. }
            if cause == "Base hedge_hedge is 0"
    ));
    let message = errors[0].to_string();
    assert!(message.contains("can't meet hedge to hedge"), "{message}");
    assert!(message.contains("Endless mazes need it"), "{message}");
}

/// Every tile weight is still above 0, but straight can't meet straight:
/// the case tile weights alone would miss (Rowan's DR-0031 note).
#[test]
fn a_straight_to_straight_multiplier_of_zero_is_refused() {
    let errors = errors(&edit("multiplier = 0.5", "multiplier = 0.0"));
    assert_eq!(
        errors,
        vec![LoadError::FallbackPairIsZero {
            cause: "Exception \"Long straights are rarer than turns\" has multiplier 0".into(),
            row: "corridor_straight".into(),
            row_side: SideKind::Passage,
            column: "corridor_straight".into(),
            column_side: SideKind::Passage,
        }]
    );
    assert!(errors[0].to_string().contains("Endless mazes need it"));
}

/// Rule 7's added clause: a stored doorway's door can meet the fallback's
/// passage across the frontier, so a zero there is refused too.
#[test]
fn a_doorway_against_a_dead_end_of_zero_is_refused() {
    let errors = errors(&edit("multiplier = 0.1\n", "multiplier = 0.0\n"));
    assert_eq!(
        errors,
        vec![LoadError::FallbackPairIsZero {
            cause: "Exception \"Doorway rarely faces a dead end\" has multiplier 0".into(),
            row: "room_doorway".into(),
            row_side: SideKind::Door,
            column: "corridor_dead_end".into(),
            column_side: SideKind::Passage,
        }]
    );
}

#[test]
fn a_passage_door_base_of_zero_is_refused() {
    let all = messages(&errors(&edit("passage_door = 1.0", "passage_door = 0.0")));
    assert_eq!(all.len(), 1, "{all:#?}");
    assert!(
        all[0].starts_with("Base passage_door is 0, so room_doorway and "),
        "{}",
        all[0]
    );
    assert!(all[0].contains("can't meet door to passage"), "{}", all[0]);
}

/// The check is rule 7 and no wider: zeros among room tiles, or a tile no
/// one can draw, don't touch the fallback run.
#[test]
fn zeros_outside_the_fallback_tiles_still_load() {
    for text in [
        edit("door_door = 0.05", "door_door = 0.0"),
        edit("floor_floor = 1.0", "floor_floor = 0.0"),
        with_weight("open_room", "0.0"),
        with_weight("room_doorway", "0.0"),
    ] {
        TileSet::from_toml(&text).unwrap_or_else(|e| panic!("refused: {e:?}"));
    }
}

#[test]
fn opening_pressure_must_be_above_0_and_at_most_1() {
    // Any value core.toml has is taken out first: no key means 1.
    let bare: String = CORE
        .lines()
        .filter(|l| !l.trim_start().starts_with("opening_pressure"))
        .map(|l| format!("{l}\n"))
        .collect();
    let with = |v: &str| {
        bare.replace(
            "name = \"core\"\n",
            &format!("name = \"core\"\nopening_pressure = {v}\n"),
        )
    };
    assert_eq!(TileSet::from_toml(&bare).unwrap().opening_pressure, 1.0);
    assert_eq!(
        TileSet::from_toml(&with("0.25")).unwrap().opening_pressure,
        0.25
    );
    assert_eq!(
        TileSet::from_toml(&with("1.0")).unwrap().opening_pressure,
        1.0
    );
    for bad in [
        "0.0",
        "-0.0",
        "-0.5",
        "1.0000001",
        "1.5",
        "inf",
        "-inf",
        "nan",
    ] {
        let all = messages(&errors(&with(bad)));
        assert_eq!(all.len(), 1, "{bad}: {all:#?}");
        assert!(all[0].starts_with("opening_pressure is "), "{}", all[0]);
    }
}

/// DR-0036 rule 9's note of 2026-11-01: `world_opening_pressure` has the
/// same range check, and an absent key means 1 whatever
/// `opening_pressure` says.
#[test]
fn world_opening_pressure_must_be_above_0_and_at_most_1() {
    let bare: String = CORE
        .lines()
        .filter(|l| !l.trim_start().contains("opening_pressure"))
        .map(|l| format!("{l}\n"))
        .collect();
    let with = |keys: &str| bare.replace("name = \"core\"\n", &format!("name = \"core\"\n{keys}"));
    let world = |keys: &str| {
        TileSet::from_toml(&with(keys))
            .unwrap()
            .world_opening_pressure
    };
    assert_eq!(world(""), 1.0);
    assert_eq!(world("opening_pressure = 0.3\n"), 1.0, "no fallback");
    assert_eq!(world("world_opening_pressure = 0.25\n"), 0.25);
    assert_eq!(world("world_opening_pressure = 1.0\n"), 1.0);
    let both = TileSet::from_toml(&with(
        "opening_pressure = 0.5\nworld_opening_pressure = 0.7\n",
    ))
    .unwrap();
    assert_eq!(
        (both.opening_pressure, both.world_opening_pressure),
        (0.5, 0.7)
    );
    for bad in [
        "0.0",
        "-0.0",
        "-0.5",
        "1.0000001",
        "1.5",
        "inf",
        "-inf",
        "nan",
    ] {
        let all = messages(&errors(&with(&format!("world_opening_pressure = {bad}\n"))));
        assert_eq!(all.len(), 1, "{bad}: {all:#?}");
        assert!(
            all[0].starts_with("world_opening_pressure is "),
            "{}",
            all[0]
        );
    }
}

#[test]
fn every_gap_is_reported_at_once() {
    let text = without_tile("corridor_crossroads")
        .replace("passage_passage = 1.0", "passage_passage = 0.0")
        .replace("hedge_hedge = 1.0", "hedge_hedge = 0.0");
    let all = messages(&errors(&text));
    assert_eq!(all.len(), 3, "{all:#?}");
    assert!(all[0].contains("(a crossroads)"));
    assert!(
        all.iter()
            .any(|m| m.starts_with("Base passage_passage is 0"))
    );
    assert!(all.iter().any(|m| m.starts_with("Base hedge_hedge is 0")));
}

/// The check reads the table, so it waits until everything else passes:
/// a file with a bad noise amount shows that one error, as before.
#[test]
fn other_errors_come_first_and_alone() {
    let text = edit("amount = 0.15", "amount = 1.5")
        .replace("passage_passage = 1.0", "passage_passage = 0.0");
    assert_eq!(errors(&text), vec![LoadError::BadNoiseAmount(1.5)]);
}

/// Small fixtures that can't make endless mazes still load for tests of the
/// table and 2.1's restarts, and every other check still runs on them.
#[test]
fn allow_incomplete_skips_only_the_completeness_check() {
    let text = without_tile("corridor_crossroads");
    let set = TileSet::from_toml_allow_incomplete(&text).unwrap();
    assert_eq!(check(&set, &Table::calculate(&set)).len(), 1);
    let bad = text.replace("amount = 0.15", "amount = 1.5");
    assert_eq!(
        TileSet::from_toml_allow_incomplete(&bad).unwrap_err(),
        vec![LoadError::BadNoiseAmount(1.5)]
    );
}
