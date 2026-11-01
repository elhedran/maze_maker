//! The tile set: tiles, side-pair bases, tile weights, exceptions and noise
//! (DR-0015, DR-0022), parsed and validated from the TOML of DR-0024.
//!
//! `TileSet::from_toml` takes a string; reading the file is the caller's job
//! (DR-0012: no I/O in core). Every validation error is collected and
//! returned together, each naming the tile, base or exception at fault.

use crate::completeness::{self, describe_sides};
use crate::side::{BasePair, Direction, SideKind, SideMatch};
use crate::table::Table;
use serde::Deserialize;
use std::collections::HashSet;
use std::fmt;

/// The only file format version this code reads.
pub const FORMAT: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TileKind {
    Corridor,
    Room,
    Hedge,
}

/// One base per pair of side kinds that may meet. All six are required, and
/// any other key is a parse error, so a base for a pair that may not meet
/// can't be written.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bases {
    pub hedge_hedge: f64,
    pub passage_passage: f64,
    pub passage_door: f64,
    pub door_door: f64,
    pub floor_floor: f64,
    pub floor_wall_floor_wall: f64,
}

impl Bases {
    pub fn get(&self, pair: BasePair) -> f64 {
        match pair {
            BasePair::HedgeHedge => self.hedge_hedge,
            BasePair::PassagePassage => self.passage_passage,
            BasePair::PassageDoor => self.passage_door,
            BasePair::DoorDoor => self.door_door,
            BasePair::FloorFloor => self.floor_floor,
            BasePair::FloorWallFloorWall => self.floor_wall_floor_wall,
        }
    }
}

/// Seeded variation on every allowed entry (DR-0024).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Noise {
    pub seed: u32,
    /// In [0, 1). Each allowed entry gets a factor in [1 - amount, 1 + amount].
    pub amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tile {
    /// The tile type, snake_case and unique.
    pub id: String,
    pub name: String,
    pub kind: TileKind,
    pub tags: Vec<String>,
    /// N, E, S, W at rotation 0.
    pub sides: [SideKind; 4],
    pub weight: f64,
}

/// A (tile, orientation): one row and one column of the table.
#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    /// Index into `TileSet::tiles`.
    pub tile: usize,
    /// Clockwise quarter turns from rotation 0.
    pub orientation: u32,
    /// N, E, S, W in this orientation.
    pub sides: [SideKind; 4],
}

impl Variant {
    pub fn side(&self, direction: Direction) -> SideKind {
        self.sides[direction.index()]
    }
}

/// Which tiles one end of an exception matches.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    Tile(String),
    Tag(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Matcher {
    pub selector: Selector,
    pub side: SideMatch,
}

impl Matcher {
    pub fn matches(&self, tile: &Tile, side: SideKind) -> bool {
        let tile_matches = match &self.selector {
            Selector::Tile(id) => &tile.id == id,
            Selector::Tag(tag) => tile.tags.contains(tag),
        };
        tile_matches && self.side.matches(side)
    }
}

impl fmt::Display for Matcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.selector {
            Selector::Tile(id) => write!(f, "tile {id} ({})", self.side.key()),
            Selector::Tag(tag) => write!(f, "tag {tag} ({})", self.side.key()),
        }
    }
}

/// A multiplier for particular pairs (DR-0022). Applies both ways.
#[derive(Clone, Debug, PartialEq)]
pub struct Exception {
    pub name: String,
    pub a: Matcher,
    pub b: Matcher,
    pub multiplier: f64,
}

impl Exception {
    /// Whether this exception applies where `row`'s `row_side` meets
    /// `column`'s `column_side`, either way round.
    pub fn applies(
        &self,
        row: &Tile,
        row_side: SideKind,
        column: &Tile,
        column_side: SideKind,
    ) -> bool {
        (self.a.matches(row, row_side) && self.b.matches(column, column_side))
            || (self.a.matches(column, column_side) && self.b.matches(row, row_side))
    }

    /// The rule as written in the data, for the explanation.
    pub fn rule(&self) -> String {
        format!("{} beside {}", self.a, self.b)
    }
}

/// A loaded, validated tile set.
#[derive(Clone, Debug, PartialEq)]
pub struct TileSet {
    pub name: String,
    pub noise: Option<Noise>,
    pub bases: Bases,
    pub tiles: Vec<Tile>,
    /// Every (tile, orientation) in table order: file order, then orientation.
    pub variants: Vec<Variant>,
    pub exceptions: Vec<Exception>,
}

/// Why a tile set was rejected. `Display` gives the readable message.
#[derive(Clone, Debug, PartialEq)]
pub enum LoadError {
    /// The TOML didn't parse or didn't fit the format (including unknown
    /// keys and missing bases).
    Parse(String),
    UnsupportedFormat(u32),
    NoTiles,
    BadId {
        what: &'static str,
        id: String,
    },
    DuplicateTile(String),
    BadNumber {
        what: String,
        value: f64,
    },
    BadNoiseAmount(f64),
    OrientationsMismatch {
        tile: String,
        declared: u32,
        distinct: u32,
    },
    /// An exception end names both or neither of `tile` and `tag`.
    BadSelector {
        exception: String,
    },
    UnknownTile {
        exception: String,
        tile: String,
    },
    UnknownTag {
        exception: String,
        tag: String,
    },
    NoAllowedPair {
        exception: String,
    },
    Overlap {
        first: String,
        second: String,
        example: String,
    },
    /// DR-0031 rule 6: no fallback tile (weight above 0, every side hedge
    /// or passage) has exactly these sides.
    NoFallbackTile {
        sides: [SideKind; 4],
    },
    /// DR-0031 rule 6: two fallback tiles whose facing sides are both
    /// `side` have a table entry weight of 0. One per cause; `row` and
    /// `column` are the first such pair in table order.
    FallbackPairIsZero {
        cause: String,
        row: String,
        column: String,
        side: SideKind,
    },
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Parse(msg) => write!(f, "The file doesn't parse: {msg}"),
            LoadError::UnsupportedFormat(n) => write!(
                f,
                "format = {n} isn't supported; this build reads format = {FORMAT}."
            ),
            LoadError::NoTiles => write!(f, "The tile set has no tiles."),
            LoadError::BadId { what, id } => write!(
                f,
                "{what} \"{id}\" must be snake_case: lower-case letters, digits and _, not empty."
            ),
            LoadError::DuplicateTile(id) => write!(f, "Tile \"{id}\" is defined more than once."),
            LoadError::BadNumber { what, value } => write!(
                f,
                "{what} is {value}; it must be a finite number, 0 or more."
            ),
            LoadError::BadNoiseAmount(v) => write!(
                f,
                "Noise amount is {v}; it must be at least 0 and less than 1."
            ),
            LoadError::OrientationsMismatch {
                tile,
                declared,
                distinct,
            } => write!(
                f,
                "Tile \"{tile}\" says orientations = {declared}, but its sides have {distinct} distinct rotations."
            ),
            LoadError::BadSelector { exception } => write!(
                f,
                "Exception \"{exception}\": each end needs exactly one of tile = ... or tag = ...."
            ),
            LoadError::UnknownTile { exception, tile } => write!(
                f,
                "Exception \"{exception}\" names tile \"{tile}\", which isn't in the tile set."
            ),
            LoadError::UnknownTag { exception, tag } => write!(
                f,
                "Exception \"{exception}\" names tag \"{tag}\", which no tile carries."
            ),
            LoadError::NoAllowedPair { exception } => write!(
                f,
                "Exception \"{exception}\" matches no pair of sides that may meet, so it would never apply."
            ),
            LoadError::Overlap {
                first,
                second,
                example,
            } => write!(
                f,
                "Exceptions \"{first}\" and \"{second}\" both match the same pair ({example}); at most one exception may apply to a pair."
            ),
            LoadError::NoFallbackTile { sides } => write!(
                f,
                "No tile of weight above 0 has {}. Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs one for every mix of hedge and passage on a cell's four sides (DR-0031 rule 6).",
                describe_sides(*sides)
            ),
            LoadError::FallbackPairIsZero {
                cause,
                row,
                column,
                side,
            } => write!(
                f,
                "{cause}, so {row} and {column} can't meet {side} to {side}. Endless mazes need it: their fallback run uses only hedge and corridor tiles, and needs every hedge-to-hedge and passage-to-passage pairing of them above 0 (DR-0031 rule 6).",
                side = side.key()
            ),
        }
    }
}

impl std::error::Error for LoadError {}

// The file as written. Kept private: the public model is validated.

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTileSet {
    format: u32,
    name: String,
    noise: Option<Noise>,
    bases: Bases,
    #[serde(default, rename = "tile")]
    tiles: Vec<RawTile>,
    #[serde(default, rename = "exception")]
    exceptions: Vec<RawException>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawTile {
    id: String,
    name: String,
    kind: TileKind,
    #[serde(default)]
    tags: Vec<String>,
    sides: [SideKind; 4],
    orientations: u32,
    weight: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawException {
    name: String,
    a: RawMatcher,
    b: RawMatcher,
    multiplier: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMatcher {
    tile: Option<String>,
    tag: Option<String>,
    side: SideMatch,
}

fn is_snake_case(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn is_good_number(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

/// The distinct clockwise rotations of `sides`, lowest turn count first.
pub fn distinct_rotations(sides: [SideKind; 4]) -> Vec<(u32, [SideKind; 4])> {
    let mut out: Vec<(u32, [SideKind; 4])> = Vec::new();
    let mut current = sides;
    for k in 0..4 {
        if !out.iter().any(|(_, s)| *s == current) {
            out.push((k, current));
        }
        // One clockwise quarter turn: the side facing d moves to face d+1,
        // and a wall end turns with it.
        let mut next = current;
        for d in Direction::ALL {
            next[d.clockwise().index()] = current[d.index()].rotated();
        }
        current = next;
    }
    out
}

impl TileSet {
    /// Parses and validates a tile set (DR-0024), and checks it can make
    /// endless mazes (DR-0031 rule 6, `completeness::check`). The tool's
    /// Reload comes through here, so its banner shows every error. On
    /// failure, returns every problem found; the completeness check reads
    /// the table, so it runs only once everything else passes.
    pub fn from_toml(text: &str) -> Result<TileSet, Vec<LoadError>> {
        let set = TileSet::from_toml_allow_incomplete(text)?;
        let errors = completeness::check(&set, &Table::calculate(&set));
        if errors.is_empty() {
            Ok(set)
        } else {
            Err(errors)
        }
    }

    /// As `from_toml`, without DR-0031's completeness check: for small sets
    /// that can't make endless mazes and aren't meant to, such as test
    /// fixtures for the table or for 2.1's restarts. Every other check
    /// still runs. The tool and the game load with `from_toml`.
    pub fn from_toml_allow_incomplete(text: &str) -> Result<TileSet, Vec<LoadError>> {
        let raw: RawTileSet =
            toml::from_str(text).map_err(|e| vec![LoadError::Parse(e.to_string())])?;
        let mut errors = Vec::new();

        if raw.format != FORMAT {
            errors.push(LoadError::UnsupportedFormat(raw.format));
        }
        if let Some(noise) = raw.noise
            && !(noise.amount.is_finite() && (0.0..1.0).contains(&noise.amount))
        {
            errors.push(LoadError::BadNoiseAmount(noise.amount));
        }
        for pair in BasePair::ALL {
            let value = raw.bases.get(pair);
            if !is_good_number(value) {
                errors.push(LoadError::BadNumber {
                    what: format!("Base {}", pair.key()),
                    value,
                });
            }
        }

        if raw.tiles.is_empty() {
            errors.push(LoadError::NoTiles);
        }
        let mut tiles = Vec::new();
        let mut variants = Vec::new();
        let mut seen = HashSet::new();
        for raw_tile in raw.tiles {
            if !is_snake_case(&raw_tile.id) {
                errors.push(LoadError::BadId {
                    what: "Tile id",
                    id: raw_tile.id.clone(),
                });
            }
            if !seen.insert(raw_tile.id.clone()) {
                errors.push(LoadError::DuplicateTile(raw_tile.id.clone()));
            }
            for tag in &raw_tile.tags {
                if !is_snake_case(tag) {
                    errors.push(LoadError::BadId {
                        what: "Tag",
                        id: tag.clone(),
                    });
                }
            }
            if !is_good_number(raw_tile.weight) {
                errors.push(LoadError::BadNumber {
                    what: format!("Tile \"{}\" weight", raw_tile.id),
                    value: raw_tile.weight,
                });
            }
            let rotations = distinct_rotations(raw_tile.sides);
            if rotations.len() as u32 != raw_tile.orientations {
                errors.push(LoadError::OrientationsMismatch {
                    tile: raw_tile.id.clone(),
                    declared: raw_tile.orientations,
                    distinct: rotations.len() as u32,
                });
            }
            let index = tiles.len();
            variants.extend(rotations.into_iter().map(|(k, sides)| Variant {
                tile: index,
                orientation: k,
                sides,
            }));
            tiles.push(Tile {
                id: raw_tile.id,
                name: raw_tile.name,
                kind: raw_tile.kind,
                tags: raw_tile.tags,
                sides: raw_tile.sides,
                weight: raw_tile.weight,
            });
        }

        let mut exceptions = Vec::new();
        for raw_ex in raw.exceptions {
            if !is_good_number(raw_ex.multiplier) {
                errors.push(LoadError::BadNumber {
                    what: format!("Exception \"{}\" multiplier", raw_ex.name),
                    value: raw_ex.multiplier,
                });
            }
            let mut matcher = |end: RawMatcher| -> Option<Matcher> {
                let selector = match (end.tile, end.tag) {
                    (Some(tile), None) => {
                        if !tiles.iter().any(|t| t.id == tile) {
                            errors.push(LoadError::UnknownTile {
                                exception: raw_ex.name.clone(),
                                tile: tile.clone(),
                            });
                        }
                        Selector::Tile(tile)
                    }
                    (None, Some(tag)) => {
                        if !tiles.iter().any(|t| t.tags.contains(&tag)) {
                            errors.push(LoadError::UnknownTag {
                                exception: raw_ex.name.clone(),
                                tag: tag.clone(),
                            });
                        }
                        Selector::Tag(tag)
                    }
                    _ => {
                        errors.push(LoadError::BadSelector {
                            exception: raw_ex.name.clone(),
                        });
                        return None;
                    }
                };
                Some(Matcher {
                    selector,
                    side: end.side,
                })
            };
            let a = matcher(raw_ex.a);
            let b = matcher(raw_ex.b);
            // An end with a bad selector is already reported; drop the
            // exception so the coverage check doesn't report it twice.
            if let (Some(a), Some(b)) = (a, b) {
                exceptions.push(Exception {
                    name: raw_ex.name,
                    a,
                    b,
                    multiplier: raw_ex.multiplier,
                });
            }
        }

        let set = TileSet {
            name: raw.name,
            noise: raw.noise,
            bases: raw.bases,
            tiles,
            variants,
            exceptions,
        };
        errors.extend(set.check_exception_coverage());

        if errors.is_empty() {
            Ok(set)
        } else {
            Err(errors)
        }
    }

    /// Each exception must apply to at least one allowed pair, and no
    /// allowed pair may have two (DR-0022 condition 3). Exceptions apply
    /// both ways and ignore direction, so checking one direction of each
    /// facing pair per variant pair covers every case.
    fn check_exception_coverage(&self) -> Vec<LoadError> {
        let mut used = vec![false; self.exceptions.len()];
        let mut overlaps: Vec<(usize, usize, String)> = Vec::new();
        for row in &self.variants {
            for column in &self.variants {
                for d in Direction::ALL {
                    let row_side = row.side(d);
                    let column_side = column.side(d.opposite());
                    if !row_side.may_meet(column_side) {
                        continue;
                    }
                    let row_tile = &self.tiles[row.tile];
                    let column_tile = &self.tiles[column.tile];
                    let applying: Vec<usize> = (0..self.exceptions.len())
                        .filter(|&i| {
                            self.exceptions[i].applies(row_tile, row_side, column_tile, column_side)
                        })
                        .collect();
                    for &i in &applying {
                        used[i] = true;
                    }
                    for (n, &i) in applying.iter().enumerate() {
                        for &j in &applying[n + 1..] {
                            if !overlaps.iter().any(|(x, y, _)| (*x, *y) == (i, j)) {
                                let example = format!(
                                    "{} {} beside {} {}",
                                    row_tile.id,
                                    row_side.key(),
                                    column_tile.id,
                                    column_side.key()
                                );
                                overlaps.push((i, j, example));
                            }
                        }
                    }
                }
            }
        }
        let mut errors: Vec<LoadError> = used
            .iter()
            .enumerate()
            .filter(|(_, used)| !**used)
            .map(|(i, _)| LoadError::NoAllowedPair {
                exception: self.exceptions[i].name.clone(),
            })
            .collect();
        errors.extend(
            overlaps
                .into_iter()
                .map(|(i, j, example)| LoadError::Overlap {
                    first: self.exceptions[i].name.clone(),
                    second: self.exceptions[j].name.clone(),
                    example,
                }),
        );
        errors
    }

    pub fn tile_of(&self, variant: &Variant) -> &Tile {
        &self.tiles[variant.tile]
    }
}
