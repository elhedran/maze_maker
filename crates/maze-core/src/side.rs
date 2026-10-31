//! Directions on the square grid (DR-0014) and the five kinds of side, with
//! DR-0015's rules for which sides may meet.
//!
//! Which sides may meet is code, not data (DR-0024): these are DR-0015's
//! invariants, and no weight, exception or noise may override them (DR-0022).

use serde::Deserialize;
use std::fmt;

/// A compass direction. Also the order of a tile's sides: N, E, S, W.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    /// All four, in side order.
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    /// 0 for North up to 3 for West.
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(index: usize) -> Direction {
        Direction::ALL[index % 4]
    }

    pub fn opposite(self) -> Direction {
        Direction::from_index(self.index() + 2)
    }

    /// The direction one clockwise quarter turn on.
    pub fn clockwise(self) -> Direction {
        Direction::from_index(self.index() + 1)
    }

    /// One capital letter, as in DR-0015's "wall at N end".
    pub fn letter(self) -> char {
        ['N', 'E', 'S', 'W'][self.index()]
    }
}

/// The kind of a tile's side (DR-0015). `FloorWall(d)` is floor with the
/// room's wall reaching this side at its `d` end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideKind {
    Hedge,
    Passage,
    Door,
    Floor,
    FloorWallN,
    FloorWallE,
    FloorWallS,
    FloorWallW,
}

impl SideKind {
    pub const ALL: [SideKind; 8] = [
        SideKind::Hedge,
        SideKind::Passage,
        SideKind::Door,
        SideKind::Floor,
        SideKind::FloorWallN,
        SideKind::FloorWallE,
        SideKind::FloorWallS,
        SideKind::FloorWallW,
    ];

    /// The end the wall reaches, for the floor-with-wall kinds.
    pub fn wall_end(self) -> Option<Direction> {
        match self {
            SideKind::FloorWallN => Some(Direction::North),
            SideKind::FloorWallE => Some(Direction::East),
            SideKind::FloorWallS => Some(Direction::South),
            SideKind::FloorWallW => Some(Direction::West),
            _ => None,
        }
    }

    fn floor_wall(end: Direction) -> SideKind {
        match end {
            Direction::North => SideKind::FloorWallN,
            Direction::East => SideKind::FloorWallE,
            Direction::South => SideKind::FloorWallS,
            Direction::West => SideKind::FloorWallW,
        }
    }

    /// This side after one clockwise quarter turn of its tile: a wall end
    /// turns with the tile (DR-0024).
    pub fn rotated(self) -> SideKind {
        match self.wall_end() {
            Some(end) => SideKind::floor_wall(end.clockwise()),
            None => self,
        }
    }

    /// DR-0015's rules: hedge meets hedge; passage meets passage or door;
    /// door meets passage or door; floor meets floor; floor with a wall meets
    /// the same kind with the wall at the same end.
    pub fn may_meet(self, other: SideKind) -> bool {
        self.base_pair(other).is_some()
    }

    /// Every side kind this one may meet, in `SideKind::ALL` order.
    pub fn may_meet_list(self) -> Vec<SideKind> {
        SideKind::ALL
            .into_iter()
            .filter(|&other| self.may_meet(other))
            .collect()
    }

    /// The side-pair base this meeting uses, or `None` if the sides may not
    /// meet. This is the single place the side rules live.
    pub fn base_pair(self, other: SideKind) -> Option<BasePair> {
        use SideKind::*;
        match (self, other) {
            (Hedge, Hedge) => Some(BasePair::HedgeHedge),
            (Passage, Passage) => Some(BasePair::PassagePassage),
            (Passage, Door) | (Door, Passage) => Some(BasePair::PassageDoor),
            (Door, Door) => Some(BasePair::DoorDoor),
            (Floor, Floor) => Some(BasePair::FloorFloor),
            (a, b) if a.wall_end().is_some() && a == b => Some(BasePair::FloorWallFloorWall),
            _ => None,
        }
    }

    /// The name used in the data file, e.g. `floor_wall_n`.
    pub fn key(self) -> &'static str {
        match self {
            SideKind::Hedge => "hedge",
            SideKind::Passage => "passage",
            SideKind::Door => "door",
            SideKind::Floor => "floor",
            SideKind::FloorWallN => "floor_wall_n",
            SideKind::FloorWallE => "floor_wall_e",
            SideKind::FloorWallS => "floor_wall_s",
            SideKind::FloorWallW => "floor_wall_w",
        }
    }
}

/// DR-0015's names, e.g. "Floor, wall at N end".
impl fmt::Display for SideKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.wall_end() {
            Some(end) => write!(f, "Floor, wall at {} end", end.letter()),
            None => f.write_str(match self {
                SideKind::Hedge => "Hedge",
                SideKind::Passage => "Passage",
                SideKind::Door => "Door",
                _ => "Floor",
            }),
        }
    }
}

/// A pair of side kinds that may meet. Each has one base in the data file;
/// bases ignore which end a wall is at (DR-0024).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BasePair {
    HedgeHedge,
    PassagePassage,
    PassageDoor,
    DoorDoor,
    FloorFloor,
    FloorWallFloorWall,
}

impl BasePair {
    pub const ALL: [BasePair; 6] = [
        BasePair::HedgeHedge,
        BasePair::PassagePassage,
        BasePair::PassageDoor,
        BasePair::DoorDoor,
        BasePair::FloorFloor,
        BasePair::FloorWallFloorWall,
    ];

    /// The key under `[bases]` in the data file.
    pub fn key(self) -> &'static str {
        match self {
            BasePair::HedgeHedge => "hedge_hedge",
            BasePair::PassagePassage => "passage_passage",
            BasePair::PassageDoor => "passage_door",
            BasePair::DoorDoor => "door_door",
            BasePair::FloorFloor => "floor_floor",
            BasePair::FloorWallFloorWall => "floor_wall_floor_wall",
        }
    }
}

/// The side an exception matches on: any side kind, or `floor_wall` for a
/// floor with a wall at either end (DR-0024).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SideMatch {
    Hedge,
    Passage,
    Door,
    Floor,
    FloorWall,
    FloorWallN,
    FloorWallE,
    FloorWallS,
    FloorWallW,
}

impl SideMatch {
    pub fn matches(self, side: SideKind) -> bool {
        match self {
            SideMatch::Hedge => side == SideKind::Hedge,
            SideMatch::Passage => side == SideKind::Passage,
            SideMatch::Door => side == SideKind::Door,
            SideMatch::Floor => side == SideKind::Floor,
            SideMatch::FloorWall => side.wall_end().is_some(),
            SideMatch::FloorWallN => side == SideKind::FloorWallN,
            SideMatch::FloorWallE => side == SideKind::FloorWallE,
            SideMatch::FloorWallS => side == SideKind::FloorWallS,
            SideMatch::FloorWallW => side == SideKind::FloorWallW,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            SideMatch::FloorWall => "floor_wall",
            SideMatch::Hedge => "hedge",
            SideMatch::Passage => "passage",
            SideMatch::Door => "door",
            SideMatch::Floor => "floor",
            SideMatch::FloorWallN => "floor_wall_n",
            SideMatch::FloorWallE => "floor_wall_e",
            SideMatch::FloorWallS => "floor_wall_s",
            SideMatch::FloorWallW => "floor_wall_w",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_rules_match_dr_0015() {
        use SideKind::*;
        assert!(Hedge.may_meet(Hedge));
        assert!(Passage.may_meet(Door) && Door.may_meet(Passage));
        assert!(Door.may_meet(Door));
        assert!(Floor.may_meet(Floor));
        assert!(FloorWallN.may_meet(FloorWallN));
        assert!(!FloorWallN.may_meet(FloorWallS));
        assert!(!Passage.may_meet(Hedge));
        assert!(!Door.may_meet(Floor));
        assert!(!Floor.may_meet(FloorWallN));
        assert!(!Passage.may_meet(FloorWallN));
        // Symmetric for every pair.
        for a in SideKind::ALL {
            for b in SideKind::ALL {
                assert_eq!(a.may_meet(b), b.may_meet(a), "{a:?} {b:?}");
            }
        }
    }

    #[test]
    fn wall_end_turns_with_the_tile() {
        assert_eq!(SideKind::FloorWallN.rotated(), SideKind::FloorWallE);
        assert_eq!(SideKind::FloorWallW.rotated(), SideKind::FloorWallN);
        assert_eq!(SideKind::Door.rotated(), SideKind::Door);
    }

    #[test]
    fn names_follow_dr_0015() {
        assert_eq!(SideKind::FloorWallN.to_string(), "Floor, wall at N end");
        assert_eq!(SideKind::Passage.to_string(), "Passage");
    }
}
