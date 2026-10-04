// Rust translation of ModelingAlgorithms/TKShHealing/ShapeConstruct at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeConstruct: curve adjustment, B-spline conversion and projection.
//! Curve construction and the native sampled surface projector share the geometric foundations.
mod curve;
pub use curve::{
    ShapeConstructCurve, ShapeConstructCurveJoin2d, ShapeConstructCurveJoin2dRequest,
    ShapeConstructCurveJoin3d,
};

mod project_curve_on_surface;
pub use project_curve_on_surface::ShapeConstructProjectCurveOnSurface;
