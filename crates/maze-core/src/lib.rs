//! Maze Maker's domain logic: tiles, the adjacency table, table checks and,
//! later, WFC generation.
//!
//! This crate is a plain library (DR-0012): it never depends on Tauri or a UI
//! crate, and does no I/O. Callers hand it data and get values back.

pub mod noise;
pub mod side;
pub mod table;
pub mod tile_set;

pub use side::{BasePair, Direction, SideKind, SideMatch};
pub use table::{Entry, Table};
pub use tile_set::{Exception, LoadError, Noise, Tile, TileKind, TileSet, Variant};

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
