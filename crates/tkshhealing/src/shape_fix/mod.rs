// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeFix: the fixing tools that correct the problems that ShapeAnalysis
//! detects. docs/shhealing-design.md section 4 fixes how its classes map;
//! docs/tkshhealing-progress.md lists the ported classes.
mod edge_connect;
mod face;
mod face_connect;
mod root;
mod shape;
mod shape_tolerance;
mod wire;

pub use edge_connect::ShapeFixEdgeConnect;
pub use face::ShapeFixFace;
pub use face_connect::ShapeFixFaceConnect;
pub use root::{FixRoot, FixRootData, FixRootFields, ShapeFixRoot, need_fix};
pub use shape::ShapeFixShape;
pub use shape_tolerance::ShapeFixShapeTolerance;
pub use wire::ShapeFixWire;

mod wire_segment;
pub use wire_segment::{ShapeFixWireSegment, WirePatchIndex};
