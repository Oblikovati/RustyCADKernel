// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeFix: the fixing tools that correct the problems that ShapeAnalysis
//! detects. docs/shhealing-design.md section 4 fixes how its classes map;
//! docs/tkshhealing-progress.md lists the ported classes.
mod edge_connect;
mod root;
mod shape_tolerance;

pub use edge_connect::ShapeFixEdgeConnect;
pub use root::{FixRoot, FixRootData, FixRootFields, ShapeFixRoot, need_fix};
pub use shape_tolerance::ShapeFixShapeTolerance;
