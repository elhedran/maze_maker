//! Maze Maker's domain logic: tiles, the adjacency table, table checks
//! (including the completeness check for endless mazes), WFC generation
//! (DR-0028), measures of a generated maze (DR-0030), the trace of one
//! cell's draw (DR-0034), and the endless world, stored and grown from
//! open endpoints (DR-0035, DR-0036).
//!
//! This crate is a plain library (DR-0012): it never depends on Tauri or a UI
//! crate, and does no I/O. Callers hand it data and get values back.

pub mod completeness;
pub mod measures;
pub mod noise;
pub mod side;
pub mod table;
pub mod tile_set;
pub mod trace;
pub mod wfc;
pub mod world;

pub use measures::{AreaCounts, Measure, count_areas, measures};
pub use side::{BasePair, Direction, SideKind, SideMatch};
pub use table::{Entry, Table};
pub use tile_set::{Exception, LoadError, Noise, Tile, TileKind, TileSet, Variant};
pub use trace::{
    CandidateScore, CellTrace, CollapseTrace, NeighbourAtDraw, PairFactor, TraceError, trace_cell,
};
pub use wfc::{CollapseStep, GenerateError, Generation, Observer, generate, generate_observed};

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
