// Rust translation of ModelingAlgorithms/TKShHealing/ShapeFix at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeFix: the fixing tools that correct the problems that ShapeAnalysis
//! detects. docs/shhealing-design.md section 4 fixes how its classes map;
//! docs/tkshhealing-progress.md lists the ported classes.
mod edge;
mod edge_connect;
mod edge_proj_aux;
mod face;
mod face_connect;
mod face_loops;
mod intersection;
mod root;
mod shape;
mod shape_tolerance;
mod shell_orientation;
mod split_common_vertex;
mod wire;
mod wire_vertex;

pub use edge::ShapeFixEdge;
pub use edge_connect::ShapeFixEdgeConnect;
pub use edge_proj_aux::ShapeFixEdgeProjAux;
pub use face::{ShapeFixFace, ShapeFixFaceWireMap};
pub use face_connect::ShapeFixFaceConnect;
pub use intersection::{PcurveCrossing, line_pcurve_crossing};
pub use root::{FixRoot, FixRootData, FixRootFields, ShapeFixRoot, need_fix};
pub use shape::ShapeFixShape;
pub use shape_tolerance::ShapeFixShapeTolerance;
pub use split_common_vertex::ShapeFixSplitCommonVertex;
pub use wire::ShapeFixWire;
pub use wire_vertex::ShapeFixWireVertex;

mod wire_segment;
pub use wire_segment::{ShapeFixWireSegment, WirePatchIndex};

mod free_bounds;
pub use free_bounds::ShapeFixFreeBounds;
