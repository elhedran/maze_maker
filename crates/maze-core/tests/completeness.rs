//! DR-0031 rule 6, the completeness check, as an error when a tile set
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
        "No tile of weight above 0 has passage on all four sides (a crossroads). Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs one for every mix of hedge and passage on a cell's four sides (DR-0031 rule 6)."
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
            "Base passage_passage is 0, so corridor_straight and corridor_straight can't meet passage to passage. Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs every hedge-to-hedge and passage-to-passage pairing of them above 0 (DR-0031 rule 6)."
        ]
    );
}

#[test]
fn a_hedge_hedge_base_of_zero_is_refused() {
    let errors = errors(&edit("hedge_hedge = 1.0", "hedge_hedge = 0.0"));
    assert_eq!(errors.len(), 1, "{:#?}", messages(&errors));
    assert!(matches!(
        &errors[0],
        LoadError::FallbackPairIsZero { cause, side: SideKind::Hedge, .. }
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
            column: "corridor_straight".into(),
            side: SideKind::Passage,
        }]
    );
    assert!(errors[0].to_string().contains("Endless mazes need it"));
}

/// The check is rule 6 and no wider: zeros among room tiles, or between a
/// room tile and a corridor, don't touch the fallback run.
#[test]
fn zeros_outside_the_fallback_tiles_still_load() {
    for text in [
        edit("door_door = 0.3", "door_door = 0.0"),
        edit("multiplier = 0.1", "multiplier = 0.0"),
        with_weight("open_room", "0.0"),
        with_weight("room_doorway", "0.0"),
    ] {
        TileSet::from_toml(&text).unwrap_or_else(|e| panic!("refused: {e:?}"));
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
