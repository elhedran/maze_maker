//! One game: the stored world (DR-0035), the player in it, and where they
//! have been. Plain Rust, so tests drive it without Tauri; `state.rs` loads
//! the tile set and `lib.rs` wraps the state for Tauri.
//!
//! The world grows from open endpoints near the player (DR-0036): after
//! every move `World::grow_around` the new cell, so walking is what grows
//! it. Cells are stored once and never change, so walking back finds them
//! as they were.

use crate::ipc::{self, GameError, GameView, ViewCell};
use maze_core::world::{self, Pos, Settings, World};
use maze_core::{Direction, LoadError, Table, TileSet};
use std::collections::BTreeSet;

/// How far the player sees, Chebyshev: the view lists stored cells up to
/// this far away (DR-0044).
pub const VIEW_RADIUS: i64 = 6;

/// DR-0036 rule 2 and DR-0044: the growth radius r must be more than the
/// player can see, or cells in view would fill in as the player
/// approaches. The game grows with maze-core's default settings (r = 8).
pub fn settings() -> Settings {
    Settings::default()
}
const _: () = assert!(0 < VIEW_RADIUS && VIEW_RADIUS < world::RADIUS);

pub struct Game {
    world: World,
    player: Pos,
    visited: BTreeSet<Pos>,
}

impl Game {
    /// A new game: the world for `seed`, grown until no anchor is within r
    /// of the origin, with the player standing there (DR-0044). DR-0036 rule 2 masks the origin to tiles with
    /// a walkable side and keeps only cells joined to it, so the player
    /// spawns on a walkable cell with a way onward. Fails only if the tile
    /// set fails the completeness check; one from `TileSet::from_toml` has
    /// passed it.
    pub fn new(set: &TileSet, table: &Table, seed: u32) -> Result<Game, Vec<LoadError>> {
        let world = World::new(set, table, seed, settings())?;
        let player = (0, 0);
        Ok(Game {
            world,
            player,
            visited: BTreeSet::from([player]),
        })
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn player(&self) -> Pos {
        self.player
    }

    pub fn visited(&self) -> &BTreeSet<Pos> {
        &self.visited
    }

    /// Steps the player one cell in `d` if the two cells are joined
    /// (`World::joined`: both stored, neither facing side hedge; DR-0044
    /// point 1), then
    /// grows the world around them and marks the cell visited. Returns
    /// whether they moved; a blocked step changes nothing.
    pub fn step(&mut self, d: Direction) -> bool {
        if !self.world.joined(self.player, d) {
            return false;
        }
        self.player = world::step(self.player, d);
        self.world.grow_around(self.player);
        self.visited.insert(self.player);
        true
    }

    /// What the player sees: every stored cell within `VIEW_RADIUS`, in
    /// (y, x) order. An error only if a coordinate doesn't fit in i32,
    /// which no walk can reach (DR-0043 point 7).
    pub fn view(&self, moved: bool) -> Result<GameView, GameError> {
        let set = self.world.tile_set();
        let (px, py) = self.player;
        let r = VIEW_RADIUS;
        let mut cells = Vec::new();
        for y in py - r..=py + r {
            for x in px - r..=px + r {
                let Some(v) = self.world.get((x, y)) else {
                    continue;
                };
                let variant = &set.variants[v];
                let tile = set.tile_of(variant);
                cells.push(ViewCell {
                    x: coord(x)?,
                    y: coord(y)?,
                    tile: tile.id.clone(),
                    orientation: variant.orientation,
                    kind: tile.kind.into(),
                    visited: self.visited.contains(&(x, y)),
                });
            }
        }
        Ok(GameView {
            player: ipc::Pos {
                x: coord(px)?,
                y: coord(py)?,
            },
            moved,
            radius: VIEW_RADIUS as u32,
            cells,
        })
    }
}

/// A world coordinate for the frontend. i32 keeps it a plain number in
/// TypeScript (DR-0023); a player would need two billion steps to pass it.
fn coord(c: i64) -> Result<i32, GameError> {
    i32::try_from(c).map_err(|_| GameError {
        message: format!("World coordinate {c} is too far out to show."),
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::collections::BTreeMap;

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");

    pub(crate) fn core() -> (TileSet, Table) {
        let set = TileSet::from_toml(CORE).expect("core.toml loads");
        let table = Table::calculate(&set);
        (set, table)
    }

    fn game(seed: u32) -> Game {
        let (set, table) = core();
        Game::new(&set, &table, seed).expect("core passes the completeness check")
    }

    /// A seeded wander: each step takes an open way, preferring cells not
    /// yet visited, so it heads into new ground. Returns the directions
    /// taken; every one moved.
    fn wander(game: &mut Game, seed: u64, steps: usize) -> Vec<Direction> {
        let mut taken = Vec::new();
        for i in 0..steps {
            let open: Vec<Direction> = Direction::ALL
                .into_iter()
                .filter(|&d| game.world().joined(game.player(), d))
                .collect();
            assert!(
                !open.is_empty(),
                "the player is shut in at {:?}",
                game.player()
            );
            let fresh: Vec<Direction> = open
                .iter()
                .copied()
                .filter(|&d| !game.visited().contains(&world::step(game.player(), d)))
                .collect();
            let ways = if fresh.is_empty() { open } else { fresh };
            let d = ways[(world::hash(&[seed, i as u64]) % ways.len() as u64) as usize];
            assert!(game.step(d));
            taken.push(d);
        }
        taken
    }

    fn tiles(view: &GameView) -> BTreeMap<(i32, i32), (String, u32)> {
        view.cells
            .iter()
            .map(|c| ((c.x, c.y), (c.tile.clone(), c.orientation)))
            .collect()
    }

    #[test]
    fn the_player_starts_on_a_walkable_cell_with_a_way_on() {
        for seed in 0..20 {
            let game = game(seed);
            assert_eq!(game.player(), (0, 0));
            let v = game.world().get((0, 0)).expect("the origin is stored");
            let set = game.world().tile_set();
            assert_ne!(
                set.tile_of(&set.variants[v]).kind,
                maze_core::TileKind::Hedge
            );
            assert!(
                Direction::ALL
                    .into_iter()
                    .any(|d| game.world().joined((0, 0), d)),
                "seed {seed}: no way off the start cell"
            );
            let view = game.view(true).unwrap();
            let here = view.cells.iter().find(|c| (c.x, c.y) == (0, 0)).unwrap();
            assert!(here.visited);
        }
    }

    /// DR-0035: what was generated is kept. Walk out, then retrace every
    /// step: at each cell on the way back the view shows the same tiles it
    /// showed there on the way out (the world may have grown at the edges
    /// since, but nothing seen has changed).
    #[test]
    fn walking_back_finds_the_world_as_it_was() {
        for seed in [1, 7, 42] {
            let mut game = game(seed);
            let mut out = vec![game.view(true).unwrap()];
            let path = wander_recording(&mut game, seed.into(), 60, &mut out);
            for (i, &d) in path.iter().enumerate().rev() {
                assert!(game.step(d.opposite()), "seed {seed}: can't step back");
                let before = tiles(&out[i]);
                let now = tiles(&game.view(true).unwrap());
                for (p, tile) in &before {
                    assert_eq!(now.get(p), Some(tile), "seed {seed}: {p:?} changed");
                }
            }
            assert_eq!(game.player(), (0, 0));
        }
    }

    fn wander_recording(
        game: &mut Game,
        seed: u64,
        steps: usize,
        views: &mut Vec<GameView>,
    ) -> Vec<Direction> {
        let mut path = Vec::new();
        for i in 0..steps {
            let d = wander(game, world::hash(&[seed, i as u64]), 1)[0];
            path.push(d);
            views.push(game.view(true).unwrap());
        }
        path
    }

    #[test]
    fn a_blocked_step_changes_nothing() {
        let mut game = game(3);
        let mut checked = 0;
        for i in 0..200 {
            for d in Direction::ALL {
                if game.world().joined(game.player(), d) {
                    continue;
                }
                let before = game.view(true).unwrap();
                let stored = game.world().len();
                let visited = game.visited().len();
                assert!(!game.step(d));
                assert_eq!(game.view(true).unwrap(), before);
                assert_eq!(game.world().len(), stored);
                assert_eq!(game.visited().len(), visited);
                checked += 1;
            }
            wander(&mut game, i, 1);
        }
        assert!(checked > 0, "never met a blocked side");
    }

    #[test]
    fn the_view_radius_is_below_the_growth_radius() {
        // DR-0036 rule 2, DR-0043 point 7. Also checked at compile time.
        assert!(VIEW_RADIUS < settings().radius);
    }

    #[test]
    fn a_coordinate_past_i32_is_an_error() {
        assert_eq!(coord(-5).unwrap(), -5);
        assert!(coord(i64::from(i32::MAX) + 1).is_err());
    }

    #[test]
    fn a_seed_gives_the_same_game() {
        let mut a = game(11);
        let mut b = game(11);
        assert_eq!(a.view(true).unwrap(), b.view(true).unwrap());
        let path = wander(&mut a, 99, 50);
        for d in path {
            assert!(b.step(d));
        }
        assert_eq!(a.view(true).unwrap(), b.view(true).unwrap());
        assert_eq!(a.world().len(), b.world().len());
        assert_ne!(game(12).view(true).unwrap(), game(11).view(true).unwrap());
    }

    #[test]
    fn the_world_grows_as_the_player_walks() {
        let mut game = game(8);
        let at_start = game.world().len();
        wander(&mut game, 8, 80);
        assert!(
            game.world().len() > at_start,
            "{} cells after walking, {at_start} at the start",
            game.world().len()
        );
        // Every visited cell is stored.
        assert!(
            game.visited()
                .iter()
                .all(|&p| game.world().get(p).is_some())
        );
    }

    #[test]
    fn the_view_lists_stored_cells_in_range_in_row_major_order() {
        let mut game = game(4);
        wander(&mut game, 4, 30);
        let view = game.view(false).unwrap();
        assert!(!view.moved);
        assert_eq!(view.radius, 6);
        let (px, py) = (view.player.x, view.player.y);
        let mut expected = Vec::new();
        for y in py - 6..=py + 6 {
            for x in px - 6..=px + 6 {
                if game.world().get((x.into(), y.into())).is_some() {
                    expected.push((x, y));
                }
            }
        }
        let listed: Vec<(i32, i32)> = view.cells.iter().map(|c| (c.x, c.y)).collect();
        assert_eq!(listed, expected);
        for c in &view.cells {
            assert_eq!(
                c.visited,
                game.visited().contains(&(c.x.into(), c.y.into()))
            );
            assert!(c.orientation < 4);
        }
    }
}
