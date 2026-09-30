// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntStart at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntStart: the two templates that find the start points of a walking
//! on a surface domain, on its restrictions (`IntStart_SearchOnBoundaries.gxx`) and
//! inside it (`IntStart_SearchInside.gxx`). The package has no header of its own:
//! IntPatch and Contap instantiate the templates. Each template is one Rust generic
//! over the tool traits of [`tools`] (docs/geomalgo-design.md section 12).
pub(crate) mod failures;
mod search_inside;
mod search_on_boundaries;
pub mod tools;

pub use search_inside::SearchInside;
pub use search_on_boundaries::SearchOnBoundaries;
pub use tools::{
    SearchInsideTypes, SearchOnBoundariesTypes, TheArc, TheArcFunction, TheInsideFunction,
    ThePathPoint, TheSOBTool, TheSegment, TheTopolTool, TheVertex,
};
