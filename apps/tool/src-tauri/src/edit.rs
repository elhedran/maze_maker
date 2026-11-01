//! Saving edited numbers into the tile set file (DR-0038 rules 1 to 6).
//!
//! The pieces `ToolState::edit` runs in order: `apply` edits the text with
//! `toml_edit`, so only the edited values change and every comment stays;
//! `validate` puts the result through the loader Reload uses; `write_file`
//! replaces the file through a temporary file and a rename; and
//! `append_record` adds the saved numbers to the sibling edit record.
//! Ranges and the completeness check are the loader's alone: nothing here
//! judges a value.

use crate::ipc::{EditRecord, NumberEdit, NumberTarget, ToolError, ToolErrorKind};
use maze_core::{BasePair, TileSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Value};

/// One number as `apply` changed it, before it is saved and timestamped.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    pub target: NumberTarget,
    /// Rule 4's readable path, e.g. `tile.corridor_corner.weight`.
    pub number: String,
    /// The value in the text before; `None` when the key wasn't there.
    pub from: Option<f64>,
    pub to: f64,
}

impl Applied {
    pub fn record(self, at: &str) -> EditRecord {
        EditRecord {
            at: at.to_string(),
            target: self.target,
            number: self.number,
            from: self.from,
            to: self.to,
        }
    }
}

fn bad_request(message: String) -> ToolError {
    ToolError {
        kind: ToolErrorKind::BadRequest,
        messages: vec![message],
    }
}

/// `text` with each edit applied in turn, and what each changed. `set` is
/// the tile set `text` loads as; it names the exceptions, which have no id
/// in the file. Fails with `bad_request` on no edits or a target that
/// isn't in the set.
pub fn apply(
    text: &str,
    set: &TileSet,
    edits: &[NumberEdit],
) -> Result<(String, Vec<Applied>), ToolError> {
    if edits.is_empty() {
        return Err(bad_request("There is no number to save.".into()));
    }
    // The text loaded as `set`, so it parses; this is the backstop.
    let mut doc: DocumentMut = text
        .parse()
        .map_err(|e| bad_request(format!("The tile set file doesn't parse: {e}")))?;
    let mut applied = Vec::with_capacity(edits.len());
    for edit in edits {
        let number = number_path(set, &edit.target)?;
        let from = match slot(&mut doc, set, &edit.target) {
            Some(item) => {
                let from = number_of(item);
                replace(item, edit.value);
                from
            }
            None if edit.target == NumberTarget::OpeningPressure => {
                insert_opening_pressure(&mut doc, edit.value);
                None
            }
            None => {
                return Err(bad_request(format!(
                    "{number} isn't written in the tile set file as a plain key, so the tool can't change it. Edit it by hand."
                )));
            }
        };
        applied.push(Applied {
            target: edit.target.clone(),
            number,
            from,
            to: edit.value,
        });
    }
    Ok((doc.to_string(), applied))
}

/// Rule 4's readable path for `target`, or `bad_request` if the loaded set
/// has no such number.
fn number_path(set: &TileSet, target: &NumberTarget) -> Result<String, ToolError> {
    match target {
        NumberTarget::TileWeight { tile_id } => {
            if set.tiles.iter().any(|t| &t.id == tile_id) {
                Ok(format!("tile.{tile_id}.weight"))
            } else {
                Err(bad_request(format!(
                    "There is no tile \"{tile_id}\" in the tile set."
                )))
            }
        }
        NumberTarget::Base { pair } => {
            if BasePair::ALL.iter().any(|p| p.key() == pair) {
                Ok(format!("bases.{pair}"))
            } else {
                Err(bad_request(format!(
                    "There is no base \"{pair}\"; bases are {}.",
                    BasePair::ALL.map(BasePair::key).join(", ")
                )))
            }
        }
        NumberTarget::Exception { index } => match set.exceptions.get(*index as usize) {
            Some(e) => Ok(format!(
                "exception.{}.multiplier",
                Value::from(e.name.as_str()).decorated("", "")
            )),
            None => Err(bad_request(format!(
                "There is no exception {index}; the tile set has {}.",
                set.exceptions.len()
            ))),
        },
        NumberTarget::OpeningPressure => Ok("opening_pressure".into()),
    }
}

/// Where `target`'s value sits in `doc`, if it is written there as a
/// plain key. Tiles are found by id; the n-th exception of `set` is the
/// n-th `[[exception]]`, as the loader reads them in file order.
fn slot<'d>(
    doc: &'d mut DocumentMut,
    set: &TileSet,
    target: &NumberTarget,
) -> Option<&'d mut Item> {
    let item = match target {
        NumberTarget::TileWeight { tile_id } => doc
            .get_mut("tile")?
            .as_array_of_tables_mut()?
            .iter_mut()
            .find(|t| t.get("id").and_then(Item::as_str) == Some(tile_id.as_str()))?
            .get_mut("weight")?,
        NumberTarget::Base { pair } => doc.get_mut("bases")?.as_table_like_mut()?.get_mut(pair)?,
        NumberTarget::Exception { index } => {
            debug_assert!((*index as usize) < set.exceptions.len());
            doc.get_mut("exception")?
                .as_array_of_tables_mut()?
                .get_mut(*index as usize)?
                .get_mut("multiplier")?
        }
        NumberTarget::OpeningPressure => doc.get_mut("opening_pressure")?,
    };
    item.is_value().then_some(item)
}

/// The number at `item`, written as a float or an integer.
fn number_of(item: &Item) -> Option<f64> {
    item.as_float()
        .or_else(|| item.as_integer().map(|i| i as f64))
}

/// Puts `value` at `item`, keeping the old value's decor: the space before
/// it and any trailing comment, which `toml_edit` drops on replacing a
/// value (rule 2). The float is written in Rust's shortest round-trip form,
/// with `.0` added to a whole number so it stays a float.
fn replace(item: &mut Item, value: f64) {
    let decor = item.as_value().map(|v| v.decor().clone());
    let mut new = Value::from(value);
    if let Some(decor) = decor {
        *new.decor_mut() = decor;
    }
    *item = Item::Value(new);
}

/// Adds the root `opening_pressure` key. Root keys are written before any
/// table, in insertion order, so it lands straight after the last root key,
/// `name` in `core.toml` (rule 2).
fn insert_opening_pressure(doc: &mut DocumentMut, value: f64) {
    doc.insert("opening_pressure", Item::Value(Value::from(value)));
}

/// Loads `text` exactly as Reload would. `refused`, with every one of the
/// loader's messages, when it fails.
pub fn validate(text: &str) -> Result<TileSet, ToolError> {
    TileSet::from_toml(text).map_err(|errors| ToolError {
        kind: ToolErrorKind::Refused,
        messages: errors.iter().map(|e| e.to_string()).collect(),
    })
}

/// Replaces the file at `path` with `text` through a temporary file beside
/// it and a rename, so a crash never leaves half a file (rule 3.4).
pub fn write_file(path: &Path, text: &str) -> std::io::Result<()> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let temp = path.with_file_name(format!(".{name}.saving"));
    let written = (|| {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temp, path)
    })();
    if written.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    written
}

/// The edit record beside the tile set file: `core.toml` gives
/// `core.edits.toml` (rule 4).
pub fn record_path(path: &Path) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!("{stem}.edits.toml"))
}

/// The header written when the record is created.
fn record_header(tile_set_file: &str) -> String {
    format!(
        "# Numbers changed in the tool in {tile_set_file}, oldest first (DR-0038).\n\
         # To set one back, set it to its \"from\" value, in the tool or in {tile_set_file}.\n"
    )
}

/// One `[[edit]]` per record, each after a blank line.
fn record_text(records: &[EditRecord]) -> String {
    let mut out = String::new();
    for r in records {
        let mut table = toml_edit::Table::new();
        table.insert("at", toml_edit::value(r.at.as_str()));
        table.insert("number", toml_edit::value(r.number.as_str()));
        if let Some(from) = r.from {
            table.insert("from", toml_edit::value(from));
        }
        table.insert("to", toml_edit::value(r.to));
        let mut list = toml_edit::ArrayOfTables::new();
        list.push(table);
        let mut doc = DocumentMut::new();
        doc.insert("edit", Item::ArrayOfTables(list));
        out.push('\n');
        out.push_str(&doc.to_string());
    }
    out
}

/// Appends `records` to the edit record for the tile set at `path`,
/// creating it with its header the first time (rules 4 and 5).
pub fn append_record(path: &Path, records: &[EditRecord]) -> std::io::Result<()> {
    let record = record_path(path);
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record)?;
    let mut text = String::new();
    if file.metadata()?.len() == 0 {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        text.push_str(&record_header(&name));
    }
    text.push_str(&record_text(records));
    file.write_all(text.as_bytes())?;
    file.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    fn core() -> TileSet {
        TileSet::from_toml(CORE).unwrap()
    }

    fn edit(target: NumberTarget, value: f64) -> NumberEdit {
        NumberEdit { target, value }
    }

    fn weight(id: &str) -> NumberTarget {
        NumberTarget::TileWeight { tile_id: id.into() }
    }

    /// The lines of `a` and `b` that differ, as (line number, a, b). Both
    /// must have the same number of lines.
    fn changed_lines<'t>(a: &'t str, b: &'t str) -> Vec<(usize, &'t str, &'t str)> {
        let (a, b): (Vec<&str>, Vec<&str>) = (a.lines().collect(), b.lines().collect());
        assert_eq!(a.len(), b.len(), "line count changed");
        a.iter()
            .zip(&b)
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, (x, y))| (i, *x, *y))
            .collect()
    }

    #[test]
    fn each_target_changes_only_its_value() {
        let set = core();
        let cases = [
            (
                weight("corridor_corner"),
                0.8,
                "weight = 0.8",
                "tile.corridor_corner.weight",
            ),
            (
                NumberTarget::Base {
                    pair: "door_door".into(),
                },
                0.25,
                "door_door = 0.25",
                "bases.door_door",
            ),
            (
                NumberTarget::Exception { index: 1 },
                0.2,
                "multiplier = 0.2",
                "exception.\"Doorway rarely faces a dead end\".multiplier",
            ),
        ];
        for (target, value, line, number) in cases {
            let (text, applied) = apply(CORE, &set, &[edit(target.clone(), value)]).unwrap();
            let diff = changed_lines(CORE, &text);
            assert_eq!(diff.len(), 1, "{number}: {diff:#?}");
            assert_eq!(diff[0].2, line, "{number}");
            assert_eq!(applied.len(), 1);
            assert_eq!(applied[0].number, number);
            assert_eq!(applied[0].to, value);
            assert!(applied[0].from.is_some());
            assert_eq!(validate(&text).unwrap(), {
                let mut s = set.clone();
                match target {
                    NumberTarget::TileWeight { .. } => {
                        s.tiles
                            .iter_mut()
                            .find(|t| t.id == "corridor_corner")
                            .unwrap()
                            .weight = value
                    }
                    NumberTarget::Base { .. } => s.bases.door_door = value,
                    NumberTarget::Exception { index } => {
                        s.exceptions[index as usize].multiplier = value
                    }
                    NumberTarget::OpeningPressure => unreachable!(),
                }
                s
            });
        }
        // The right tile: corridor_corner's weight, not another's.
        let (_, applied) = apply(CORE, &set, &[edit(weight("corridor_corner"), 0.8)]).unwrap();
        let old = set
            .tiles
            .iter()
            .find(|t| t.id == "corridor_corner")
            .unwrap();
        assert_eq!(applied[0].from, Some(old.weight));
    }

    #[test]
    fn opening_pressure_is_inserted_after_name_then_replaced() {
        let set = core();
        assert!(
            !CORE.contains("opening_pressure"),
            "core.toml gained the key; use a fixture"
        );
        let (text, applied) =
            apply(CORE, &set, &[edit(NumberTarget::OpeningPressure, 0.5)]).unwrap();
        assert_eq!(applied[0].from, None);
        assert_eq!(applied[0].number, "opening_pressure");
        assert!(
            text.contains("name = \"core\"\nopening_pressure = 0.5\n"),
            "{}",
            &text[..text.find("[noise]").unwrap()]
        );
        // Everything else is byte for byte as it was.
        assert_eq!(text.replacen("opening_pressure = 0.5\n", "", 1), CORE);
        let pressed = validate(&text).unwrap();
        assert_eq!(pressed.opening_pressure, 0.5);

        // Present: replaced in place; back to 1 writes 1.0 (rule 6).
        let (back, applied) =
            apply(&text, &pressed, &[edit(NumberTarget::OpeningPressure, 1.0)]).unwrap();
        assert_eq!(applied[0].from, Some(0.5));
        assert!(back.contains("\nopening_pressure = 1.0\n"));
        assert_eq!(validate(&back).unwrap(), set);
    }

    #[test]
    fn a_trailing_comment_and_spacing_are_kept() {
        let text = CORE.replace("door_door = 0.3\n", "door_door   =   0.3   # uncommon\n");
        let set = TileSet::from_toml(&text).unwrap();
        let target = NumberTarget::Base {
            pair: "door_door".into(),
        };
        let (out, _) = apply(&text, &set, &[edit(target, 0.45)]).unwrap();
        let diff = changed_lines(&text, &out);
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].2, "door_door   =   0.45   # uncommon");
    }

    #[test]
    fn whole_numbers_stay_floats_and_values_round_trip() {
        let set = core();
        for value in [1.0, 2.0, 0.1, 1.0 / 3.0, 1e-7, 123456.789] {
            let (text, _) = apply(CORE, &set, &[edit(weight("hedge"), value)]).unwrap();
            let loaded = validate(&text).unwrap();
            let hedge = loaded.tiles.iter().find(|t| t.id == "hedge").unwrap();
            assert_eq!(hedge.weight, value, "{value} read back differently");
        }
        let (text, _) = apply(CORE, &set, &[edit(weight("hedge"), 2.0)]).unwrap();
        assert!(changed_lines(CORE, &text)[0].2.ends_with("weight = 2.0"));
    }

    #[test]
    fn an_edit_and_its_set_back_give_the_same_tile_set() {
        let set = core();
        let (text, applied) = apply(CORE, &set, &[edit(weight("open_room"), 0.05)]).unwrap();
        let edited = validate(&text).unwrap();
        assert_ne!(edited, set);
        let from = applied[0].from.unwrap();
        let (back, _) = apply(&text, &edited, &[edit(weight("open_room"), from)]).unwrap();
        assert_eq!(validate(&back).unwrap(), set);
    }

    #[test]
    fn several_edits_apply_in_turn() {
        let set = core();
        let edits = [
            edit(weight("hedge"), 3.0),
            edit(NumberTarget::OpeningPressure, 0.7),
            edit(weight("hedge"), 2.5),
        ];
        let (text, applied) = apply(CORE, &set, &edits).unwrap();
        let froms: Vec<Option<f64>> = applied.iter().map(|a| a.from).collect();
        let hedge = set.tiles.iter().find(|t| t.id == "hedge").unwrap().weight;
        assert_eq!(froms, [Some(hedge), None, Some(3.0)]);
        let loaded = validate(&text).unwrap();
        assert_eq!(
            loaded
                .tiles
                .iter()
                .find(|t| t.id == "hedge")
                .unwrap()
                .weight,
            2.5
        );
        assert_eq!(loaded.opening_pressure, 0.7);
    }

    #[test]
    fn unknown_targets_and_no_edits_are_bad_requests() {
        let set = core();
        let bad = [
            weight("no_such_tile"),
            NumberTarget::Base {
                pair: "hedge_door".into(),
            },
            NumberTarget::Exception { index: 2 },
        ];
        for target in bad {
            let err = apply(CORE, &set, &[edit(target.clone(), 0.5)]).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::BadRequest, "{target:?}");
        }
        let err = apply(CORE, &set, &[]).unwrap_err();
        assert_eq!(err.kind, ToolErrorKind::BadRequest);
    }

    /// The loader decides: these are its refusals, passed through.
    #[test]
    fn values_the_loader_refuses_are_refused() {
        let set = core();
        let base = |pair: &str| NumberTarget::Base { pair: pair.into() };
        for (target, value, says) in [
            (base("passage_passage"), 0.0, "DR-0036 rule 7"),
            (NumberTarget::OpeningPressure, 0.0, "opening_pressure is 0"),
            (
                NumberTarget::OpeningPressure,
                1.5,
                "opening_pressure is 1.5",
            ),
            (weight("hedge"), -1.0, "hedge"),
        ] {
            let (text, _) = apply(CORE, &set, &[edit(target.clone(), value)]).unwrap();
            let err = validate(&text).unwrap_err();
            assert_eq!(err.kind, ToolErrorKind::Refused, "{target:?}");
            assert!(
                err.messages.iter().any(|m| m.contains(says)),
                "{target:?} = {value}: {:?}",
                err.messages
            );
        }
    }

    #[test]
    fn the_record_sits_beside_the_tile_set() {
        assert_eq!(
            record_path(Path::new("data/tile-sets/core.toml")),
            Path::new("data/tile-sets/core.edits.toml")
        );
    }

    #[test]
    fn record_entries_read_back() {
        let records = [
            EditRecord {
                at: "2026-11-03T14:03:22Z".into(),
                target: weight("corridor_corner"),
                number: "tile.corridor_corner.weight".into(),
                from: Some(1.2),
                to: 0.8,
            },
            EditRecord {
                at: "2026-11-03T14:05:00Z".into(),
                target: NumberTarget::OpeningPressure,
                number: "opening_pressure".into(),
                from: None,
                to: 0.5,
            },
            EditRecord {
                at: "2026-11-03T14:06:00Z".into(),
                target: NumberTarget::Exception { index: 0 },
                number: "exception.\"Say \\\"hi\\\"\".multiplier".into(),
                from: Some(0.5),
                to: 1.0,
            },
        ];
        let text = record_text(&records);
        assert!(text.starts_with(
            "\n[[edit]]\nat = \"2026-11-03T14:03:22Z\"\nnumber = \"tile.corridor_corner.weight\"\nfrom = 1.2\nto = 0.8\n"
        ), "{text}");
        let doc: DocumentMut = text.parse().unwrap();
        let list = doc["edit"].as_array_of_tables().unwrap();
        assert_eq!(list.len(), 3);
        assert!(list.get(1).unwrap().get("from").is_none());
        assert_eq!(
            list.get(2).unwrap()["number"].as_str(),
            Some("exception.\"Say \\\"hi\\\"\".multiplier")
        );
        assert_eq!(list.get(2).unwrap()["to"].as_float(), Some(1.0));
    }

    #[test]
    fn number_targets_cross_ipc_as_ts_rs_writes_them() {
        let json = serde_json::to_string(&weight("hedge")).unwrap();
        assert_eq!(json, r#"{"kind":"tileWeight","tileId":"hedge"}"#);
        let json = serde_json::to_string(&NumberTarget::OpeningPressure).unwrap();
        assert_eq!(json, r#"{"kind":"openingPressure"}"#);
        let back: NumberTarget = serde_json::from_str(r#"{"kind":"exception","index":1}"#).unwrap();
        assert_eq!(back, NumberTarget::Exception { index: 1 });
        let back: NumberTarget =
            serde_json::from_str(r#"{"kind":"base","pair":"door_door"}"#).unwrap();
        assert_eq!(
            back,
            NumberTarget::Base {
                pair: "door_door".into()
            }
        );
    }
}
