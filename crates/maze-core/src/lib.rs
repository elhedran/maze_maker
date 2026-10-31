//! Maze Maker's domain logic: tiles, the adjacency table, table checks, WFC
//! generation (DR-0028) and measures of a generated maze (DR-0030).
//!
//! This crate is a plain library (DR-0012): it never depends on Tauri or a UI
//! crate, and does no I/O. Callers hand it data and get values back.

pub mod measures;
pub mod noise;
pub mod side;
pub mod table;
pub mod tile_set;
pub mod wfc;

pub use measures::{AreaCounts, Measure, count_areas, measures};
pub use side::{BasePair, Direction, SideKind, SideMatch};
pub use table::{Entry, Table};
pub use tile_set::{Exception, LoadError, Noise, Tile, TileKind, TileSet, Variant};
pub use wfc::{GenerateError, Generation, generate};

/// The version of this crate, as set in its `Cargo.toml`.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_matches_manifest() {
        assert_eq!(version(), "0.1.0");
    }
}
