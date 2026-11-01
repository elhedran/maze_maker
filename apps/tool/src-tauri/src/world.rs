//! The World tab's state and commands (DR-0041, the view in DR-0039).
//!
//! `WorldState` is plain Rust, tested directly like `ToolState`. It holds
//! one world at a time, grown with a copy of the tile set that was loaded
//! when it started, so a Reload never touches it or its replay. `lib.rs`
//! wraps it in its own `Mutex`; no command holds that lock and
//! `ToolState`'s together.
//!
//! Growth is driven one patch at a time, so each patch's delta, discarded
//! cells included, is captured as it is made (DR-0041 rule 5).

use crate::ipc::{
    Endpoint, PatchDelta, ReplayCheck, ReplayStepRequest, ReplayUpdate, StoredCell, ToolError,
    ToolErrorKind, Walk, WalkRequest, WorldPos, WorldRect, WorldRef, WorldStartRequest,
    WorldStarted, WorldUpdate,
};
use crate::state::ToolState;
use crate::view::{self, Loaded};
use maze_core::walker::Walker;
use maze_core::world::{Anchor, Pos, Settings, World, step};
use maze_core::{Table, TileSet};
use std::collections::BTreeSet;
use std::sync::{Mutex, MutexGuard};

/// Steps a seeded walk may take in one request (DR-0041 rule 9).
pub const MAX_WALK_STEPS: u32 = 1_000;
/// Steps a world may take in all (DR-0041 rule 9). With it, no stored
/// cell lies more than about a million cells from the origin, so every
/// position fits `i32` (rule 10).
pub const MAX_WORLD_STEPS: u32 = 1_000_000;

const REPLACED: &str = "That world has been replaced. Press New world.";

fn error(kind: ToolErrorKind, message: impl Into<String>) -> ToolError {
    ToolError {
        kind,
        messages: vec![message.into()],
    }
}

fn bad_request(message: impl Into<String>) -> ToolError {
    error(ToolErrorKind::BadRequest, message)
}

/// A core position over IPC. Never fails in practice (rule 10), but a
/// failure is `bad_request`, not a panic.
fn pos(p: Pos) -> Result<WorldPos, ToolError> {
    match (i32::try_from(p.0), i32::try_from(p.1)) {
        (Ok(x), Ok(y)) => Ok(WorldPos { x, y }),
        _ => Err(bad_request(format!(
            "The cell at {}, {} is too far from the origin to send.",
            p.0, p.1
        ))),
    }
}

fn rect(lo: Pos, hi: Pos) -> Result<WorldRect, ToolError> {
    let at = pos(lo)?;
    let size = |a: i64, b: i64| {
        u32::try_from(b - a + 1).map_err(|_| bad_request("The world is too wide to send."))
    };
    Ok(WorldRect {
        x: at.x,
        y: at.y,
        width: size(lo.0, hi.0)?,
        height: size(lo.1, hi.1)?,
    })
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// What a world is grown with: a copy of the load it started from.
#[derive(Clone, Debug)]
pub struct Source {
    pub tile_set: TileSet,
    pub table: Table,
    pub revision: u32,
    pub loaded_at: String,
}

impl Source {
    /// Copies what a world needs out of the loaded tile set, so the caller
    /// can release `ToolState`'s lock before locking `WorldState`.
    pub fn of(loaded: &Loaded) -> Source {
        Source {
            tile_set: loaded.tile_set.clone(),
            table: loaded.table.clone(),
            revision: loaded.revision,
            loaded_at: loaded.loaded_at.clone(),
        }
    }
}

/// What the growth log doesn't hold about a patch: the walk step after
/// which it grew, and its discarded cells.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PatchNotes {
    step: u32,
    discarded: Vec<Pos>,
}

/// A world's replay from its growth log (rule 11).
struct Replay {
    world: World,
    notes: Vec<PatchNotes>,
    /// Set when replay ends: the last patch replayed, or a divergence.
    check: Option<ReplayCheck>,
}

struct Session {
    id: u32,
    source: Source,
    world: World,
    player: Pos,
    steps: u32,
    /// The seeded walk's seed and walker, carried on while the seed stays
    /// and the player hasn't moved by keys since.
    walk: Option<(u32, Walker)>,
    /// One per growth log entry.
    notes: Vec<PatchNotes>,
    replay: Option<Replay>,
}

/// The World tab's state: at most one world, named by its id.
#[derive(Default)]
pub struct WorldState {
    last_id: u32,
    session: Option<Session>,
}

/// An empty world from `source`. A set `World` refuses comes back as
/// `bad_request` with its messages.
fn empty_world(source: &Source, seed: u32) -> Result<World, ToolError> {
    World::replay(
        &source.tile_set,
        &source.table,
        seed,
        Settings::default(),
        &[],
    )
    .map_err(|errors| ToolError {
        kind: ToolErrorKind::BadRequest,
        messages: errors.iter().map(|e| e.to_string()).collect(),
    })
}

/// Grows one patch at `anchor` and returns its notes.
fn grow(world: &mut World, anchor: Anchor, step: u32) -> PatchNotes {
    PatchNotes {
        step,
        discarded: world.grow_with_discards(anchor),
    }
}

/// The delta for growth log entry `index` of `world`.
fn delta(world: &World, index: usize, notes: &PatchNotes) -> Result<PatchDelta, ToolError> {
    let patch = &world.log()[index];
    let size = i64::try_from(world.settings().patch_size).expect("patch size is at most 64");
    let (wx, wy) = patch.window;
    let cells = patch
        .stored
        .iter()
        .map(|&p| {
            let at = pos(p)?;
            let tile = world.get(p).expect("a logged cell is stored");
            Ok(StoredCell {
                x: at.x,
                y: at.y,
                tile: count(tile),
            })
        })
        .collect::<Result<_, ToolError>>()?;
    let positions = |ps: &[Pos]| ps.iter().map(|&p| pos(p)).collect::<Result<Vec<_>, _>>();
    Ok(PatchDelta {
        index: count(index),
        step: notes.step,
        anchor: pos(patch.anchor.cell)?,
        from: patch.anchor.from.map(view::direction),
        window: rect((wx, wy), (wx + size - 1, wy + size - 1))?,
        attempts: patch.attempts,
        spine: patch.spine,
        fallback: patch.fallback,
        cells,
        spine_cells: positions(&patch.spine_cells)?,
        discarded: positions(&notes.discarded)?,
    })
}

fn frontier(world: &World) -> Result<Vec<Endpoint>, ToolError> {
    world
        .frontier()
        .into_iter()
        .map(|e| {
            Ok(Endpoint {
                cell: pos(e.cell)?,
                facing: view::direction(e.facing),
                anchor: pos(step(e.cell, e.facing))?,
                outward: e.outward,
            })
        })
        .collect()
}

fn bounds(world: &World) -> Result<Option<WorldRect>, ToolError> {
    world.bounds().map(|(lo, hi)| rect(lo, hi)).transpose()
}

/// Rust's same-world check (rule 12): same cells with the same tiles, and
/// the same growth log with the same discards.
fn check(walked: &World, walked_notes: &[PatchNotes], replay: &Replay) -> ReplayCheck {
    let replayed = &replay.world;
    let mut cells = 0;
    let mut differ: BTreeSet<(i64, i64)> = BTreeSet::new();
    for (p, v) in walked.cells() {
        match replayed.get(p) {
            Some(u) if u == v => cells += 1,
            _ => {
                differ.insert((p.1, p.0));
            }
        }
    }
    for (p, _) in replayed.cells() {
        if walked.get(p).is_none() {
            differ.insert((p.1, p.0));
        }
    }
    let first_difference = differ.first().and_then(|&(y, x)| pos((x, y)).ok());
    let same = differ.is_empty()
        && replayed.log() == walked.log()
        && replay
            .notes
            .iter()
            .map(|n| &n.discarded)
            .eq(walked_notes.iter().map(|n| &n.discarded));
    ReplayCheck {
        same,
        cells: count(cells),
        first_difference,
    }
}

impl Session {
    /// Grows around the player until no anchor is within r, returning the
    /// patches' deltas.
    fn grow_around(&mut self) -> Result<Vec<PatchDelta>, ToolError> {
        let mut deltas = Vec::new();
        while let Some(anchor) = self.world.nearest_anchor(self.player) {
            let index = self.world.log().len();
            self.notes.push(grow(&mut self.world, anchor, self.steps));
            deltas.push(delta(&self.world, index, &self.notes[index])?);
        }
        Ok(deltas)
    }

    fn update(
        &self,
        path: Vec<WorldPos>,
        patches: Vec<PatchDelta>,
    ) -> Result<WorldUpdate, ToolError> {
        Ok(WorldUpdate {
            world_id: self.id,
            player: pos(self.player)?,
            path,
            patches,
            frontier: frontier(&self.world)?,
            bounds: bounds(&self.world)?.expect("a started world has its origin stored"),
            steps: self.steps,
            patch_count: count(self.world.log().len()),
            stored_count: count(self.world.len()),
        })
    }

    fn replay_update(&self, patches: Vec<PatchDelta>) -> Result<ReplayUpdate, ToolError> {
        let replay = self.replay.as_ref().expect("a replay is running");
        Ok(ReplayUpdate {
            world_id: self.id,
            patches,
            frontier: frontier(&replay.world)?,
            bounds: bounds(&replay.world)?,
            replayed: count(replay.world.log().len()),
            total: count(self.world.log().len()),
            check: replay.check,
        })
    }
}

impl WorldState {
    fn session(&mut self, world_id: u32) -> Result<&mut Session, ToolError> {
        self.session
            .as_mut()
            .filter(|s| s.id == world_id)
            .ok_or_else(|| error(ToolErrorKind::Stale, REPLACED))
    }

    /// Starts a new world from `source`, replacing any world: the origin
    /// patch, then growth around the player at the origin.
    pub fn start(
        &mut self,
        source: Source,
        request: &WorldStartRequest,
    ) -> Result<WorldStarted, ToolError> {
        let mut world = empty_world(&source, request.seed)?;
        let origin = Anchor {
            cell: (0, 0),
            from: None,
        };
        let tiles = view::tile_views(&source.tile_set);
        let notes = vec![grow(&mut world, origin, 0)];
        self.last_id = self.last_id.wrapping_add(1);
        let mut session = Session {
            id: self.last_id,
            source,
            world,
            player: (0, 0),
            steps: 0,
            walk: None,
            notes,
            replay: None,
        };
        // Drop the old world first, so a failure below leaves none, not a
        // stale one under a new id.
        self.session = None;
        let mut patches = vec![delta(&session.world, 0, &session.notes[0])?];
        patches.extend(session.grow_around()?);
        let update = session.update(Vec::new(), patches)?;
        let started = WorldStarted {
            world_id: session.id,
            seed: request.seed,
            revision: session.source.revision,
            loaded_at: session.source.loaded_at.clone(),
            tiles,
            update,
        };
        self.session = Some(session);
        Ok(started)
    }

    /// One key step, or a run of the seeded random walk.
    pub fn walk(&mut self, request: &WalkRequest) -> Result<WorldUpdate, ToolError> {
        let session = self.session(request.world_id)?;
        let wanted = match request.walk {
            Walk::Step { .. } => 1,
            Walk::Random { steps, .. } => {
                if !(1..=MAX_WALK_STEPS).contains(&steps) {
                    return Err(bad_request(format!(
                        "A walk takes 1 to {MAX_WALK_STEPS} steps at a time, not {steps}."
                    )));
                }
                steps
            }
        };
        if session.steps.saturating_add(wanted) > MAX_WORLD_STEPS {
            return Err(bad_request(format!(
                "A world takes at most {MAX_WORLD_STEPS} steps. Press New world."
            )));
        }
        // The view doesn't walk during replay; a walk ends one, so a replay
        // never runs against a log that grew under it.
        session.replay = None;
        let mut path = Vec::new();
        let mut patches = Vec::new();
        match request.walk {
            Walk::Step { direction } => {
                let d = core_direction(direction);
                // Blocked: nothing moves, nothing grows, and it isn't an
                // error (rule 6).
                if session.world.joined(session.player, d) {
                    session.player = step(session.player, d);
                    session.steps += 1;
                    path.push(pos(session.player)?);
                    patches = session.grow_around()?;
                }
            }
            Walk::Random { seed, steps } => {
                // A Walker at the default revisit weight, so the walk
                // matches core's `World::random_walk` and rarely retraces
                // itself (Ian's steering). A new seed, or a key step since,
                // starts a new walker at the player, with a fresh memory of
                // where it has walked.
                let mut walker = match session.walk.take() {
                    Some((s, w)) if s == seed && w.at() == session.player => w,
                    _ => Walker::new(session.player, u64::from(seed)),
                };
                for _ in 0..steps {
                    // `advance` doesn't grow; the session grows, so each
                    // patch gets its notes.
                    session.player = walker.advance(&session.world);
                    session.steps += 1;
                    path.push(pos(session.player)?);
                    patches.extend(session.grow_around()?);
                }
                session.walk = Some((seed, walker));
            }
        }
        session.update(path, patches)
    }

    /// Starts replaying the world's growth log into a fresh world, grown
    /// with the same copied tile set. Replaces any replay.
    pub fn replay_start(&mut self, request: &WorldRef) -> Result<ReplayUpdate, ToolError> {
        let session = self.session(request.world_id)?;
        let world = empty_world(&session.source, session.world.seed())?;
        session.replay = Some(Replay {
            world,
            notes: Vec::new(),
            check: None,
        });
        session.replay_update(Vec::new())
    }

    /// Replays the next `patches` patches, clamped to what's left. When
    /// the last is done, or the replayed world diverges, the response
    /// carries the check.
    pub fn replay_step(&mut self, request: &ReplayStepRequest) -> Result<ReplayUpdate, ToolError> {
        let session = self.session(request.world_id)?;
        if request.patches == 0 {
            return Err(bad_request("Replay at least one patch at a time."));
        }
        let Some(replay) = session.replay.as_mut() else {
            return Err(error(
                ToolErrorKind::Stale,
                "No replay is running for this world. Press Replay walk.",
            ));
        };
        let total = session.world.log().len();
        let mut deltas = Vec::new();
        let mut diverged = false;
        while replay.check.is_none() && deltas.len() < request.patches as usize {
            let index = replay.world.log().len();
            if index == total {
                break;
            }
            let anchor = session.world.log()[index].anchor;
            // A replay that grew differently may not have this anchor;
            // growing it would panic, so replay ends with "no" instead.
            if !replay.world.is_anchor(anchor) {
                diverged = true;
                break;
            }
            replay
                .notes
                .push(grow(&mut replay.world, anchor, session.notes[index].step));
            deltas.push(delta(&replay.world, index, &replay.notes[index])?);
        }
        if replay.check.is_none() && (diverged || replay.world.log().len() == total) {
            let mut result = check(&session.world, &session.notes, replay);
            result.same &= !diverged;
            replay.check = Some(result);
        }
        session.replay_update(deltas)
    }
}

fn core_direction(d: crate::ipc::Direction) -> maze_core::Direction {
    use crate::ipc::Direction as D;
    match d {
        D::North => maze_core::Direction::North,
        D::East => maze_core::Direction::East,
        D::South => maze_core::Direction::South,
        D::West => maze_core::Direction::West,
    }
}

/// As `crate::lock`: a poisoned lock means a panic mid-request. The world
/// may then be part-grown, but every command checks its inputs before
/// growing and none panics by design, so keep serving it.
fn lock_worlds(state: &Mutex<WorldState>) -> MutexGuard<'_, WorldState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

// The commands. `async` runs them off the main thread, so a long walk
// doesn't block the window; nothing is awaited while a lock is held.

/// Starts a world from the loaded tile set. `ToolState` is locked only to
/// copy the set out, and released before `WorldState` is locked.
#[tauri::command(async)]
pub fn world_start(
    tool: tauri::State<'_, Mutex<ToolState>>,
    worlds: tauri::State<'_, Mutex<WorldState>>,
    request: WorldStartRequest,
) -> Result<WorldStarted, ToolError> {
    let source = Source::of(crate::lock(&tool).loaded()?);
    lock_worlds(&worlds).start(source, &request)
}

#[tauri::command(async)]
pub fn world_walk(
    worlds: tauri::State<'_, Mutex<WorldState>>,
    request: WalkRequest,
) -> Result<WorldUpdate, ToolError> {
    lock_worlds(&worlds).walk(&request)
}

#[tauri::command(async)]
pub fn world_replay_start(
    worlds: tauri::State<'_, Mutex<WorldState>>,
    request: WorldRef,
) -> Result<ReplayUpdate, ToolError> {
    lock_worlds(&worlds).replay_start(&request)
}

#[tauri::command(async)]
pub fn world_replay_step(
    worlds: tauri::State<'_, Mutex<WorldState>>,
    request: ReplayStepRequest,
) -> Result<ReplayUpdate, ToolError> {
    lock_worlds(&worlds).replay_step(&request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{Direction, TableViewRequest};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");
    const DIRECTIONS: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    fn source_of(text: &str, revision: u32) -> Source {
        let tile_set = TileSet::from_toml(text).expect("loads");
        let table = Table::calculate(&tile_set);
        Source {
            tile_set,
            table,
            revision,
            loaded_at: "2026-11-01T20:14:00Z".into(),
        }
    }

    fn started(seed: u32) -> (WorldState, WorldStarted) {
        let mut state = WorldState::default();
        let out = state
            .start(source_of(CORE, 1), &WorldStartRequest { seed })
            .unwrap();
        (state, out)
    }

    fn random(world_id: u32, seed: u32, steps: u32) -> WalkRequest {
        WalkRequest {
            world_id,
            walk: Walk::Random { seed, steps },
        }
    }

    fn key(world_id: u32, direction: Direction) -> WalkRequest {
        WalkRequest {
            world_id,
            walk: Walk::Step { direction },
        }
    }

    /// The cells the deltas stored, by position.
    fn rebuilt<'a>(deltas: impl IntoIterator<Item = &'a PatchDelta>) -> BTreeMap<(i32, i32), u32> {
        let mut out = BTreeMap::new();
        for d in deltas {
            for c in &d.cells {
                assert!(out.insert((c.x, c.y), c.tile).is_none(), "stored twice");
            }
        }
        out
    }

    fn cells_of(world: &World) -> BTreeMap<(i32, i32), u32> {
        world
            .cells()
            .map(|((x, y), v)| ((x as i32, y as i32), v as u32))
            .collect()
    }

    fn walked(state: &WorldState) -> &World {
        &state.session.as_ref().unwrap().world
    }

    #[test]
    fn start_grows_the_origin_and_growth_around_it() {
        let (state, out) = started(48213);
        let set = source_of(CORE, 1).tile_set;
        let table = Table::calculate(&set);
        let new = World::new(&set, &table, 48213, Settings::default()).unwrap();
        let u = &out.update;
        assert_eq!(rebuilt(&u.patches), cells_of(&new));
        assert_eq!(cells_of(walked(&state)), cells_of(&new));
        let first = &u.patches[0];
        assert_eq!(first.anchor, WorldPos { x: 0, y: 0 });
        assert_eq!((first.from, first.step, first.index), (None, 0, 0));
        assert!(
            u.patches
                .iter()
                .skip(1)
                .all(|p| p.from.is_some() && p.step == 0)
        );
        let indices: Vec<u32> = u.patches.iter().map(|p| p.index).collect();
        assert_eq!(indices, (0..u.patches.len() as u32).collect::<Vec<_>>());
        assert_eq!(u.player, WorldPos { x: 0, y: 0 });
        assert!(u.path.is_empty());
        assert_eq!(u.steps, 0);
        assert_eq!(u.patch_count as usize, new.log().len());
        assert_eq!(u.stored_count as usize, new.len());
        assert!(u.frontier.iter().any(|e| e.outward));
        assert_eq!((out.seed, out.revision), (48213, 1));
        for e in &u.frontier {
            let (dx, dy) = match e.facing {
                Direction::North => (0, -1),
                Direction::East => (1, 0),
                Direction::South => (0, 1),
                Direction::West => (-1, 0),
            };
            assert_eq!(
                e.anchor,
                WorldPos {
                    x: e.cell.x + dx,
                    y: e.cell.y + dy
                }
            );
        }
        let (lo, hi) = new.bounds().unwrap();
        assert_eq!(
            u.bounds,
            WorldRect {
                x: lo.0 as i32,
                y: lo.1 as i32,
                width: (hi.0 - lo.0 + 1) as u32,
                height: (hi.1 - lo.1 + 1) as u32,
            }
        );
    }

    #[test]
    fn tiles_are_the_world_sets_in_table_order() {
        let (_, out) = started(1);
        let loaded = Loaded {
            tile_set: source_of(CORE, 1).tile_set,
            table: source_of(CORE, 1).table,
            revision: 1,
            tile_set_path: "core.toml".into(),
            loaded_at: String::new(),
        };
        let table = view::table_view(&loaded, &TableViewRequest::everything()).unwrap();
        assert_eq!(out.tiles, table.tiles);
        assert_eq!(view::tile_views(&loaded.tile_set), table.tiles);
    }

    #[test]
    fn a_blocked_key_step_moves_and_grows_nothing() {
        let (mut state, out) = started(7);
        let id = out.world_id;
        let world = walked(&state);
        let blocked = DIRECTIONS
            .into_iter()
            .find(|&d| !world.joined((0, 0), core_direction(d)))
            .expect("some side of the origin is blocked");
        let u = state.walk(&key(id, blocked)).unwrap();
        assert!(u.path.is_empty() && u.patches.is_empty());
        assert_eq!(u.player, WorldPos { x: 0, y: 0 });
        assert_eq!(u.steps, 0);
        assert_eq!(u.stored_count, out.update.stored_count);
        assert_eq!(u.frontier, out.update.frontier);
    }

    #[test]
    fn a_legal_key_step_moves_one_joined_cell() {
        let (mut state, out) = started(7);
        let id = out.world_id;
        let world = walked(&state);
        let open = DIRECTIONS
            .into_iter()
            .find(|&d| world.joined((0, 0), core_direction(d)))
            .expect("the origin is walkable");
        let u = state.walk(&key(id, open)).unwrap();
        let to = step((0, 0), core_direction(open));
        let to = WorldPos {
            x: to.0 as i32,
            y: to.1 as i32,
        };
        assert_eq!(u.path, vec![to]);
        assert_eq!((u.player, u.steps), (to, 1));
        assert!(u.patches.iter().all(|p| p.step == 1));
    }

    #[test]
    fn seeded_walk_one_step_at_a_time_equals_one_call() {
        let (mut by_one, out) = started(48213);
        let id = out.world_id;
        let ones: Vec<WorldUpdate> = (0..200)
            .map(|_| by_one.walk(&random(id, 907, 1)).unwrap())
            .collect();
        let (mut at_once, _) = started(48213);
        let all = at_once.walk(&random(id, 907, 200)).unwrap();
        let path: Vec<WorldPos> = ones.iter().flat_map(|u| u.path.clone()).collect();
        let patches: Vec<PatchDelta> = ones.iter().flat_map(|u| u.patches.clone()).collect();
        assert_eq!(path, all.path);
        assert_eq!(patches, all.patches);
        let last = ones.last().unwrap();
        assert_eq!(
            (
                &last.frontier,
                last.bounds,
                last.player,
                last.steps,
                last.patch_count,
                last.stored_count
            ),
            (
                &all.frontier,
                all.bounds,
                all.player,
                all.steps,
                all.patch_count,
                all.stored_count
            )
        );
        // And core's own random walk from the same cell and seed.
        let set = source_of(CORE, 1).tile_set;
        let table = Table::calculate(&set);
        let mut core = World::new(&set, &table, 48213, Settings::default()).unwrap();
        let core_path: Vec<WorldPos> = core
            .random_walk((0, 0), 907, 200)
            .into_iter()
            .map(|(x, y)| WorldPos {
                x: x as i32,
                y: y as i32,
            })
            .collect();
        assert_eq!(all.path, core_path);
        assert_eq!(cells_of(walked(&at_once)), cells_of(&core));
        assert_eq!(walked(&at_once).log(), core.log());
        let mut every = out.update.patches.clone();
        every.extend(all.patches.clone());
        assert_eq!(rebuilt(&every), cells_of(&core));
        // The same two seeds in a fresh world give identical updates.
        let (mut again, out2) = started(48213);
        assert_eq!(out2, out);
        assert_eq!(again.walk(&random(out2.world_id, 907, 200)).unwrap(), all);
    }

    #[test]
    fn a_new_walk_seed_starts_a_new_stream_from_the_player() {
        let (mut state, out) = started(5);
        let id = out.world_id;
        let first = state.walk(&random(id, 1, 50)).unwrap();
        let second = state.walk(&random(id, 2, 50)).unwrap();
        let set = source_of(CORE, 1).tile_set;
        let table = Table::calculate(&set);
        let mut core = World::new(&set, &table, 5, Settings::default()).unwrap();
        core.random_walk((0, 0), 1, 50);
        let from = (first.player.x as i64, first.player.y as i64);
        let path: Vec<WorldPos> = core
            .random_walk(from, 2, 50)
            .into_iter()
            .map(|(x, y)| WorldPos {
                x: x as i32,
                y: y as i32,
            })
            .collect();
        assert_eq!(second.path, path);
    }

    /// A world walked with keys and seeds, and every delta it sent.
    fn walked_world() -> (WorldState, u32, Vec<PatchDelta>, u32) {
        let (mut state, out) = started(2026);
        let id = out.world_id;
        let mut deltas = out.update.patches.clone();
        for seed in [11, 12, 13] {
            deltas.extend(state.walk(&random(id, seed, 150)).unwrap().patches);
            for d in DIRECTIONS {
                deltas.extend(state.walk(&key(id, d)).unwrap().patches);
            }
        }
        let stored = walked(&state).len() as u32;
        assert!(deltas.len() > out.update.patches.len(), "the walk grew");
        (state, id, deltas, stored)
    }

    #[test]
    fn replay_one_patch_at_a_time_is_the_same_world() {
        let (mut state, id, walked_deltas, stored) = walked_world();
        let begun = state.replay_start(&WorldRef { world_id: id }).unwrap();
        assert_eq!(
            (begun.replayed, begun.total),
            (0, walked_deltas.len() as u32)
        );
        assert!(begun.bounds.is_none() && begun.check.is_none() && begun.patches.is_empty());
        let mut replayed = Vec::new();
        loop {
            let u = state
                .replay_step(&ReplayStepRequest {
                    world_id: id,
                    patches: 1,
                })
                .unwrap();
            assert_eq!(u.patches.len(), 1);
            assert!(u.bounds.is_some());
            replayed.extend(u.patches);
            if let Some(check) = u.check {
                assert_eq!(u.replayed, u.total);
                assert_eq!(
                    check,
                    ReplayCheck {
                        same: true,
                        cells: stored,
                        first_difference: None
                    }
                );
                break;
            }
            assert!(u.replayed < u.total);
        }
        assert_eq!(replayed, walked_deltas);
    }

    #[test]
    fn replay_in_one_jump_is_the_same_world() {
        let (mut state, id, walked_deltas, stored) = walked_world();
        state.replay_start(&WorldRef { world_id: id }).unwrap();
        let u = state
            .replay_step(&ReplayStepRequest {
                world_id: id,
                patches: u32::MAX,
            })
            .unwrap();
        assert_eq!(u.patches, walked_deltas);
        assert_eq!(u.check.map(|c| (c.same, c.cells)), Some((true, stored)));
        // The same frontier as the walked world's.
        let walked_update = state
            .session
            .as_ref()
            .unwrap()
            .update(vec![], vec![])
            .unwrap();
        assert_eq!(u.frontier, walked_update.frontier);
        assert_eq!(u.bounds, Some(walked_update.bounds));
        // Past the end: nothing more, the same check.
        let after = state
            .replay_step(&ReplayStepRequest {
                world_id: id,
                patches: 1,
            })
            .unwrap();
        assert!(after.patches.is_empty());
        assert_eq!(after.check, u.check);
    }

    #[test]
    fn a_tampered_world_fails_the_check_at_the_first_difference() {
        let (state, _, _, _) = walked_world();
        let session = state.session.as_ref().unwrap();
        // A replay of only some of the log: the rest of the cells differ.
        let anchors: Vec<Anchor> = session.world.log().iter().map(|p| p.anchor).collect();
        let keep = anchors.len() / 2;
        let partial = World::replay(
            &session.source.tile_set,
            &session.source.table,
            session.world.seed(),
            Settings::default(),
            &anchors[..keep],
        )
        .unwrap();
        let mut missing: Vec<(i64, i64)> = session
            .world
            .cells()
            .map(|(p, _)| p)
            .filter(|&p| partial.get(p).is_none())
            .collect();
        missing.sort_by_key(|&(x, y)| (y, x));
        let replay = Replay {
            world: partial,
            notes: session.notes[..keep].to_vec(),
            check: None,
        };
        let result = check(&session.world, &session.notes, &replay);
        assert!(!result.same);
        assert_eq!(result.cells as usize, replay.world.len());
        let (x, y) = missing[0];
        assert_eq!(
            result.first_difference,
            Some(WorldPos {
                x: x as i32,
                y: y as i32
            })
        );
    }

    #[test]
    fn a_replay_that_diverges_says_no_instead_of_panicking() {
        let (mut state, id, _, _) = walked_world();
        state.replay_start(&WorldRef { world_id: id }).unwrap();
        // Swap in a replay world grown from another seed: its log can't
        // match, and its anchors soon stop being the walked log's.
        let session = state.session.as_mut().unwrap();
        let other = empty_world(&session.source, session.world.seed() + 1).unwrap();
        session.replay.as_mut().unwrap().world = other;
        let u = state
            .replay_step(&ReplayStepRequest {
                world_id: id,
                patches: u32::MAX,
            })
            .unwrap();
        let check = u.check.expect("replay ended");
        assert!(!check.same);
        assert!(check.first_difference.is_some());
    }

    /// A file in a fresh temp folder, removed on drop.
    struct TempFile(PathBuf);
    impl TempFile {
        fn new(name: &str, text: &str) -> TempFile {
            let dir = std::env::temp_dir().join(format!(
                "maze-tool-world-test-{}-{name}",
                std::process::id()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            let path = dir.join("tiles.toml");
            std::fs::write(&path, text).unwrap();
            TempFile(path)
        }
    }
    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(self.0.parent().unwrap());
        }
    }

    #[test]
    fn a_reload_leaves_the_world_on_its_own_tile_set() {
        let changed = CORE.replacen("weight = 1.2", "weight = 4.0", 1);
        assert_ne!(changed, CORE);
        let file = TempFile::new("reload", CORE);
        let mut tool = ToolState::default();
        tool.reload(&file.0).unwrap();
        let mut state = WorldState::default();
        let request = WorldStartRequest { seed: 31 };
        let out = state
            .start(Source::of(tool.loaded().unwrap()), &request)
            .unwrap();
        std::fs::write(&file.0, &changed).unwrap();
        assert_eq!(tool.reload(&file.0).unwrap().revision, 2);
        // The world walks and replays as one never reloaded.
        let (mut control, control_out) = started(31);
        assert_eq!(out.tiles, control_out.tiles);
        assert_eq!(out.revision, 1);
        let walk = state.walk(&random(out.world_id, 3, 300)).unwrap();
        assert_eq!(walk, control.walk(&random(out.world_id, 3, 300)).unwrap());
        state
            .replay_start(&WorldRef {
                world_id: out.world_id,
            })
            .unwrap();
        let replay = state
            .replay_step(&ReplayStepRequest {
                world_id: out.world_id,
                patches: u32::MAX,
            })
            .unwrap();
        assert!(replay.check.unwrap().same);
        // While a new world uses the reloaded set, which grows differently.
        let fresh = state
            .start(Source::of(tool.loaded().unwrap()), &request)
            .unwrap();
        assert_eq!(fresh.revision, 2);
        assert_ne!(rebuilt(&fresh.update.patches), rebuilt(&out.update.patches));
    }

    fn kind(result: Result<impl std::fmt::Debug, ToolError>) -> ToolErrorKind {
        let e = result.unwrap_err();
        assert!(!e.messages.is_empty() && !e.messages[0].is_empty());
        e.kind
    }

    #[test]
    fn errors_are_reported_not_panicked() {
        use ToolErrorKind::{BadRequest, NotLoaded, Stale};
        // No tile set loaded.
        assert_eq!(kind(ToolState::default().loaded()), NotLoaded);
        // No world yet.
        let mut empty = WorldState::default();
        assert_eq!(kind(empty.walk(&random(1, 1, 1))), Stale);
        assert_eq!(kind(empty.replay_start(&WorldRef { world_id: 1 })), Stale);
        let step = |id, patches| ReplayStepRequest {
            world_id: id,
            patches,
        };
        assert_eq!(kind(empty.replay_step(&step(1, 1))), Stale);

        let (mut state, out) = started(1);
        let id = out.world_id;
        // An unknown id, and a world replaced since.
        assert_eq!(kind(state.walk(&random(id + 1, 1, 1))), Stale);
        let newer = state
            .start(source_of(CORE, 1), &WorldStartRequest { seed: 2 })
            .unwrap();
        assert_ne!(newer.world_id, id);
        let replaced = state.walk(&random(id, 1, 1)).unwrap_err();
        assert_eq!(replaced.kind, Stale);
        assert_eq!(replaced.messages, vec![REPLACED.to_string()]);
        let id = newer.world_id;
        // Replay step with no replay, and after a walk ended it.
        assert_eq!(kind(state.replay_step(&step(id, 1))), Stale);
        state.replay_start(&WorldRef { world_id: id }).unwrap();
        state.walk(&random(id, 1, 1)).unwrap();
        assert_eq!(kind(state.replay_step(&step(id, 1))), Stale);
        // Out of range.
        assert_eq!(kind(state.walk(&random(id, 1, 0))), BadRequest);
        assert_eq!(kind(state.walk(&random(id, 1, 1001))), BadRequest);
        state.walk(&random(id, 1, 1000)).unwrap();
        state.replay_start(&WorldRef { world_id: id }).unwrap();
        assert_eq!(kind(state.replay_step(&step(id, 0))), BadRequest);
        // The world's step cap, checked before anything moves.
        let session = state.session.as_mut().unwrap();
        session.steps = MAX_WORLD_STEPS - 5;
        let before = session.player;
        assert_eq!(kind(state.walk(&random(id, 1, 6))), BadRequest);
        assert_eq!(state.session.as_ref().unwrap().player, before);
        state.walk(&random(id, 1, 5)).unwrap();
        assert_eq!(kind(state.walk(&key(id, Direction::North))), BadRequest);
        // A position out of i32.
        assert_eq!(kind(pos((i64::from(i32::MAX) + 1, 0))), BadRequest);
        assert_eq!(kind(pos((0, i64::from(i32::MIN) - 1))), BadRequest);
        assert_eq!(pos((-3, 4)).unwrap(), WorldPos { x: -3, y: 4 });
    }

    /// Every path in `v` that holds null.
    fn nulls(v: &serde_json::Value, at: &str, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Null => out.push(at.to_string()),
            serde_json::Value::Array(items) => {
                for item in items {
                    nulls(item, &format!("{at}[]"), out);
                }
            }
            serde_json::Value::Object(fields) => {
                for (k, item) in fields {
                    nulls(item, &format!("{at}.{k}"), out);
                }
            }
            _ => {}
        }
    }

    fn null_fields(v: &impl serde::Serialize) -> BTreeSet<String> {
        let mut out = Vec::new();
        nulls(&serde_json::to_value(v).unwrap(), "", &mut out);
        out.into_iter()
            .map(|p| p.rsplit('.').next().unwrap().to_string())
            .collect()
    }

    #[test]
    fn the_only_nulls_are_the_documented_ones() {
        let (mut state, out) = started(9);
        let id = out.world_id;
        let allowed: BTreeSet<String> = ["from", "bounds", "check", "firstDifference"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut seen = BTreeSet::new();
        seen.extend(null_fields(&out));
        seen.extend(null_fields(&state.walk(&random(id, 4, 100)).unwrap()));
        seen.extend(null_fields(
            &state.replay_start(&WorldRef { world_id: id }).unwrap(),
        ));
        let step = ReplayStepRequest {
            world_id: id,
            patches: u32::MAX,
        };
        seen.extend(null_fields(&state.replay_step(&step).unwrap()));
        assert!(seen.is_subset(&allowed), "{seen:?}");
        // `from` is null only for the origin; `bounds` only in a replay.
        let json = serde_json::to_value(&out).unwrap();
        let froms: Vec<bool> = json["update"]["patches"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["from"].is_null())
            .collect();
        assert!(froms[0] && froms[1..].iter().all(|n| !n));
        assert!(!json["update"]["bounds"].is_null());
    }

    #[test]
    fn requests_parse_from_the_bindings_json() {
        let walk: WalkRequest =
            serde_json::from_str(r#"{"worldId":3,"walk":{"kind":"step","direction":"north"}}"#)
                .unwrap();
        assert_eq!(walk, key(3, Direction::North));
        let walk: WalkRequest = serde_json::from_str(
            r#"{"worldId":3,"walk":{"kind":"random","seed":907,"steps":200}}"#,
        )
        .unwrap();
        assert_eq!(walk, random(3, 907, 200));
        let step: ReplayStepRequest = serde_json::from_str(r#"{"worldId":3,"patches":1}"#).unwrap();
        assert_eq!(step.patches, 1);
    }
}
