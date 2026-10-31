//! The adjacency table, calculated from a tile set (DR-0013, DR-0022).
//!
//! For row variant R, neighbour (column) variant C and direction d, R's side
//! facing d meets C's side facing back:
//!
//! > weight = side-pair base × weight of R × weight of C × exception
//! > multiplier × noise factor
//!
//! and 0 when the sides may not meet. Everything that multiplies a 0 leaves
//! it 0, so the side rules always win (DR-0022 condition 1).
//!
//! `probability = weight / row total`, where the row total is over every
//! neighbour of R in direction d (DR-0024). A row whose total is 0 has
//! probability 0 everywhere, never NaN.

use crate::noise::{self, SideRef};
use crate::side::{Direction, SideKind};
use crate::tile_set::TileSet;

/// One entry and everything that went into it.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The row tile's side facing the neighbour.
    pub row_side: SideKind,
    /// The neighbour's side facing back.
    pub column_side: SideKind,
    pub allowed: bool,
    /// 0 when not allowed.
    pub base: f64,
    pub row_weight: f64,
    pub column_weight: f64,
    /// Index into `TileSet::exceptions`; at most one applies (DR-0022).
    /// Only set on allowed entries.
    pub exception: Option<usize>,
    /// 1 when there is no noise or the entry isn't allowed.
    pub noise: f64,
    pub weight: f64,
}

/// The whole table: every direction × row × column.
#[derive(Clone, Debug, PartialEq)]
pub struct Table {
    size: usize,
    entries: Vec<Entry>,
    row_totals: Vec<f64>,
}

impl Table {
    pub fn calculate(set: &TileSet) -> Table {
        let size = set.variants.len();
        let mut entries = Vec::with_capacity(4 * size * size);
        let mut row_totals = Vec::with_capacity(4 * size);
        for d in Direction::ALL {
            for row in &set.variants {
                let mut total = 0.0;
                for column in &set.variants {
                    let row_tile = set.tile_of(row);
                    let column_tile = set.tile_of(column);
                    let row_side = row.side(d);
                    let column_side = column.side(d.opposite());
                    let mut entry = Entry {
                        row_side,
                        column_side,
                        allowed: false,
                        base: 0.0,
                        row_weight: row_tile.weight,
                        column_weight: column_tile.weight,
                        exception: None,
                        noise: 1.0,
                        weight: 0.0,
                    };
                    if let Some(pair) = row_side.base_pair(column_side) {
                        entry.allowed = true;
                        entry.base = set.bases.get(pair);
                        entry.exception = set
                            .exceptions
                            .iter()
                            .position(|e| e.applies(row_tile, row_side, column_tile, column_side));
                        let multiplier = entry
                            .exception
                            .map_or(1.0, |i| set.exceptions[i].multiplier);
                        if let Some(n) = &set.noise {
                            entry.noise = noise::factor(
                                n,
                                SideRef {
                                    tile_id: &row_tile.id,
                                    orientation: row.orientation,
                                    direction: d,
                                },
                                SideRef {
                                    tile_id: &column_tile.id,
                                    orientation: column.orientation,
                                    direction: d.opposite(),
                                },
                            );
                        }
                        entry.weight = entry.base
                            * entry.row_weight
                            * entry.column_weight
                            * multiplier
                            * entry.noise;
                    }
                    total += entry.weight;
                    entries.push(entry);
                }
                row_totals.push(total);
            }
        }
        Table {
            size,
            entries,
            row_totals,
        }
    }

    /// Rows (and columns): the number of (tile, orientation) pairs.
    pub fn size(&self) -> usize {
        self.size
    }

    /// Panics if `row` or `column` is out of range.
    pub fn entry(&self, direction: Direction, row: usize, column: usize) -> &Entry {
        assert!(
            row < self.size && column < self.size,
            "table index out of range"
        );
        &self.entries[(direction.index() * self.size + row) * self.size + column]
    }

    pub fn row_total(&self, direction: Direction, row: usize) -> f64 {
        assert!(row < self.size, "table index out of range");
        self.row_totals[direction.index() * self.size + row]
    }

    /// The share of the row: the chance of this neighbour on this side given
    /// the row tile. 0 when the row total is 0.
    pub fn probability(&self, direction: Direction, row: usize, column: usize) -> f64 {
        let total = self.row_total(direction, row);
        if total > 0.0 {
            self.entry(direction, row, column).weight / total
        } else {
            0.0
        }
    }
}
