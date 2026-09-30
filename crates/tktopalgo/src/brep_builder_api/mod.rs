// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepBuilderAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepBuilderAPI. Not ported yet: its classes land in this module when
//! the package is ported (docs/tktopalgo-progress.md lists the ported packages).
//!
//! STUB: [`MakeVertex`] is a minimal stand-in for `BRepBuilderAPI_MakeVertex`, kept
//! only because the TKernel test `crates/tkernel/tests/toploc_location.rs` and the
//! TKernel geometry probe build vertices through it. It is not the port of the
//! package and has no `api` row; the real port of BRepBuilderAPI replaces it.
mod make_vertex_stub;

pub use make_vertex_stub::MakeVertex;

// The sewing group of BRepBuilderAPI (docs/topalgo-design.md section 11).
mod bnd_box_tree_selector;
mod fast_sewing;
mod sewing;
mod vertex_inspector;

pub use bnd_box_tree_selector::BRepBuilderAPIBndBoxTreeSelector;
pub use fast_sewing::{BRepBuilderAPIFastSewing, FastSewingStatus, FastSewingStatuses};
pub use sewing::{BRepBuilderAPISewing, SewingShapeIndexedMap, SewingShapeListMap, SewingShapeMap};
pub use vertex_inspector::BRepBuilderAPIVertexInspector;
