//! Face size checks used by the small-face healing pipeline.
mod check_single_strip;
mod find_strip_edges;
mod spot;
mod strip;
mod strip_edges;

pub use spot::ShapeAnalysisCheckSmallFace;
