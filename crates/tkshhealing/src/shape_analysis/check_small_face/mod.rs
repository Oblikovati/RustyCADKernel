//! Face size checks used by the small-face healing pipeline.
mod check_single_strip;
mod check_strip_face;
mod find_strip_edges;
mod pin;
mod pin_edges;
mod splitting_vertices;
mod spot;
mod strip;
mod strip_edges;

pub use splitting_vertices::{SplittingVertexEdges, SplittingVertexParameters};
pub use spot::ShapeAnalysisCheckSmallFace;
