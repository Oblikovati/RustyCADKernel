// Rust translation of ModelingAlgorithms/TKGeomAlgo/TopTrans at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package TopTrans: the complex transitions at an intersection. A curve
//! transition (`CurveTransition`) accumulates the boundary curves that a curve crosses
//! at one point; a surface transition (`SurfaceTransition`) accumulates the boundary
//! faces that a surface crosses along one curve. Both answer the states just before
//! and just after the intersection.
mod curve_transition;
mod nearest;
mod surface_transition;

pub use curve_transition::CurveTransition;
pub use surface_transition::{SurfaceTransition, get_after, get_before};
