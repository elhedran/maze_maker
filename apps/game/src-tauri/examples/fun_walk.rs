//! G.1's scripted fun walk: a player walks grown worlds under DR-0044's
//! rules, and this reports what Wren needs to judge DR-0044's demo
//! checklist without running the game.
//!
//! cargo run --release -p maze-game --example fun_walk -- [--seeds 1,2,3] [--steps N] [--every N] [--out PATH]
//!
//! The walk is seeded, so the same arguments give the same counts and
//! snapshots; only the times vary with the machine. Without `--out` the
//! report goes to stdout; with it, to a Markdown file with OKF frontmatter
//! (DR-0003), such as docs/plans/g1-fun-walks.md.
//!
//! It uses only maze-core's public API, the same calls the game's `step`
//! makes: move into a stored, joined neighbour, then `grow_around`
//! (DR-0044 rules 1 to 3, DR-0036 rule 2). Nothing here changes core.

use maze_core::wfc::SplitMix64;
use maze_core::world::{Pos, Settings, World, step};
use maze_core::{Direction, SideKind, Table, TileKind, TileSet};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Write as _;
use std::time::{Duration, Instant};

const CORE: &str = include_str!("../../../../data/tile-sets/core.toml");
/// DR-0044 rule 4: the player sees 6 cells each way (Chebyshev), 13 × 13.
const VIEW: i64 = 6;
/// Dead ends longer than this count against DR-0044 checklist item 4.
const LONG_DEAD_END: usize = 3;

// ---------------------------------------------------------------- arguments

struct Args {
    seeds: Vec<u32>,
    steps: usize,
    every: usize,
    out: Option<String>,
}

fn args() -> Args {
    let mut out = Args {
        seeds: vec![1, 2, 3, 4, 5],
        steps: 300,
        every: 50,
        out: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it
            .next()
            .unwrap_or_else(|| usage(&format!("{flag} needs a value")));
        let number = |v: &str| -> u64 {
            v.trim()
                .parse()
                .unwrap_or_else(|_| usage(&format!("{flag} {value}: not a whole number")))
        };
        match flag.as_str() {
            "--seeds" => {
                out.seeds = value
                    .split(',')
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
            "--every" => match number(&value) {
                n @ 1..=100_000 => out.every = n as usize,
                _ => usage("--every is 1 to 100000"),
            },
            "--out" => out.out = Some(value),
            _ => usage(&format!("unknown flag {flag}")),
        }
    }
    out
}

fn usage(why: &str) -> ! {
    eprintln!(
        "{why}\nusage: fun_walk [--seeds 1,2,3 (default 1,2,3,4,5)] [--steps N (default 300)] \
         [--every N (snapshot interval, default 50)] [--out PATH (default stdout)]"
    );
    std::process::exit(2)
}

// ----------------------------------------------------------------- drawing

/// DR-0017's glyph for (tile id, orientation). A Rust copy of
/// apps/tool/src/table/tile-glyphs.ts at cac4b35, for this report only;
/// the game draws with its TypeScript copy (DR-0043 point 5).
fn glyph(id: &str, orientation: u32) -> char {
    let o = orientation as usize;
    let pick = |g: &[char]| g.get(o).copied().unwrap_or('?');
    match id {
        "corridor_straight" => pick(&['│', '─']),
        "corridor_corner" => pick(&['└', '┌', '┐', '┘']),
        "corridor_t_intersection" => pick(&['┴', '├', '┬', '┤']),
        "corridor_crossroads" => '┼',
        "corridor_dead_end" => pick(&['╵', '╶', '╷', '╴']),
        "open_room" => '·',
        "room_wall" => pick(&['═', '║', '═', '║']),
        "room_corner" => pick(&['╗', '╝', '╚', '╔']),
        "room_doorway" => pick(&['╧', '╟', '╤', '╢']),
        "hedge" => '░',
        _ => '?',
    }
}

/// The east filler, copied from apps/tool/src/maze/maze-text.ts at cac4b35
/// (DR-0029 point 7): cells are two characters and lines join across them.
fn east_filler(g: char) -> char {
    if "─└┌├┴┬┼╶╟".contains(g) {
        '─'
    } else if "═╔╚╧╤".contains(g) {
        '═'
    } else if g == '░' {
        '░'
    } else {
        ' '
    }
}

// --------------------------------------------------------------- the world

/// Read-only questions about a world, named for the report.
struct Look<'w> {
    world: &'w World,
}

impl Look<'_> {
    fn tile(&self, p: Pos) -> Option<(&str, u32, TileKind)> {
        let set = self.world.tile_set();
        self.world.get(p).map(|v| {
            let variant = &set.variants[v];
            let tile = set.tile_of(variant);
            (tile.id.as_str(), variant.orientation, tile.kind)
        })
    }

    fn kind(&self, p: Pos) -> Option<TileKind> {
        self.tile(p).map(|(_, _, k)| k)
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
        self.tile(p).is_some_and(|(id, _, _)| id == "room_doorway")
    }

    /// A dead end: a stored, non-room cell with exactly one walkable side,
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
    /// doorway: the cost of walking in and back out. `None` if the corridor
    /// runs into the unknown first (it may still open up).
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

    /// The room cells joined to `p`, a room cell. Rooms never span patches
    /// (DR-0036 rule 1), so a room is complete once one cell is stored.
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

    /// Ways onward in view: of the player's ways, how many lead (within the
    /// view, without coming back through the player) to the view's edge or
    /// to the unknown. A way that only reaches dead ends inside the view
    /// doesn't count. DR-0044 checklist item 1.
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

/// Who chooses the next step. One implementation now; maze-core's coming
/// `Walker` (2.6 step 2, Ian's steering: visited cells weighted low but not
/// zero) slots in behind this trait, a one-line change in `walk_one`.
trait Policy {
    /// The direction to step from `at`, or `None` if there's nowhere to go.
    fn choose(&mut self, world: &World, at: Pos, visited: &BTreeSet<Pos>) -> Option<Direction>;
}

/// A curious player: takes an unvisited way if there is one (a seeded
/// draw among them), else heads back along the shortest path to the
/// nearest cell that has one or that faces the unknown. Thorough: it walks
/// into every dead end and every room cell, so it pays every dead end's
/// cost, as a careful player might.
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
        // Backtrack: breadth-first to the nearest cell worth reaching.
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
struct RoomSeen {
    cells: usize,
    seen_at: usize,
    entered_at: Option<usize>,
}

#[derive(Default)]
struct WalkReport {
    seed: u32,
    steps: usize,
    moved: usize,
    /// Steps where the policy found nowhere to go (DR-0044 item 5).
    boxed_in: usize,
    start_ways: usize,
    start_ways_onward: usize,
    /// Per step, ways onward in view after the step.
    ways_onward: Vec<usize>,
    /// Steps whose view shows a junction, a doorway or a room (item 3).
    choice_in_view: usize,
    junction_in_view: usize,
    doorway_in_view: usize,
    room_in_view: usize,
    /// Runs of steps onto already-visited cells: the walk back (item 4).
    backtracks: Vec<usize>,
    /// Lengths of every closed dead end seen, at the end of the walk.
    dead_ends: Vec<usize>,
    /// Keyed by the room's first cell.
    rooms: BTreeMap<Pos, RoomSeen>,
    /// Cells seen whose tile later differed (item 6): must stay 0.
    changed: usize,
    /// Cells newly stored inside the view after a step, by distance from
    /// the player (index 0..=VIEW), and how many were walkable.
    appeared_by_distance: [usize; VIEW as usize + 1],
    appeared_walkable: usize,
    steps_with_appearance: usize,
    /// Steps where a cell appeared within VIEW - 1, not just on the rim.
    steps_with_inner_appearance: usize,
    step_times: Vec<Duration>,
    patches_per_step: Vec<usize>,
    start_time: Duration,
    stored: usize,
    /// Stored cells of hedge kind. DR-0036 rule 4 keeps only joined cells,
    /// and hedge has no walkable side, so this is 0: hedge is never stored.
    hedge_stored: usize,
    /// Per step, cells in the 13 × 13 view that are not yet known.
    unknown_in_view: Vec<usize>,
    patches: usize,
    has_way_onward: bool,
    snapshots: Vec<String>,
}

fn walk_one(set: &TileSet, table: &Table, seed: u32, a: &Args) -> WalkReport {
    let t = Instant::now();
    let mut world =
        World::new(set, table, seed, Settings::default()).expect("core.toml is complete");
    let mut r = WalkReport {
        seed,
        steps: a.steps,
        start_time: t.elapsed(),
        ..WalkReport::default()
    };
    // The walker's own seed, apart from the world's, so changing the
    // policy never changes the world it walks.
    let mut policy: Box<dyn Policy> = Box::new(Explorer {
        rng: SplitMix64::new(0x6675_6e5f_7761_6c6b ^ u64::from(seed)),
    });

    let mut at: Pos = (0, 0);
    let mut visited = BTreeSet::from([at]);
    let mut seen: BTreeMap<Pos, usize> = BTreeMap::new();
    let mut moved_count = 0usize;
    let mut backtrack_run = 0usize;
    {
        let look = Look { world: &world };
        r.start_ways = look.ways(at).len();
        r.start_ways_onward = look.ways_onward(at);
        observe(&look, at, 0, &mut seen, &mut r);
        r.snapshots.push(snapshot(&look, at, seed, 0, &visited));
    }

    for n in 1..=a.steps {
        let Some(d) = policy.choose(&world, at, &visited) else {
            r.boxed_in += 1;
            continue;
        };
        // The game's step: Rust checks the move, moves, grows (DR-0044).
        let unknown_before: Vec<Pos> = in_view(step(at, d))
            .filter(|&p| world.get(p).is_none())
            .collect();
        let patches_before = world.log().len();
        let t = Instant::now();
        let moved = world.joined(at, d);
        if moved {
            at = step(at, d);
            world.grow_around(at);
        }
        r.step_times.push(t.elapsed());
        r.patches_per_step.push(world.log().len() - patches_before);
        if !moved {
            continue; // The policy only picks joined ways; kept for safety.
        }
        moved_count += 1;
        if visited.insert(at) {
            if backtrack_run > 0 {
                r.backtracks.push(backtrack_run);
            }
            backtrack_run = 0;
        } else {
            backtrack_run += 1;
        }

        let look = Look { world: &world };
        let appeared: Vec<Pos> = unknown_before
            .into_iter()
            .filter(|&p| world.get(p).is_some())
            .collect();
        if !appeared.is_empty() {
            r.steps_with_appearance += 1;
            if appeared.iter().any(|&p| chebyshev(p, at) < VIEW) {
                r.steps_with_inner_appearance += 1;
            }
        }
        for &p in &appeared {
            r.appeared_by_distance[chebyshev(p, at) as usize] += 1;
            if look.kind(p) != Some(TileKind::Hedge) {
                r.appeared_walkable += 1;
            }
        }
        r.ways_onward.push(look.ways_onward(at));
        r.unknown_in_view
            .push(in_view(at).filter(|&p| world.get(p).is_none()).count());
        observe(&look, at, n, &mut seen, &mut r);
        if look.is_room(at) {
            let first = look.room_at(at)[0];
            if let Some(room) = r.rooms.get_mut(&first) {
                room.entered_at.get_or_insert(n);
            }
        }
        if n % a.every == 0 {
            r.snapshots
                .push(snapshot(&look, at, seed, moved_count, &visited));
        }
    }
    if backtrack_run > 0 {
        r.backtracks.push(backtrack_run);
    }
    r.moved = moved_count;

    let look = Look { world: &world };
    let mut ends: Vec<usize> = seen
        .keys()
        .filter(|&&p| look.is_dead_end(p))
        .filter_map(|&p| look.dead_end_length(p))
        .collect();
    ends.sort_unstable();
    r.dead_ends = ends;
    r.stored = world.len();
    r.hedge_stored = world
        .cells()
        .filter(|&(p, _)| look.kind(p) == Some(TileKind::Hedge))
        .count();
    r.patches = world.log().len();
    r.has_way_onward = world.has_way_onward();
    r
}

/// What the player sees at step `n`: choices, rooms, and the check that
/// no cell already seen has changed.
fn observe(look: &Look, at: Pos, n: usize, seen: &mut BTreeMap<Pos, usize>, r: &mut WalkReport) {
    let (mut junction, mut doorway, mut room) = (false, false, false);
    for p in in_view(at) {
        let Some(v) = look.world.get(p) else { continue };
        match seen.insert(p, v) {
            Some(before) if before != v => r.changed += 1,
            _ => {}
        }
        junction |= look.is_junction(p);
        doorway |= look.is_doorway(p);
        if look.is_room(p) {
            room = true;
            let cells = look.room_at(p);
            r.rooms.entry(cells[0]).or_insert(RoomSeen {
                cells: cells.len(),
                seen_at: n,
                entered_at: None,
            });
        }
    }
    if n > 0 {
        r.junction_in_view += junction as usize;
        r.doorway_in_view += doorway as usize;
        r.room_in_view += room as usize;
        r.choice_in_view += (junction || doorway || room) as usize;
    }
}

/// The 13 × 13 view as DR-0045 draws it, in text: DR-0017 glyphs two
/// characters per cell, `@` for the player, `::` for not yet known.
/// The trail can't show in plain text; visited cells are counted instead.
fn snapshot(look: &Look, at: Pos, seed: u32, steps: usize, visited: &BTreeSet<Pos>) -> String {
    let mut s = String::new();
    let trail = in_view(at).filter(|p| visited.contains(p)).count();
    let _ = writeln!(
        s,
        "Seed {seed} · {} · {steps} {} · {trail} trodden cells in view",
        position_text(at),
        if steps == 1 { "step" } else { "steps" }
    );
    let rule = "-".repeat((2 * VIEW as usize + 1) * 2);
    let _ = writeln!(s, "+{rule}+");
    for dy in -VIEW..=VIEW {
        s.push('|');
        for dx in -VIEW..=VIEW {
            let p = (at.0 + dx, at.1 + dy);
            match look.tile(p) {
                None => s.push_str(if p == at { "@ " } else { "::" }),
                Some((id, o, _)) => {
                    let g = glyph(id, o);
                    s.push(if p == at { '@' } else { g });
                    s.push(east_filler(g));
                }
            }
        }
        s.push_str("|\n");
    }
    let _ = writeln!(s, "+{rule}+");
    s
}

/// DR-0045 point 1's words: "3 east, 5 north of the start".
fn position_text(at: Pos) -> String {
    let mut parts = Vec::new();
    if at.0 != 0 {
        parts.push(format!(
            "{} {}",
            at.0.abs(),
            if at.0 > 0 { "east" } else { "west" }
        ));
    }
    if at.1 != 0 {
        parts.push(format!(
            "{} {}",
            at.1.abs(),
            if at.1 > 0 { "south" } else { "north" }
        ));
    }
    if parts.is_empty() {
        "at the start".into()
    } else {
        format!("{} of the start", parts.join(", "))
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

fn quantile<T: Copy + Ord>(values: &[T], q: f64) -> Option<T> {
    let mut v = values.to_vec();
    v.sort_unstable();
    if v.is_empty() {
        return None;
    }
    let i = ((q * v.len() as f64).ceil() as usize).clamp(1, v.len()) - 1;
    Some(v[i])
}

fn ms(d: Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

fn report_walk(r: &WalkReport, out: &mut String) {
    let steps = r.ways_onward.len();
    let _ = writeln!(out, "## Seed {}\n", r.seed);
    let _ = writeln!(out, "```text");
    let _ = writeln!(
        out,
        "{} steps asked, {} moved, {} with nowhere to go; {} cells stored in {} patches; way onward at the end: {}",
        r.steps,
        r.moved,
        r.boxed_in,
        r.stored,
        r.patches,
        if r.has_way_onward { "yes" } else { "NO" }
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "1. Start: {} ways out of the start cell, {} of them lead onward within the view.",
        r.start_ways, r.start_ways_onward
    );
    let first_seen = r.rooms.values().map(|m| m.seen_at).min();
    let first_entered = r.rooms.values().filter_map(|m| m.entered_at).min();
    let entered = r.rooms.values().filter(|m| m.entered_at.is_some()).count();
    let never: Vec<String> = r
        .rooms
        .values()
        .filter(|m| m.entered_at.is_none())
        .map(|m| format!("{} cells, seen at step {}", m.cells, m.seen_at))
        .collect();
    let _ = writeln!(
        out,
        "2. Rooms: {} seen, {} entered. First seen at step {}, first entered at step {}.",
        r.rooms.len(),
        entered,
        first_seen.map_or("never".into(), |n| n.to_string()),
        first_entered.map_or("never".into(), |n| n.to_string())
    );
    if !never.is_empty() {
        let _ = writeln!(out, "   Seen, never entered: {}.", never.join("; "));
    }
    let sizes: Vec<String> = r.rooms.values().map(|m| m.cells.to_string()).collect();
    if !sizes.is_empty() {
        let _ = writeln!(out, "   Room sizes in cells: {}.", sizes.join(", "));
    }
    let _ = writeln!(
        out,
        "3. 2+ ways onward in view on {} of steps. Any junction, doorway or room in view on {} \
         (junction {}, doorway {}, room {}).",
        pct(r.ways_onward.iter().filter(|&&w| w >= 2).count(), steps),
        pct(r.choice_in_view, steps),
        pct(r.junction_in_view, steps),
        pct(r.doorway_in_view, steps),
        pct(r.room_in_view, steps)
    );
    let mut onward = [0usize; 5];
    for &w in &r.ways_onward {
        onward[w.min(4)] += 1;
    }
    let _ = writeln!(
        out,
        "   Ways onward in view, by steps: 0: {}, 1: {}, 2: {}, 3+: {}.",
        pct(onward[0], steps),
        pct(onward[1], steps),
        pct(onward[2], steps),
        pct(onward[3] + onward[4], steps)
    );
    let long = r.dead_ends.iter().filter(|&&l| l > LONG_DEAD_END).count();
    let _ = writeln!(
        out,
        "4. Dead ends seen: {}; lengths (cells to the junction) {}; {} longer than {}.",
        r.dead_ends.len(),
        list(&r.dead_ends),
        long,
        LONG_DEAD_END
    );
    let walked_back: usize = r.backtracks.iter().sum();
    let _ = writeln!(
        out,
        "   Walking back over trodden cells: {} runs, {} steps ({} of moves); longest {}, median {}.",
        r.backtracks.len(),
        walked_back,
        pct(walked_back, r.moved),
        r.backtracks.iter().max().copied().unwrap_or(0),
        quantile(&r.backtracks, 0.5).unwrap_or(0)
    );
    let _ = writeln!(
        out,
        "5. Boxed in: {} steps with nowhere to go; the world's way onward check: {}.",
        r.boxed_in,
        if r.has_way_onward { "holds" } else { "FAILS" }
    );
    let appeared: usize = r.appeared_by_distance.iter().sum();
    let _ = writeln!(
        out,
        "6. Seen cells that changed: {} (must be 0).",
        r.changed
    );
    let _ = writeln!(
        out,
        "   Cells appearing inside the view: on {} of steps ({} steps closer in than the rim); {} cells, {} walkable.",
        pct(r.steps_with_appearance, steps),
        r.steps_with_inner_appearance,
        appeared,
        r.appeared_walkable
    );
    let by: Vec<String> = r
        .appeared_by_distance
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0)
        .map(|(d, c)| format!("{d}: {c}"))
        .collect();
    let _ = writeln!(
        out,
        "   By distance from the player (6 is the rim): {}.",
        if by.is_empty() {
            "none".into()
        } else {
            by.join(", ")
        }
    );
    let view_cells = ((2 * VIEW + 1) * (2 * VIEW + 1)) as usize;
    let _ = writeln!(
        out,
        "7. (Judged by eye from the snapshots below.) Not yet known in view: median {} of {} cells; hedge cells stored: {}.",
        quantile(&r.unknown_in_view, 0.5).unwrap_or(0),
        view_cells,
        r.hedge_stored
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Time: new world {}; a step (move and grow) median {}, p95 {}, max {}; patches per step median {}, max {}.",
        ms(r.start_time),
        ms(quantile(&r.step_times, 0.5).unwrap_or_default()),
        ms(quantile(&r.step_times, 0.95).unwrap_or_default()),
        ms(r.step_times.iter().max().copied().unwrap_or_default()),
        quantile(&r.patches_per_step, 0.5).unwrap_or(0),
        r.patches_per_step.iter().max().copied().unwrap_or(0)
    );
    let _ = writeln!(out, "```\n");
    let _ = writeln!(out, "### Snapshots\n");
    let _ = writeln!(out, "```text");
    for s in &r.snapshots {
        let _ = writeln!(out, "{s}");
    }
    let _ = writeln!(out, "```\n");
}

fn list(values: &[usize]) -> String {
    if values.is_empty() {
        return "none".into();
    }
    let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
    for &v in values {
        *counts.entry(v).or_default() += 1;
    }
    counts
        .iter()
        .map(|(v, c)| {
            if *c == 1 {
                v.to_string()
            } else {
                format!("{v}×{c}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let a = args();
    let set = TileSet::from_toml(CORE).expect("core.toml loads");
    let table = Table::calculate(&set);
    let walks: Vec<WalkReport> = a
        .seeds
        .iter()
        .map(|&s| walk_one(&set, &table, s, &a))
        .collect();

    let mut out = String::new();
    let seeds = a
        .seeds
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let command = format!(
        "cargo run --release -p maze-game --example fun_walk -- --seeds {} --steps {} --every {}",
        a.seeds
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(","),
        a.steps,
        a.every
    );
    if a.out.is_some() {
        let _ = writeln!(
            out,
            "---\ntype: Report\ntitle: G.1 fun walks, scripted walks for DR-0044's checklist\n\
             description: A seeded player walks grown worlds under DR-0044's rules for seeds {seeds}, \
             {} steps each; counts for each checklist point, 13 by 13 snapshots and step times, for Wren to judge.\n\
             status: draft\nowners: [engineer-ash/1]\ntags: [report, game, slice-g-1, fun, measures]\n\
             generated:\n  by: engineer-ash/1\n  at: {}\n---\n",
            a.steps,
            now_utc()
        );
        let _ = writeln!(out, "# G.1 fun walks\n");
    }
    let _ = writeln!(
        out,
        "Made by `{command}`. Counts are seeded and repeat exactly; times are this machine's, release build.\n\n\
         **The player.** Starts at the origin, steps only into stored, joined cells, and the world grows \
         around it after every move (DR-0044 rules 1 to 3). It's a curious, thorough player: it takes an \
         unvisited way when there is one (a seeded draw), otherwise it walks back by the shortest path to the \
         nearest cell with an unvisited way or an open endpoint. So it walks into every dead end and every room \
         cell, which makes the dead-end and walking-back counts a worst case. It never bumps a hedge, so blocked \
         moves aren't measured here.\n\n\
         **Terms.** A *way* is a joined neighbour. *Ways onward in view*: of the player's ways, those that reach \
         the view's rim or an open endpoint within the 13 by 13 view without coming back through the player. \
         A *choice* is a junction (a corridor cell with 3 or 4 ways), a room doorway or any room cell in view. \
         A *dead end* is a corridor cell with one walkable side, joined; its length is the cells walked from it \
         to the first junction, room or doorway (dead ends that run into the unknown aren't counted). \
         *Appearing* cells were unknown in the view the player stepped into and stored after the growth that \
         step caused. Snapshots draw DR-0017's glyphs two characters per cell, `@` for the player and `::` for \
         not yet known; the trail can't show in text, so each snapshot counts the trodden cells in view.\n"
    );
    let _ = writeln!(out, "## Across the seeds\n");
    let _ = writeln!(out, "```text");
    let _ = writeln!(
        out,
        "seed  start ways  rooms seen/entered  1st room entered  2+ ways onward  dead ends >{LONG_DEAD_END}  walked back  appeared (steps)  changed  step p95"
    );
    for r in &walks {
        let steps = r.ways_onward.len();
        let _ = writeln!(
            out,
            "{:<5} {:>10}  {:>18}  {:>16}  {:>14}  {:>12}  {:>11}  {:>16}  {:>7}  {:>8}",
            r.seed,
            format!("{}/{}", r.start_ways_onward, r.start_ways),
            format!(
                "{}/{}",
                r.rooms.len(),
                r.rooms.values().filter(|m| m.entered_at.is_some()).count()
            ),
            r.rooms
                .values()
                .filter_map(|m| m.entered_at)
                .min()
                .map_or("never".into(), |n| n.to_string()),
            pct(r.ways_onward.iter().filter(|&&w| w >= 2).count(), steps),
            format!(
                "{}/{}",
                r.dead_ends.iter().filter(|&&l| l > LONG_DEAD_END).count(),
                r.dead_ends.len()
            ),
            pct(r.backtracks.iter().sum(), r.moved),
            pct(r.steps_with_appearance, steps),
            r.changed,
            ms(quantile(&r.step_times, 0.95).unwrap_or_default())
        );
    }
    let _ = writeln!(out, "```\n");
    let _ = writeln!(
        out,
        "Start ways is onward/all. Walked back is the share of moves onto trodden cells. Appeared is the share \
         of steps where any cell appeared inside the view.\n"
    );
    if walks.iter().all(|r| r.hedge_stored == 0) {
        let _ = writeln!(
            out,
            "**No hedge cell is ever stored.** A patch keeps only the cells joined to its anchor \
             (DR-0036 rule 4), and a hedge tile has no walkable side, so it is never joined. Every gap \
             between paths in these walks is a cell not yet known, drawn `::`, which is how DR-0045 point 7's \
             haze will look on screen: the game will show no `░` at all.\n"
        );
    }
    for r in &walks {
        report_walk(r, &mut out);
    }

    match &a.out {
        None => print!("{out}"),
        Some(path) => {
            std::fs::write(path, &out).unwrap_or_else(|e| {
                eprintln!("can't write {path}: {e}");
                std::process::exit(1)
            });
            eprintln!("wrote {path}");
        }
    }
}

/// The time now as ISO 8601 UTC, for the report's frontmatter, without a
/// date library: days since 1970 to a civil date (Howard Hinnant's method).
fn now_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}
