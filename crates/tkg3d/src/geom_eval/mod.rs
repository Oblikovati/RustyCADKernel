// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepCurveDesc.hxx, GeomEval_RepSurfaceDesc.hxx, GeomEval_RepUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parts of GeomEval that Geom includes: evaluation representation
//! descriptors of curves and surfaces and their evaluation helpers.
//!
//! `GeomEval_RepCurveDesc` is identical to `Geom2dEval_RepCurveDesc` except for
//! the curve class, so both expand `rustycad_tkg2d::define_rep_curve_descriptors`
//! and share `Map1d`, `Domain1d` and the kind enumeration.
mod rep_curve_desc;
pub use rep_curve_desc::{
    RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
};
pub use rustycad_tkg2d::geom2d_eval::{Domain1d, Map1d, RepCurveKind};
mod rep_surface_desc;
pub use rep_surface_desc::{
    Domain2d, Map2d, RepSurfaceDesc, RepSurfaceDescDerivBounded, RepSurfaceDescFull,
    RepSurfaceDescMapped, RepSurfaceKind,
};
mod eval_failures;
pub mod rep_utils;

// GeomEval curves.
mod aht_bezier_curve;
mod curve_support;
pub use aht_bezier_curve::GeomEvalAHTBezierCurve;
mod circular_helix_curve;
pub use circular_helix_curve::GeomEvalCircularHelixCurve;
mod sine_wave_curve;
pub use sine_wave_curve::GeomEvalSineWaveCurve;
mod tbezier_curve;
pub use tbezier_curve::GeomEvalTBezierCurve;

// GeomEval elementary surfaces.
mod elementary_eval;
mod ellipsoid_surface;
pub use ellipsoid_surface::GeomEvalEllipsoidSurface;
mod hyperboloid_surface;
pub use hyperboloid_surface::{GeomEvalHyperboloidSurface, HyperboloidSheetMode};
mod paraboloid_surface;
pub use paraboloid_surface::GeomEvalParaboloidSurface;
mod hyp_paraboloid_surface;
pub use hyp_paraboloid_surface::GeomEvalHypParaboloidSurface;
mod circular_helicoid_surface;
pub use circular_helicoid_surface::GeomEvalCircularHelicoidSurface;

// GeomEval Bezier surfaces.
mod aht_bezier_surface;
pub use aht_bezier_surface::GeomEvalAHTBezierSurface;
mod bezier_grid;
mod bezier_results;
mod tbezier_surface;
pub use tbezier_surface::GeomEvalTBezierSurface;
