// Rust translation of ModelingAlgorithms/TKHLR/Intrv at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Intrv: parameter intervals with fuzzy bounds (`Interval`), their 13
//! relative positions (`Position`) and ordered sets of disjoint intervals with the
//! boolean operations (`Intervals`). HLRAlgo keeps the visible parts of an edge in an
//! `Intervals`.
mod interval;
mod interval_relations;
mod intervals;
mod position;

pub use interval::Interval;
pub use intervals::Intervals;
pub use position::Position;
