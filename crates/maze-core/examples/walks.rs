//! G.1's scripted walk, ported into maze-core so Wren can tune
//! `core.toml` (room and junction weights, `opening_pressure`) against the
//! same walks the G.1 fun review judged, without the game.
//!
//! cargo run --release -p maze-core --example walks -- [--tile-set PATH] [--seeds 1,2,3] [--steps N] [--opening-pressure P]
//!
//! Defaults: the repo's data/tile-sets/core.toml, seeds 1 to 5, 600 steps
//! (the review's walks), the tile set's own `opening_pressure`. With
//! `--opening-pressure P` (0 < P <= 1, 1 is off) the loaded tile set's value
//! is replaced in memory; the file is never touched. Seeded: the same
//! arguments and file always print the same numbers.
//!
//! Ported from apps/game/src-tauri/examples/fun_walk.rs at 57344e6 (branch
//! claude/game-g1-walk): the `Policy` trait and the `Explorer` player, the
//! walker's seed, the 13 × 13 Chebyshev view, and the definitions of a
//! way, ways onward, junction, doorway, room, dead end and dead-end length
//! are copied, not reworked. Snapshots, timings and the appearing-cells
//! and changed-cells counts are left out; Wren didn't ask for them here.
//!
//! What it reports, per seed and over all seeds:
//! - room in view: share of steps with any room cell in the view
//!   (fun_walk's `room_in_view`).
//! - steps per room seen / entered: steps ÷ distinct rooms seen, and
//!   steps ÷ distinct rooms the player stood in. The review's "a room comes
//!   into view every 12–24 steps" is the seen rate (600 ÷ 25–50); R1's
//!   "one entered every 60–100 steps" is the entered rate.
//! - choice in view: share of steps with a junction, doorway or room cell
//!   in view (fun_walk's `choice_in_view`), and the share with 2+ ways
//!   onward in view (fun_walk's figure for checklist item 3).
//! - quiet run (NEW, not in G.1): along the walked path, maximal runs of
//!   consecutive steps where the cell the player steps onto has 2 or fewer
//!   joined exits, counted right after that step's growth. A run ends on a
//!   step onto a cell with 3 or 4; back-to-back choice cells make no run
//!   (only runs of 1+ count), and the run still open when the walk ends is
//!   dropped. Steps walking back count too: it's the path as walked. This
//!   is the figure for R2's "quiet stretch of 3–5 cells between choices".
//! - dead ends > 3: of the closed dead ends among the cells seen during the
//!   walk, how many are longer than 3 cells (fun_walk's item 4).
//!
//! Uses only maze-core's public API: move into a stored, joined neighbour,
//! then `grow_around`, as the game's `step` does (DR-0044, DR-0036 rule 2).
//! The walk stays in this example on purpose; maze-core's own `Walker`
//! is coming separately and can slot in behind `Policy`.

use maze_core::wfc::SplitMix64;
use maze_core::world::{Pos, Settings, World, step};
use maze_core::{Direction, SideKind, Table, TileKind, TileSet};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");
const CORE_PATH: &str = "data/tile-sets/core.toml";
/// DR-0044 rule 4: the player sees 6 cells each way (Chebyshev), 13 × 13.
const VIEW: i64 = 6;
/// Dead ends longer than this count against DR-0044 checklist item 4.
const LONG_DEAD_END: usize = 3;

// ---------------------------------------------------------------- arguments

struct Args {
    tile_set: Option<String>,
    seeds: Vec<u32>,
    steps: usize,
    pressure: Option<f64>,
}

fn args() -> Args {
    let mut out = Args {
        tile_set: None,
        seeds: vec![1, 2, 3, 4, 5],
        steps: 600,
        pressure: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        if flag == "--help" || flag == "-h" {
            usage("");
        }
        let value = it
            .next()
            .unwrap_or_else(|| usage(&format!("{flag} needs a value")));
        let number = |v: &str| -> u64 {
            v.trim()
                .parse()
                .unwrap_or_else(|_| usage(&format!("{flag} {value}: not a whole number")))
        };
        match flag.as_str() {
            "--tile-set" => out.tile_set = Some(value),
            "--seeds" => {
                out.seeds = value
                    .split(',')
                    .filter(|s| !s.trim().is_empty())
                    .map(|s| {
                        u32::try_from(number(s))
                            .unwrap_or_else(|_| usage(&format!("seed {s}: more than a u32")))
                    })
                    .collect();
                if out.seeds.is_empty() {
                    usage("--seeds needs at least one seed");
                }
            }
            "--steps" => match number(&value) {
                n @ 1..=100_000 => out.steps = n as usize,
                _ => usage("--steps is 1 to 100000"),
            },
            "--opening-pressure" => match value.trim().parse::<f64>() {
                Ok(p) if p > 0.0 && p <= 1.0 => out.pressure = Some(p),
                _ => usage("--opening-pressure is above 0 and at most 1 (1 turns it off)"),
            },
            _ => usage(&format!("unknown flag {flag}")),
        }
    }
    out
}

fn usage(why: &str) -> ! {
    eprintln!(
        "{why}\nusage: walks [--tile-set PATH (default {CORE_PATH})] [--seeds 1,2,3 (default 1,2,3,4,5)] \
         [--steps N (default 600)] [--opening-pressure P (default: the tile set's)]"
    );
    std::process::exit(2)
}

// --------------------------------------------------------------- the world

/// Read-only questions about a world. Copied from fun_walk.rs.
struct Look<'w> {
    world: &'w World,
}

impl Look<'_> {
    fn tile(&self, p: Pos) -> Option<(&str, TileKind)> {
        let set = self.world.tile_set();
        self.world.get(p).map(|v| {
            let tile = set.tile_of(&set.variants[v]);
            (tile.id.as_str(), tile.kind)
        })
    }

    fn kind(&self, p: Pos) -> Option<TileKind> {
        self.tile(p).map(|(_, k)| k)
    }

    fn is_room(&self, p: Pos) -> bool {
        self.kind(p) == Some(TileKind::Room)
    }

    fn side(&self, p: Pos, d: Direction) -> Option<SideKind> {
        let set = self.world.tile_set();
        self.world.get(p).map(|v| set.variants[v].side(d))
    }

    /// Directions the player can step from `p` (DR-0044 rule 2).
    fn ways(&self, p: Pos) -> Vec<Direction> {
        Direction::ALL
            .into_iter()
            .filter(|&d| self.world.joined(p, d))
            .collect()
    }

    /// A walkable side of `p` faces an unsettled cell: an open endpoint.
    fn is_open(&self, p: Pos) -> bool {
        Direction::ALL.into_iter().any(|d| {
            self.side(p, d).is_some_and(|s| s != SideKind::Hedge)
                && self.world.get(step(p, d)).is_none()
        })
    }

    /// A junction: a corridor cell with three or four ways.
    fn is_junction(&self, p: Pos) -> bool {
        self.kind(p) == Some(TileKind::Corridor) && self.ways(p).len() >= 3
    }

    fn is_doorway(&self, p: Pos) -> bool {
        self.tile(p).is_some_and(|(id, _)| id == "room_doorway")
    }

    /// A dead end: a stored corridor cell with exactly one walkable side,
    /// and that side joined (so not an open endpoint).
    fn is_dead_end(&self, p: Pos) -> bool {
        if self.kind(p) != Some(TileKind::Corridor) {
            return false;
        }
        let walkable = Direction::ALL
            .into_iter()
            .filter(|&d| self.side(p, d).is_some_and(|s| s != SideKind::Hedge))
            .count();
        walkable == 1 && self.ways(p).len() == 1
    }

    /// From a dead end, how many cells to the first junction, room or
    /// doorway. `None` if the corridor runs into the unknown first.
    fn dead_end_length(&self, from: Pos) -> Option<usize> {
        let mut prev = from;
        let mut at = step(from, self.ways(from)[0]);
        let mut length = 1;
        loop {
            if self.is_open(at) {
                return None;
            }
            let onward: Vec<Pos> = self
                .ways(at)
                .into_iter()
                .map(|d| step(at, d))
                .filter(|&q| q != prev)
                .collect();
            if onward.len() != 1 || self.kind(at) != Some(TileKind::Corridor) {
                return Some(length);
            }
            prev = at;
            at = onward[0];
            length += 1;
            if length > 10_000 {
                return None; // A loop of corridor; not a dead end at all.
            }
        }
    }

    /// The room cells joined to `p`, a room cell; its first cell names it.
    fn room_at(&self, p: Pos) -> Vec<Pos> {
        let mut seen = BTreeSet::from([p]);
        let mut stack = vec![p];
        while let Some(q) = stack.pop() {
            for d in Direction::ALL {
                let n = step(q, d);
                if self.world.joined(q, d) && self.is_room(n) && seen.insert(n) {
                    stack.push(n);
                }
            }
        }
        seen.into_iter().collect()
    }

    /// Of the player's ways, how many lead (within the view, without coming
    /// back through the player) to the view's edge or to the unknown.
    fn ways_onward(&self, at: Pos) -> usize {
        self.ways(at)
            .into_iter()
            .filter(|&d| {
                let start = step(at, d);
                let mut seen = BTreeSet::from([at, start]);
                let mut queue = VecDeque::from([start]);
                while let Some(q) = queue.pop_front() {
                    if chebyshev(q, at) == VIEW || self.is_open(q) {
                        return true;
                    }
                    for e in self.ways(q) {
                        let n = step(q, e);
                        if chebyshev(n, at) <= VIEW && seen.insert(n) {
                            queue.push_back(n);
                        }
                    }
                }
                false
            })
            .count()
    }
}

fn chebyshev(a: Pos, b: Pos) -> i64 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn in_view(at: Pos) -> impl Iterator<Item = Pos> {
    (-VIEW..=VIEW).flat_map(move |dy| (-VIEW..=VIEW).map(move |dx| (at.0 + dx, at.1 + dy)))
}

// --------------------------------------------------------------- the player

/// Who chooses the next step. Kept from fun_walk.rs so maze-core's coming
/// `Walker` can replace `Explorer` with a one-line change in `walk_one`.
trait Policy {
    /// The direction to step from `at`, or `None` if there's nowhere to go.
    fn choose(&mut self, world: &World, at: Pos, visited: &BTreeSet<Pos>) -> Option<Direction>;
}

/// fun_walk.rs's curious, thorough player: an unvisited way if there is one
/// (a seeded draw among them), else the shortest path back to the nearest
/// cell that has one or that faces the unknown.
struct Explorer {
    rng: SplitMix64,
}

impl Policy for Explorer {
    fn choose(&mut self, world: &World, at: Pos, visited: &BTreeSet<Pos>) -> Option<Direction> {
        let look = Look { world };
        let fresh: Vec<Direction> = look
            .ways(at)
            .into_iter()
            .filter(|&d| !visited.contains(&step(at, d)))
            .collect();
        if !fresh.is_empty() {
            return Some(fresh[self.rng.below(fresh.len())]);
        }
        let mut first: BTreeMap<Pos, Direction> = BTreeMap::new();
        let mut queue = VecDeque::new();
        for d in look.ways(at) {
            let n = step(at, d);
            first.insert(n, d);
            queue.push_back(n);
        }
        while let Some(q) = queue.pop_front() {
            let worth = look.is_open(q)
                || look
                    .ways(q)
                    .into_iter()
                    .any(|e| !visited.contains(&step(q, e)));
            if worth {
                return Some(first[&q]);
            }
            for e in look.ways(q) {
                let n = step(q, e);
                if n != at && !first.contains_key(&n) {
                    first.insert(n, first[&q]);
                    queue.push_back(n);
                }
            }
        }
        None
    }
}

// --------------------------------------------------------------- one walk

#[derive(Default)]
struct Walk {
    /// Steps the player moved; every share below is out of these.
    moved: usize,
    boxed_in: usize,
    room_in_view: usize,
    choice_in_view: usize,
    two_ways_onward: usize,
    rooms_seen: usize,
    rooms_entered: usize,
    /// Cells in each room seen.
    room_sizes: Vec<usize>,
    quiet_runs: Vec<usize>,
    dead_ends: Vec<usize>,
    /// Walks counted (1 per seed; the sum in the "all" row).
    walks: usize,
    /// Ways out of the start cell, and how many lead onward in view
    /// (fun_walk.rs's checklist item 1); summed in the "all" row.
    start_ways: usize,
    start_ways_onward: usize,
    /// Walks whose start has 2+ ways onward.
    starts_with_two: usize,
}

impl Walk {
    fn long_dead_ends(&self) -> usize {
        self.dead_ends
            .iter()
            .filter(|&&l| l > LONG_DEAD_END)
            .count()
    }

    fn add(&mut self, w: &Walk) {
        self.moved += w.moved;
        self.boxed_in += w.boxed_in;
        self.room_in_view += w.room_in_view;
        self.choice_in_view += w.choice_in_view;
        self.two_ways_onward += w.two_ways_onward;
        self.rooms_seen += w.rooms_seen;
        self.rooms_entered += w.rooms_entered;
        self.room_sizes.extend(&w.room_sizes);
        self.quiet_runs.extend(&w.quiet_runs);
        self.dead_ends.extend(&w.dead_ends);
        self.walks += w.walks;
        self.start_ways += w.start_ways;
        self.start_ways_onward += w.start_ways_onward;
        self.starts_with_two += w.starts_with_two;
    }
}

fn walk_one(set: &TileSet, table: &Table, seed: u32, steps: usize) -> Walk {
    let mut world = World::new(set, table, seed, Settings::default())
        .unwrap_or_else(|e| fail(&format!("the tile set can't grow a world: {e:?}")));
    // fun_walk.rs's walker seed, apart from the world's.
    let mut policy: Box<dyn Policy> = Box::new(Explorer {
        rng: SplitMix64::new(0x6675_6e5f_7761_6c6b ^ u64::from(seed)),
    });
    let mut w = Walk {
        walks: 1,
        ..Walk::default()
    };
    let mut at: Pos = (0, 0);
    let mut visited = BTreeSet::from([at]);
    let mut seen: BTreeSet<Pos> = BTreeSet::new();
    // Room first cell -> (cells, entered yet).
    let mut rooms: BTreeMap<Pos, (usize, bool)> = BTreeMap::new();
    let mut quiet = 0usize;
    {
        let look = Look { world: &world };
        w.start_ways = look.ways(at).len();
        w.start_ways_onward = look.ways_onward(at);
        w.starts_with_two = usize::from(w.start_ways_onward >= 2);
    }
    observe(
        &Look { world: &world },
        at,
        false,
        &mut seen,
        &mut rooms,
        &mut w,
    );

    for _ in 0..steps {
        let Some(d) = policy.choose(&world, at, &visited) else {
            w.boxed_in += 1;
            continue;
        };
        if !world.joined(at, d) {
            continue; // The policy only picks joined ways; kept for safety.
        }
        at = step(at, d);
        world.grow_around(at);
        w.moved += 1;
        visited.insert(at);

        let look = Look { world: &world };
        if look.ways_onward(at) >= 2 {
            w.two_ways_onward += 1;
        }
        observe(&look, at, true, &mut seen, &mut rooms, &mut w);
        if look.is_room(at) {
            // `observe` has just recorded the room the player stands in.
            if let Some(room) = rooms.get_mut(&look.room_at(at)[0]) {
                room.1 = true;
            }
        }
        if look.ways(at).len() <= 2 {
            quiet += 1;
        } else if quiet > 0 {
            w.quiet_runs.push(quiet);
            quiet = 0;
        }
    }

    w.rooms_seen = rooms.len();
    w.rooms_entered = rooms.values().filter(|r| r.1).count();
    w.room_sizes = rooms.values().map(|r| r.0).collect();
    let look = Look { world: &world };
    w.dead_ends = seen
        .iter()
        .filter(|&&p| look.is_dead_end(p))
        .filter_map(|&p| look.dead_end_length(p))
        .collect();
    w
}

/// What the player sees: rooms (recorded as seen) and whether a choice or
/// a room is in view. `count` is false for the start, as in fun_walk.rs,
/// whose shares are out of the steps moved.
fn observe(
    look: &Look,
    at: Pos,
    count: bool,
    seen: &mut BTreeSet<Pos>,
    rooms: &mut BTreeMap<Pos, (usize, bool)>,
    w: &mut Walk,
) {
    let (mut choice, mut room) = (false, false);
    for p in in_view(at) {
        if look.world.get(p).is_none() {
            continue;
        }
        seen.insert(p);
        choice |= look.is_junction(p) || look.is_doorway(p);
        if look.is_room(p) {
            room = true;
            let cells = look.room_at(p);
            rooms.entry(cells[0]).or_insert((cells.len(), false));
        }
    }
    if count {
        w.room_in_view += usize::from(room);
        w.choice_in_view += usize::from(choice || room);
    }
}

// ---------------------------------------------------------------- the report

fn pct(part: usize, whole: usize) -> String {
    if whole == 0 {
        "n/a".into()
    } else {
        format!("{:.0}%", 100.0 * part as f64 / whole as f64)
    }
}

/// Steps per event, or "never".
fn every(steps: usize, events: usize) -> String {
    if events == 0 {
        "never".into()
    } else {
        format!("{:.0}", steps as f64 / events as f64)
    }
}

fn quiet_text(runs: &[usize]) -> String {
    if runs.is_empty() {
        return "none".into();
    }
    let mut v = runs.to_vec();
    v.sort_unstable();
    let mean = v.iter().sum::<usize>() as f64 / v.len() as f64;
    // The lower median, as fun_walk.rs's quantile(…, 0.5) takes it.
    let median = v[(v.len() - 1) / 2];
    format!("{mean:.1} / {median}")
}

fn mean(values: &[usize]) -> String {
    if values.is_empty() {
        "n/a".into()
    } else {
        format!(
            "{:.1}",
            values.iter().sum::<usize>() as f64 / values.len() as f64
        )
    }
}

fn rooms_row(label: &str, w: &Walk) -> String {
    format!(
        "{:<5} {:>7}  {:>5} {:>7}  {:>18}  {:>9}",
        label,
        pct(w.room_in_view, w.moved),
        every(w.moved, w.rooms_seen),
        every(w.moved, w.rooms_entered),
        format!(
            "{}/{}/{}",
            w.rooms_seen,
            w.rooms_entered,
            w.rooms_seen - w.rooms_entered
        ),
        mean(&w.room_sizes),
    )
}

fn paths_row(label: &str, w: &Walk) -> String {
    format!(
        "{:<5} {:>9}  {:>7}  {:>15}  {:>14} {:>3} {:>6}  {:>10}  {:>11}  {:>5}",
        label,
        pct(w.choice_in_view, w.moved),
        pct(w.two_ways_onward, w.moved),
        quiet_text(&w.quiet_runs),
        w.dead_ends.len(),
        w.long_dead_ends(),
        pct(w.long_dead_ends(), w.dead_ends.len()),
        format!("{}/{}", w.start_ways_onward, w.start_ways),
        format!("{}/{}", w.starts_with_two, w.walks),
        w.boxed_in,
    )
}

fn fail(why: &str) -> ! {
    eprintln!("{why}");
    std::process::exit(1)
}

fn main() {
    let a = args();
    let (text, path) = match &a.tile_set {
        None => (CORE.to_string(), CORE_PATH.to_string()),
        Some(p) => (
            std::fs::read_to_string(p).unwrap_or_else(|e| fail(&format!("can't read {p}: {e}"))),
            p.clone(),
        ),
    };
    let mut set =
        TileSet::from_toml(&text).unwrap_or_else(|e| fail(&format!("{path} doesn't load: {e:?}")));
    // The pressure only scales candidates in the solver (DR-0036 rule 9);
    // the table doesn't depend on it, so replacing it here is enough.
    let pressure_note = match a.pressure {
        Some(p) => {
            set.opening_pressure = p;
            "from --opening-pressure"
        }
        None => "from the tile set",
    };
    let table = Table::calculate(&set);

    let walks: Vec<Walk> = a
        .seeds
        .iter()
        .map(|&s| walk_one(&set, &table, s, a.steps))
        .collect();
    let mut all = Walk::default();
    for w in &walks {
        all.add(w);
    }
    let labels: Vec<String> = a.seeds.iter().map(u32::to_string).collect();
    let table = |head: [&str; 2], row: fn(&str, &Walk) -> String| {
        println!("{}", head[0]);
        println!("{}", head[1]);
        for (label, w) in labels.iter().zip(&walks) {
            println!("{}", row(label, w));
        }
        if walks.len() > 1 {
            println!("{}", row("all", &all));
        }
        println!();
    };

    println!(
        "G.1 walks: {path}, opening_pressure {} ({pressure_note}); {} steps per seed",
        set.opening_pressure, a.steps
    );
    println!();
    table(
        [
            "Rooms room in  steps per room  rooms seen/entered  mean room",
            "seed     view   seen entered       /never entered  size",
        ],
        rooms_row,
    );
    table(
        [
            "Paths choice in  2+ ways  quiet run (NEW)  dead ends: all  >3  share  start ways  starts with  boxed",
            "seed       view   onward    mean / median                             onward/all    2+ onward     in",
        ],
        paths_row,
    );
    println!("Shares are of steps moved (after each move and its growth).");
    println!("Mean room size is in cells, over the rooms seen.");
    println!("Choice in view: a junction, doorway or room cell in the 13 x 13 view.");
    println!("2+ ways onward: 2+ of the player's ways reach the view's rim or the unknown.");
    println!("Quiet run (new, not from G.1): consecutive steps onto cells with 2 or fewer");
    println!("  joined exits, ended by a step onto a cell with 3 or 4.");
    println!(
        "Dead ends: closed ones seen on the walk; >3 is those longer than {LONG_DEAD_END} cells."
    );
    println!("Boxed in: steps where the player had nowhere to go (DR-0044 item 5 says 0).");
}
