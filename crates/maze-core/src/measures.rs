//! DR-0030's two measures of "a way onward", from one flood fill.
//!
//! - Walkable cell: its tile is not the Hedge kind. A cell a failed attempt
//!   left open (`None`) is not walkable (Yew's addition to DR-0030).
//! - Two side-sharing cells are connected when both shared sides are open
//!   (not Hedge). Border sides lead nowhere.
//! - Centre: column ⌊W/2⌋, row ⌊H/2⌋; if that isn't walkable, the walkable
//!   cell nearest by Manhattan distance, ties to the lowest row, then the
//!   lowest column.

use crate::side::{Direction, SideKind};
use crate::tile_set::{TileKind, TileSet};
use crate::wfc::neighbour;

/// The numbers behind the measures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaCounts {
    /// (column, row) from the north-west. The geometric centre when no
    /// cell is walkable.
    pub centre: (usize, usize),
    pub walkable: usize,
    /// Walkable cells reachable from `centre`, counting itself; 0 when no
    /// cell is walkable.
    pub reachable_from_centre: usize,
    pub separate_areas: usize,
}

/// One measure as the tool shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Measure {
    pub name: &'static str,
    pub value: String,
    pub explanation: &'static str,
}

fn walkable(set: &TileSet, cell: Option<usize>) -> bool {
    cell.is_some_and(|v| set.tile_of(&set.variants[v]).kind != TileKind::Hedge)
}

fn connected(set: &TileSet, a: usize, d: Direction, b: usize) -> bool {
    set.variants[a].side(d) != SideKind::Hedge
        && set.variants[b].side(d.opposite()) != SideKind::Hedge
}

/// Counts areas and what the centre reaches. `cells` is row-major, each a
/// variant index into `set.variants` or `None`; panics if its length isn't
/// `width * height`.
pub fn count_areas(
    set: &TileSet,
    cells: &[Option<usize>],
    width: usize,
    height: usize,
) -> AreaCounts {
    assert_eq!(cells.len(), width * height, "cells don't fill the grid");
    let is_walkable: Vec<bool> = cells.iter().map(|&c| walkable(set, c)).collect();
    let walkable_count = is_walkable.iter().filter(|&&w| w).count();

    // Label each walkable cell with its area by flood fill.
    let mut area: Vec<Option<usize>> = vec![None; cells.len()];
    let mut areas = 0;
    for start in 0..cells.len() {
        if !is_walkable[start] || area[start].is_some() {
            continue;
        }
        area[start] = Some(areas);
        let mut stack = vec![start];
        while let Some(i) = stack.pop() {
            let here = cells[i].expect("walkable cells are collapsed");
            for d in Direction::ALL {
                let Some(j) = neighbour(width, height, i, d) else {
                    continue;
                };
                if !is_walkable[j] || area[j].is_some() {
                    continue;
                }
                let there = cells[j].expect("walkable cells are collapsed");
                if connected(set, here, d, there) {
                    area[j] = Some(areas);
                    stack.push(j);
                }
            }
        }
        areas += 1;
    }

    let middle = (width / 2, height / 2);
    // Row-major order visits lower rows, then lower columns, first, so the
    // first cell at the least distance is DR-0030's tie-break.
    let centre = (0..cells.len())
        .filter(|&i| is_walkable[i])
        .map(|i| (i % width, i / width))
        .min_by_key(|&(x, y)| x.abs_diff(middle.0) + y.abs_diff(middle.1))
        .unwrap_or(middle);
    let reachable_from_centre = match area[centre.1 * width + centre.0] {
        Some(a) => area.iter().filter(|&&x| x == Some(a)).count(),
        None => 0,
    };

    AreaCounts {
        centre,
        walkable: walkable_count,
        reachable_from_centre,
        separate_areas: areas,
    }
}

const NO_WALKABLE: &str = "No walkable cells";

/// DR-0030's wording for the two measures.
pub fn measures(counts: &AreaCounts) -> Vec<Measure> {
    let (reachable, areas) = if counts.walkable == 0 {
        (NO_WALKABLE.to_string(), NO_WALKABLE.to_string())
    } else {
        // Nearest whole percent, halves rounded up, in integers.
        let percent =
            (counts.reachable_from_centre * 200 + counts.walkable) / (counts.walkable * 2);
        (
            format!(
                "{} of {} walkable cells ({percent}%)",
                counts.reachable_from_centre, counts.walkable
            ),
            counts.separate_areas.to_string(),
        )
    };
    vec![
        Measure {
            name: "Reachable from centre",
            value: reachable,
            explanation: "Share of walkable cells the player could walk to from the middle of the maze.",
        },
        Measure {
            name: "Separate areas",
            value: areas,
            explanation: "Groups of walkable cells cut off from each other. 1 means everything joins up.",
        },
    ]
}
