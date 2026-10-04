//! Intersection data structures from OCCT IntTools at revision 3d097a0.
mod base_range_sample;
mod curve_range_sample;
mod range;
mod root;
mod surface_range_sample;
pub use base_range_sample::BaseRangeSample;
pub use curve_range_sample::CurveRangeSample;
pub use range::Range;
pub use root::Root;
pub use surface_range_sample::SurfaceRangeSample;
mod marked_range_set;
pub use marked_range_set::MarkedRangeSet;
mod common_prt;
mod curve;
mod pnt_on_face;
mod pnt_on_two_faces;
pub use common_prt::CommonPrt;
pub use curve::Curve;
pub use pnt_on_face::PntOnFace;
pub use pnt_on_two_faces::PntOn2Faces;

mod curve_range_localize;
pub use curve_range_localize::CurveRangeLocalizeData;
mod surface_range_localize;
pub use surface_range_localize::SurfaceRangeLocalizeData;
mod topol_tool;
pub use topol_tool::TopolTool;
mod fclass2d;
pub use fclass2d::FClass2d;
mod context;
pub use context::{Cached, Context};
mod tools;
pub use tools::Tools;
mod wline_tool;
pub use wline_tool::WLineTool;

mod face_face;
pub use face_face::FaceFace;

mod shrunk_range;
pub use shrunk_range::ShrunkRange;

mod edge_edge;
pub use edge_edge::EdgeEdge;

mod algorithms;
pub use algorithms::IntTools;

mod bean_face;
pub use bean_face::BeanFaceIntersector;

mod edge_face;
pub use edge_face::EdgeFace;
