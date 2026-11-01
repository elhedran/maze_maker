//! Numbers for Wren's 2.3 tuning pass: opening pressure against damping
//! corner-into-corner (2.1 review, tuning idea 4). Prints Markdown.
//!
//! `cargo run --release -p maze-core --example tuning`
//!
//! Each variant is `core.toml` with lines added in memory; the file itself
//! is never touched. Generation and measures are the tool's own
//! (`generate`, `count_areas`), at the tool's default 24 × 16.

use maze_core::wfc::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use maze_core::{Direction, Generation, SideKind, Table, TileKind, TileSet, count_areas, generate};

const CORE: &str = include_str!("../../../data/tile-sets/core.toml");

/// Wren's 2.1 review seeds, then 20 more.
const SEEDS: [u32; 26] = [
    1, 2, 3, 4, 5, 48213, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
    25,
];

/// The damping exception: corner beside corner, passage to passage, both
/// ways round, as "Long straights are rarer than turns" does for straights.
fn damping(multiplier: f64) -> String {
    format!(
        "\n[[exception]]\nname = \"Corner into corner is rarer\"\n\
         a = {{ tile = \"corridor_corner\", side = \"passage\" }}\n\
         b = {{ tile = \"corridor_corner\", side = \"passage\" }}\n\
         multiplier = {multiplier:?}\n"
    )
}

/// `core.toml` with `opening_pressure` (if not 1) and the damping
/// exception (if any).
fn variant(pressure: f64, damp: Option<f64>) -> (TileSet, Table) {
    assert!(
        !CORE.contains("opening_pressure"),
        "core.toml has a pressure now"
    );
    let mut text = CORE.to_string();
    if pressure != 1.0 {
        text = text.replace(
            "name = \"core\"\n",
            &format!("name = \"core\"\nopening_pressure = {pressure:?}\n"),
        );
    }
    if let Some(m) = damp {
        text.push_str(&damping(m));
    }
    let set = TileSet::from_toml(&text).unwrap_or_else(|e| panic!("refused: {e:?}"));
    let table = Table::calculate(&set);
    (set, table)
}

struct Row {
    reach: Vec<f64>,
    areas: Vec<usize>,
    restarts: u32,
    failed: usize,
}

fn run(set: &TileSet, table: &Table) -> Row {
    let mut row = Row {
        reach: Vec::new(),
        areas: Vec::new(),
        restarts: 0,
        failed: 0,
    };
    for seed in SEEDS {
        let maze = generate(set, table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
        let c = count_areas(set, &maze.cells, maze.width, maze.height);
        let share = if c.walkable == 0 {
            0.0
        } else {
            100.0 * c.reachable_from_centre as f64 / c.walkable as f64
        };
        row.reach.push(share);
        row.areas.push(c.separate_areas);
        row.restarts += maze.restarts;
        row.failed += usize::from(maze.failed);
    }
    row
}

/// Each cell as 3 × 3 characters: `#` hedge, `+` a door, space walkable.
/// A corner is open only when both edges beside it are floor, so rooms
/// read as open ground and corridors as one-wide.
fn render(set: &TileSet, maze: &Generation) -> String {
    let (w, h) = (maze.width, maze.height);
    let mut grid = vec![vec!['?'; w * 3]; h * 3];
    for (i, cell) in maze.cells.iter().enumerate() {
        let (x, y) = (i % w * 3, i / w * 3);
        let Some(v) = cell else { continue };
        let variant = &set.variants[*v];
        let hedge_tile = set.tile_of(variant).kind == TileKind::Hedge;
        let side = |d: Direction| variant.side(d);
        let edge = |s: SideKind| match s {
            SideKind::Hedge => '#',
            SideKind::Door => '+',
            _ => ' ',
        };
        let floor =
            |s: SideKind| !matches!(s, SideKind::Hedge | SideKind::Passage | SideKind::Door);
        let corner = |a: Direction, b: Direction| {
            if floor(side(a)) && floor(side(b)) {
                ' '
            } else {
                '#'
            }
        };
        use Direction::*;
        grid[y][x] = corner(North, West);
        grid[y][x + 1] = edge(side(North));
        grid[y][x + 2] = corner(North, East);
        grid[y + 1][x] = edge(side(West));
        grid[y + 1][x + 1] = if hedge_tile { '#' } else { ' ' };
        grid[y + 1][x + 2] = edge(side(East));
        grid[y + 2][x] = corner(South, West);
        grid[y + 2][x + 1] = edge(side(South));
        grid[y + 2][x + 2] = corner(South, East);
    }
    grid.into_iter()
        .map(|r| r.into_iter().collect::<String>().trim_end().to_string() + "\n")
        .collect()
}

fn main() {
    let variants: Vec<(String, f64, Option<f64>)> = vec![
        ("pressure 1 (today)".into(), 1.0, None),
        ("pressure 0.75".into(), 0.75, None),
        ("pressure 0.5".into(), 0.5, None),
        ("pressure 0.35".into(), 0.35, None),
        ("pressure 0.25".into(), 0.25, None),
        ("damping 0.5".into(), 1.0, Some(0.5)),
        ("damping 0.25".into(), 1.0, Some(0.25)),
        ("pressure 0.5 + damping 0.5".into(), 0.5, Some(0.5)),
    ];
    println!(
        "| Variant | Reachable from centre: mean | min–max | Separate areas: mean | min–max | Restarts | Failed |"
    );
    println!("|---|---|---|---|---|---|---|");
    let mut per_seed = Vec::new();
    for (name, pressure, damp) in &variants {
        let (set, table) = variant(*pressure, *damp);
        let row = run(&set, &table);
        let n = SEEDS.len() as f64;
        let mean_r = row.reach.iter().sum::<f64>() / n;
        let (min_r, max_r) = row
            .reach
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let mean_a = row.areas.iter().sum::<usize>() as f64 / n;
        let (min_a, max_a) = (
            row.areas.iter().min().unwrap(),
            row.areas.iter().max().unwrap(),
        );
        println!(
            "| {name} | {mean_r:.0}% | {min_r:.0}–{max_r:.0}% | {mean_a:.1} | {min_a}–{max_a} | {} | {} |",
            row.restarts, row.failed
        );
        per_seed.push((name.clone(), row));
    }

    println!("\n### Per seed: reachable % / separate areas\n");
    print!("| Seed |");
    for (name, _) in &per_seed {
        print!(" {name} |");
    }
    println!();
    print!("|---|");
    for _ in &per_seed {
        print!("---|");
    }
    println!();
    for (k, seed) in SEEDS.iter().enumerate() {
        print!("| {seed} |");
        for (_, row) in &per_seed {
            print!(" {:.0}% / {} |", row.reach[k], row.areas[k]);
        }
        println!();
    }

    for seed in [1, 3, 48213] {
        for (name, pressure, damp) in [
            ("pressure 1 (today)", 1.0, None),
            ("pressure 0.5", 0.5, None),
            ("damping 0.5", 1.0, Some(0.5)),
        ] {
            let (set, table) = variant(pressure, damp);
            let maze = generate(&set, &table, seed, DEFAULT_WIDTH, DEFAULT_HEIGHT).unwrap();
            let c = count_areas(&set, &maze.cells, maze.width, maze.height);
            println!(
                "\n#### Seed {seed}, {name}: {} of {} walkable reachable from the centre ({}, {}), {} separate areas\n",
                c.reachable_from_centre, c.walkable, c.centre.0, c.centre.1, c.separate_areas
            );
            println!("```text\n{}```", render(&set, &maze));
        }
    }
}
