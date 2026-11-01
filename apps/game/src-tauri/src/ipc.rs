//! IPC types for the game's Tauri commands (DR-0023's pattern, as in the
//! tool).
//!
//! Rust is the source of truth. Each type derives `ts_rs::TS`, and the
//! generated TypeScript is checked in at `apps/game/src/bindings/`. The test
//! at the bottom fails when those files are stale; run it with
//! `UPDATE_BINDINGS=1` to rewrite them.
//!
//! Conventions (DR-0023): camelCase fields, snake_case enum values, no 64-bit
//! integers (ts-rs maps them to `bigint`). These are not `maze-core` types:
//! `game.rs` maps core values into them, so core never sees ts-rs.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A compass direction to step in. North is up the screen (y falls).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl From<Direction> for maze_core::Direction {
    fn from(d: Direction) -> maze_core::Direction {
        match d {
            Direction::North => maze_core::Direction::North,
            Direction::East => maze_core::Direction::East,
            Direction::South => maze_core::Direction::South,
            Direction::West => maze_core::Direction::West,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TileKind {
    Corridor,
    Room,
    Hedge,
}

impl From<maze_core::TileKind> for TileKind {
    fn from(k: maze_core::TileKind) -> TileKind {
        match k {
            maze_core::TileKind::Corridor => TileKind::Corridor,
            maze_core::TileKind::Room => TileKind::Room,
            maze_core::TileKind::Hedge => TileKind::Hedge,
        }
    }
}

/// A world cell: x grows east, y grows south. The origin is where a new
/// world starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

/// One stored cell near the player.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ViewCell {
    pub x: i32,
    pub y: i32,
    /// The tile's id in the tile set, e.g. `corridor_bend`.
    pub tile: String,
    /// Clockwise quarter turns from the tile's rotation 0, 0 to 3.
    pub orientation: u32,
    pub kind: TileKind,
    /// The player has stood on this cell.
    pub visited: bool,
}

/// What the player can see, after `start_game` or a `step`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    pub player: Pos,
    /// False when the step was blocked; nothing else changed then.
    /// Always true from `start_game`.
    pub moved: bool,
    /// Cells within this Chebyshev distance of the player are listed.
    pub radius: u32,
    /// Every stored cell within `radius` of the player, in row-major order
    /// (y, then x). A cell in range that isn't listed is unsettled: draw it
    /// as solid hedge (DR-0036 rule 2).
    pub cells: Vec<ViewCell>,
}

/// A command's error: the promise from `invoke` rejects with this object
/// (DR-0043 point 7). Load errors come one per line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GameError {
    pub message: String,
}

#[cfg(test)]
mod bindings {
    //! The staleness test for `apps/game/src/bindings/` (DR-0023).

    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use ts_rs::Config;

    fn bindings_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("src")
            .join("bindings")
    }

    /// Every exported type, by file name. Add new IPC types here.
    fn expected() -> BTreeMap<String, String> {
        fn render<T: TS + 'static>(cfg: &Config, out: &mut BTreeMap<String, String>) {
            let file = format!("{}.ts", T::ident(cfg));
            let text = T::export_to_string(cfg).expect("ts-rs export");
            out.insert(file, text.replace("\r\n", "\n"));
        }
        let cfg = Config::new();
        let mut out = BTreeMap::new();
        render::<Direction>(&cfg, &mut out);
        render::<TileKind>(&cfg, &mut out);
        render::<Pos>(&cfg, &mut out);
        render::<ViewCell>(&cfg, &mut out);
        render::<GameView>(&cfg, &mut out);
        render::<GameError>(&cfg, &mut out);
        out
    }

    /// The `.ts` files currently in the bindings folder.
    fn on_disk() -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        let Ok(dir) = std::fs::read_dir(bindings_dir()) else {
            return out;
        };
        for entry in dir {
            let path = entry.expect("read bindings dir").path();
            if path.extension().is_some_and(|e| e == "ts") {
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                // Git on Windows may check files out with CRLF.
                let text = std::fs::read_to_string(&path).expect("read binding");
                out.insert(name, text.replace("\r\n", "\n"));
            }
        }
        out
    }

    #[test]
    fn bindings_are_up_to_date() {
        let expected = expected();
        if std::env::var_os("UPDATE_BINDINGS").is_some() {
            let dir = bindings_dir();
            std::fs::create_dir_all(&dir).expect("create bindings dir");
            for stale in on_disk().keys().filter(|k| !expected.contains_key(*k)) {
                std::fs::remove_file(dir.join(stale)).expect("remove stale binding");
            }
            for (name, text) in &expected {
                std::fs::write(dir.join(name), text).expect("write binding");
            }
            return;
        }
        let actual = on_disk();
        let mut problems = Vec::new();
        for (name, text) in &expected {
            match actual.get(name) {
                None => problems.push(format!("missing: {name}")),
                Some(found) if found != text => problems.push(format!("stale: {name}")),
                Some(_) => {}
            }
        }
        for name in actual.keys().filter(|k| !expected.contains_key(*k)) {
            problems.push(format!("left over from a removed type: {name}"));
        }
        assert!(
            problems.is_empty(),
            "apps/game/src/bindings is out of step with ipc.rs; \
             run `UPDATE_BINDINGS=1 cargo test -p maze-game bindings` and commit the result:\n  {}",
            problems.join("\n  ")
        );
    }

    /// The JSON is what the bindings say: camelCase, snake_case values.
    #[test]
    fn serializes_as_the_bindings_say() {
        let view = GameView {
            player: Pos { x: -1, y: 2 },
            moved: true,
            radius: 6,
            cells: vec![ViewCell {
                x: -1,
                y: 2,
                tile: "corridor_bend".into(),
                orientation: 3,
                kind: TileKind::Corridor,
                visited: true,
            }],
        };
        assert_eq!(
            serde_json::to_string(&view).unwrap(),
            r#"{"player":{"x":-1,"y":2},"moved":true,"radius":6,"cells":[{"x":-1,"y":2,"tile":"corridor_bend","orientation":3,"kind":"corridor","visited":true}]}"#
        );
        let d: Direction = serde_json::from_str(r#""west""#).unwrap();
        assert_eq!(d, Direction::West);
        let err = GameError {
            message: "No game has been started.".into(),
        };
        assert_eq!(
            serde_json::to_string(&err).unwrap(),
            r#"{"message":"No game has been started."}"#
        );
    }
}
