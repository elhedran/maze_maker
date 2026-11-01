//! 2.6 step 2: the measures of a grown world (DR-0036), as one short
//! plain-text report.
//!
//! cargo run --release -p maze-core --example measures -- [--walks N] [--steps N] [--seed N]
//!
//! Every count comes from seeded walks, so a run with the same arguments
//! reports the same counts; only the times vary with the machine.

use maze_core::world::{Pos, Settings, Stage, World};
use maze_core::world_measures::{
    Connectivity, Room, connectivity, edge_profile, explore, frontier, grid_cells, quantile, rooms,
    walk,
};
use maze_core::{Table, TileSet, count_areas, generate};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");
/// Pressures swept. DR-0036 rule 9 allows 0 < value ≤ 1, and 1 is off, so
/// there is nothing above 1 to try.
const PRESSURES: [f64; 5] = [1.0, 0.75, 0.5, 0.25, 0.1];
/// 2.1's default maze size, for the comparison in 6.
const MAZES: u32 = 50;

struct Args {
    walks: u32,
    steps: usize,
    seed: u32,
}

fn args() -> Args {
    let mut out = Args {
        walks: 6,
        steps: 40,
        seed: 1,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it
            .next()
            .unwrap_or_else(|| usage(&format!("{flag} needs a value")));
        let n: u64 = value
            .parse()
            .unwrap_or_else(|_| usage(&format!("{flag} {value}: not a whole number")));
        match flag.as_str() {
            "--walks" if (1..=10_000).contains(&n) => out.walks = n as u32,
            "--steps" if (1..=1_000_000).contains(&n) => out.steps = n as usize,
            "--seed" if n <= u64::from(u32::MAX) => out.seed = n as u32,
            _ => usage(&format!(
                "unknown flag or value out of range: {flag} {value}"
            )),
        }
    }
    out
}

fn usage(why: &str) -> ! {
    eprintln!(
        "{why}\nusage: measures [--walks N (default 6)] [--steps N (default 40)] [--seed N (default 1)]"
    );
    std::process::exit(2)
}

fn core(pressure: f64) -> (TileSet, Table) {
    let mut set = TileSet::from_toml(CORE).expect("core.toml loads");
    // Set here rather than in the file; PRESSURES keep to the loader's
    // range (0 < value ≤ 1).
    set.opening_pressure = pressure;
    let table = Table::calculate(&set);
    (set, table)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Walker {
    /// `explore`: jump to a random open endpoint each step.
    Explorer,
    /// Step 2's plain random walk (a `Walker` at revisit weight 1), one
    /// step at a time.
    Random,
}

/// What one configuration's walks gave.
#[derive(Default)]
struct Run {
    patches: usize,
    restarted: usize,
    attempts: u64,
    spine: usize,
    fallback: usize,
    /// Every patch's time, sorted, the walk's patches only (not those
    /// `World::new` grows at the start).
    times: Vec<Duration>,
    /// Per walk, (open, outward) at the start and after each step.
    frontier: Vec<Vec<(usize, usize)>>,
    rooms: Vec<Room>,
    profile: [(usize, usize); 4],
    connectivity: Vec<Connectivity>,
    stored: Vec<usize>,
}

struct Config {
    pressure: f64,
    patch_size: usize,
    stage: Stage,
    walker: Walker,
    walks: u32,
}

fn run(a: &Args, c: &Config) -> Run {
    let (set, table) = core(c.pressure);
    let settings = Settings {
        patch_size: c.patch_size,
        first_stage: c.stage,
        ..Settings::default()
    };
    let mut out = Run::default();
    for w in 0..c.walks {
        let seed = a.seed.wrapping_add(w);
        let walk_seed = u64::from(seed) * 7919 + 13;
        let mut world = World::new(&set, &table, seed, settings).expect("core passes");
        let start = frontier(&world);
        let mut series = vec![(start.open, start.outward)];
        let times = &mut out.times;
        let grow = |w: &mut World, anchor| {
            let t = Instant::now();
            w.grow_at(anchor);
            times.push(t.elapsed());
        };
        let after = |w: &World| {
            let f = frontier(w);
            series.push((f.open, f.outward));
        };
        match c.walker {
            Walker::Explorer => explore(&mut world, walk_seed, a.steps, grow, after),
            Walker::Random => {
                // Weight 1: step 2's plain random walk, draw for draw.
                let walker = maze_core::walker::Walker::with_revisit_weight((0, 0), walk_seed, 1.0)
                    .expect("1 is in range");
                walk(&mut world, walker, a.steps, grow, after);
            }
        }
        out.frontier.push(series);
        for p in world.log() {
            out.patches += 1;
            out.restarted += usize::from(p.attempts > 1);
            out.attempts += u64::from(p.attempts);
            out.spine += usize::from(p.spine);
            out.fallback += usize::from(p.fallback);
        }
        out.rooms.extend(rooms(&world));
        for (total, x) in out.profile.iter_mut().zip(edge_profile(&world)) {
            total.0 += x.0;
            total.1 += x.1;
        }
        let cells: BTreeMap<Pos, usize> = world.cells().collect();
        out.connectivity
            .push(connectivity(&set, &cells, (0, 0), true));
        out.stored.push(world.len());
    }
    out.times.sort();
    out
}

fn pct(n: usize, of: usize) -> String {
    if of == 0 {
        "-".into()
    } else {
        format!("{:.1}%", 100.0 * n as f64 / of as f64)
    }
}

fn us(d: Option<Duration>) -> String {
    d.map_or("-".into(), |d| format!("{:.0} µs", d.as_secs_f64() * 1e6))
}

fn median(mut v: Vec<usize>) -> usize {
    v.sort();
    quantile(&v, 0.5).unwrap_or(0)
}

fn min_med_end(series: &[Vec<(usize, usize)>], pick: impl Fn(&(usize, usize)) -> usize) -> String {
    let mut all: Vec<usize> = series.iter().flatten().map(&pick).collect();
    all.sort();
    let ends = series.iter().filter_map(|s| s.last().map(&pick)).collect();
    format!(
        "{}/{}/{}",
        all.first().copied().unwrap_or(0),
        quantile(&all, 0.5).unwrap_or(0),
        median(ends)
    )
}

fn main() {
    let a = args();
    let started = Instant::now();
    let explorer = |pressure, patch_size, stage, walks| Config {
        pressure,
        patch_size,
        stage,
        walker: Walker::Explorer,
        walks,
    };
    println!("Maze Maker 2.6 step 2: measures of a grown world (DR-0036)");
    println!(
        "core.toml, r = 8, R = 3. {} walks of {} steps, world seeds {} to {}. Counts are seeded;",
        a.walks,
        a.steps,
        a.seed,
        a.seed.wrapping_add(a.walks - 1)
    );
    println!(
        "times are this machine's. Explorer: each step the player jumps to a random open endpoint"
    );
    println!(
        "and the world grows around it. Random walk: a Walker at revisit weight 1, one cell a step."
    );

    // 1 and 2: rates and the frontier, swept over opening pressure.
    println!("\n1-2. Patches and the frontier, P = 16");
    println!(
        "  {:<18} {:>7} {:>9} {:>8} {:>6} {:>8} | {:>14} {:>17} {:>6} {:>7}",
        "walker, pressure",
        "patches",
        "restarted",
        "attempts",
        "spine",
        "fallback",
        "open min/med/end",
        "outward min/med/end",
        "closed",
        "stored"
    );
    let mut runs = Vec::new();
    let mut rows: Vec<(String, Config)> = PRESSURES
        .iter()
        .map(|&p| {
            (
                format!("explorer, {p}"),
                explorer(p, 16, Stage::Checked, a.walks),
            )
        })
        .collect();
    rows.push((
        "random walk, 1".into(),
        Config {
            walker: Walker::Random,
            ..explorer(1.0, 16, Stage::Checked, a.walks)
        },
    ));
    for (name, c) in &rows {
        let r = run(&a, c);
        let closed = r.frontier.iter().flatten().filter(|f| f.1 == 0).count();
        println!(
            "  {:<18} {:>7} {:>9} {:>8.3} {:>6} {:>8} | {:>16} {:>19} {:>6} {:>7}",
            name,
            r.patches,
            pct(r.restarted, r.patches),
            r.attempts as f64 / r.patches as f64,
            pct(r.spine, r.patches),
            pct(r.fallback, r.patches),
            min_med_end(&r.frontier, |f| f.0),
            min_med_end(&r.frontier, |f| f.1),
            closed,
            median(r.stored.clone()),
        );
        runs.push(r);
    }
    println!(
        "  restarted: patches needing more than one WFC run. attempts: mean WFC runs a patch."
    );
    println!(
        "  open, outward: endpoints (walkable sides facing unsettled cells) at the start and after"
    );
    println!("  every step; min and median over all of them, end = the median walk's last value.");
    println!(
        "  closed: steps after which no endpoint was outward (rule 6 says 0; step 1's tests check"
    );
    println!("  after every patch). stored: median cells at the end of a walk.");
    println!(
        "  \"closed\" is sampled after every step; the per-patch guarantee (rule 6 after every patch)"
    );
    println!("  is a step 1 test.");
    println!(
        "  Pressure sweep: Wren's value from 2.3 doesn't exist yet; re-run at it when set. Wren keeps"
    );
    println!("  world pressure at 1 for now.");
    let base = &runs[0];
    let trend: Vec<String> = base
        .frontier
        .iter()
        .map(|s| {
            let third = (s.len() / 3).max(1);
            let mean =
                |v: &[(usize, usize)]| v.iter().map(|f| f.1).sum::<usize>() as f64 / v.len() as f64;
            format!(
                "{:.0}>{:.0}",
                mean(&s[..third]),
                mean(&s[s.len() - third..])
            )
        })
        .collect();
    println!("  outward at pressure 1, explorer, mean of each walk's first third > last third:");
    println!("    {}", trend.join(" "));

    // 3: rooms at P = 16 and P = 32.
    let big = run(&a, &explorer(1.0, 32, Stage::Checked, a.walks));
    println!(
        "\n3. Rooms, explorer, pressure 1. A room: room-kind cells joined through floor sides."
    );
    println!(
        "  {:>3} {:>6} {:>16} {:>22} {:>8} | room share of stored cells, by cells in from the window's edge",
        "P", "rooms", "cells med/p90/max", "4-9|10-19|20-49|50+", "on edge"
    );
    println!(
        "  {:>3} {:>6} {:>16} {:>22} {:>8} | {:>7} {:>7} {:>7} {:>7}",
        "", "", "", "", "", "0 (ring)", "1", "2", "3+"
    );
    for (size, r) in [(16, base), (32, &big)] {
        let mut sizes: Vec<usize> = r.rooms.iter().map(|x| x.cells).collect();
        sizes.sort();
        let bucket =
            |lo: usize, hi: usize| sizes.iter().filter(|&&s| (lo..=hi).contains(&s)).count();
        let spans = r.rooms.iter().filter(|x| x.spans_patches).count();
        let share: Vec<String> = r
            .profile
            .iter()
            .map(|&(room, all)| pct(room, all))
            .collect();
        println!(
            "  {:>3} {:>6} {:>16} {:>22} {:>8} | {:>8} {:>7} {:>7} {:>7}",
            size,
            r.rooms.len(),
            format!(
                "{}/{}/{}",
                quantile(&sizes, 0.5).unwrap_or(0),
                quantile(&sizes, 0.9).unwrap_or(0),
                sizes.last().copied().unwrap_or(0)
            ),
            format!(
                "{}|{}|{}|{}",
                bucket(0, 9),
                bucket(10, 19),
                bucket(20, 49),
                bucket(50, usize::MAX)
            ),
            pct(
                r.rooms.iter().filter(|x| x.touches_edge).count(),
                r.rooms.len()
            ),
            share[0],
            share[1],
            share[2],
            share[3],
        );
        if spans > 0 {
            println!("  !! {spans} rooms span more than one patch, which rule 1 says can't happen");
        }
    }
    println!(
        "  on edge: rooms with a cell on their patch window's outer ring. If rooms avoid patch"
    );
    println!(
        "  edges, the room share is lower at 0 than further in. No room spans two patches (checked)."
    );

    // 4: time per patch, forced stages, and the rule 6 check's growth.
    println!("\n4. Time per patch (release build)");
    let few = a.walks.min(3);
    let spine = run(&a, &explorer(1.0, 16, Stage::Spine, few));
    let fallback = run(&a, &explorer(1.0, 16, Stage::Fallback, few));
    for (name, r) in [
        ("P = 16", base),
        ("P = 32", &big),
        ("P = 16, spine forced", &spine),
        ("P = 16, fallback forced", &fallback),
    ] {
        println!(
            "  {:<24} median {:>8}  p99 {:>8}  max {:>8}  ({} patches)",
            name,
            us(quantile(&r.times, 0.5)),
            us(quantile(&r.times, 0.99)),
            us(r.times.last().copied()),
            r.times.len()
        );
    }

    // The rule 6 check on one longer explorer walk, timed at checkpoints.
    let (set, table) = core(1.0);
    let mut world = World::new(&set, &table, a.seed, Settings::default()).expect("core passes");
    let (checkpoints, per) = (6, a.steps * 2);
    let mut rows = Vec::new();
    for k in 0..checkpoints {
        let grow = |w: &mut World, anchor| {
            w.grow_at(anchor);
        };
        explore(&mut world, u64::from(a.seed) * 31 + k, per, grow, |_| {});
        let cells: Vec<Pos> = world.cells().map(|(p, _)| p).collect();
        let span = |f: fn(&Pos) -> i64| {
            cells.iter().map(f).max().unwrap_or(0) - cells.iter().map(f).min().unwrap_or(0)
        };
        let area = ((span(|p| p.0) + 3) * (span(|p| p.1) + 3)) as usize;
        let mut ts: Vec<Duration> = (0..9)
            .map(|_| {
                let t = Instant::now();
                assert!(std::hint::black_box(world.has_way_onward()), "rule 6 holds");
                t.elapsed()
            })
            .collect();
        ts.sort();
        rows.push(((k as usize + 1) * per, world.len(), area, ts[4]));
    }
    println!(
        "  the rule 6 check (the reference flood, run after every attempt) as one explorer walk grows:"
    );
    println!(
        "    {:>6} {:>8} {:>10} {:>9} {:>14}",
        "steps", "stored", "box cells", "check", "ns a box cell"
    );
    for &(steps, stored, area, t) in &rows {
        println!(
            "    {:>6} {:>8} {:>10} {:>9} {:>14.1}",
            steps,
            stored,
            area,
            us(Some(t)),
            t.as_secs_f64() * 1e9 / area as f64
        );
    }
    let ns = rows
        .last()
        .map_or(1.0, |&(_, _, area, t)| t.as_secs_f64() * 1e9 / area as f64)
        .max(1e-3);
    // Rule 8's worst path: R + 1 = 4 tries of up to 21 runs each, a check
    // after each try's completed run, then 21 spine runs and the fallback.
    // The code checks only the 4 tries; a fifth check is counted for
    // margin (Yew's review), so the bound only gets looser.
    let slowest = base.times.last().copied().unwrap_or_default();
    let check = rows.last().map(|r| r.3).unwrap_or_default();
    let fall = fallback.times.last().copied().unwrap_or_default();
    println!(
        "  forced worst case, estimated from parts: 105 runs at the slowest patch ({}), 5 checks",
        us(Some(slowest))
    );
    println!(
        "    at the last box ({}) and the slowest fallback patch ({}): {:.0} ms.",
        us(Some(check)),
        us(Some(fall)),
        (slowest * 105 + check * 5 + fall).as_secs_f64() * 1e3
    );
    println!(
        "    A conservative upper bound: it mixes scales (the slowest patch comes from small worlds,"
    );
    println!(
        "    the check time from the largest one), and counts a fifth check for margin (the code"
    );
    println!("    checks the 4 tries; the spine stage's run isn't checked).");
    println!(
        "  the check grows with the box around everything stored: at {ns:.1} ns a cell it reaches 1 ms"
    );
    println!(
        "    at about {:.0}k box cells, a {:.0} × {:.0} explored square.",
        1e6 / ns / 1e3,
        (1e6 / ns).sqrt(),
        (1e6 / ns).sqrt()
    );

    // 5: memory.
    let m = world.memory();
    let k = world.len() as f64 / 1000.0;
    println!(
        "\n5. Memory, the longer walk's world: {} stored cells in {} pages of 32 × 32 ({:.0}% full)",
        world.len(),
        m.pages,
        100.0 * world.len() as f64 / (m.pages as f64 * 1024.0)
    );
    println!(
        "  per 1,000 stored cells: table {:.1} KB, growth log {:.1} KB, palette {:.2} KB; total {:.1} KB",
        m.table as f64 / k / 1024.0,
        m.log as f64 / k / 1024.0,
        m.palette as f64 / k / 1024.0,
        (m.table + m.log + m.palette) as f64 / k / 1024.0
    );
    println!(
        "  (the log keeps every patch's list of stored cells at 16 bytes a cell; replay needs only"
    );
    println!("  the anchors.) Map overhead in the table is an estimate.");

    // 6: connectivity against 2.1.
    println!(
        "\n6. Connectivity, against Wren's 2.1 finding (8-88% reachable, 5-19 separate areas)"
    );
    let (set, table) = core(1.0);
    let mut fixed = Vec::new();
    for s in 0..MAZES {
        let g = generate(&set, &table, a.seed.wrapping_add(s), 24, 16).expect("default size");
        let centre = count_areas(&set, &g.cells, g.width, g.height).centre;
        let cells = grid_cells(&g.cells, g.width);
        fixed.push(connectivity(
            &set,
            &cells,
            (centre.0 as i64, centre.1 as i64),
            false,
        ));
    }
    println!(
        "  {:<26} {:>17} {:>9} {:>15} {:>11}",
        "", "reachable med/min/max", "areas med", "dead ends /100", "loops /100"
    );
    let random = runs.last().expect("random walk row");
    let label_21 = format!("2.1, 24 × 16, {MAZES} seeds");
    for (name, cs) in [
        ("explorer, P = 16", &base.connectivity),
        ("explorer, P = 32", &big.connectivity),
        ("random walk, P = 16", &random.connectivity),
        (label_21.as_str(), &fixed),
    ] {
        let mut reach: Vec<usize> = cs
            .iter()
            .map(|c| (100 * c.reachable).checked_div(c.walkable).unwrap_or(0))
            .collect();
        reach.sort();
        let reachable: usize = cs.iter().map(|c| c.reachable).sum();
        let per100 = |n: usize| 100.0 * n as f64 / reachable.max(1) as f64;
        println!(
            "  {:<26} {:>21} {:>9} {:>15.1} {:>11.1}",
            name,
            format!(
                "{}%/{}%/{}%",
                quantile(&reach, 0.5).unwrap_or(0),
                reach.first().copied().unwrap_or(0),
                reach.last().copied().unwrap_or(0)
            ),
            median(cs.iter().map(|c| c.areas).collect()),
            per100(cs.iter().map(|c| c.dead_ends).sum()),
            per100(cs.iter().map(|c| c.loops).sum()),
        );
    }
    println!(
        "  reachable: share of walkable (not hedge-kind) cells joined to the start: the origin in a"
    );
    println!(
        "  grown world, DR-0030's centre in 2.1. dead ends: one joined neighbour and nothing left to"
    );
    println!(
        "  grow from. loops: independent cycles of the reachable area. Both per 100 reachable cells."
    );

    println!("\nRan in {:.1} s.", started.elapsed().as_secs_f64());
}
