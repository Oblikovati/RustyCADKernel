// Rust translation of ModelingData/TKG2d/Geom2dEval at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2dEval package: evaluation representation descriptors (which Geom2d
//! includes) and the eval curves, which are defined by a closed-form
//! parameterization and refuse reversal and transformation.
mod rep_curve_desc;
pub use rep_curve_desc::{
    Domain1d, Map1d, MappedCurveState, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull,
    RepCurveDescMapped, RepCurveKind,
};
pub mod rep_utils;

pub mod aht_bezier_data;
mod analytic_dump;
pub mod bezier_curve_data;
pub mod eval_basis;
mod eval_math;
pub mod tbezier_data;

mod aht_bezier_curve;
pub use aht_bezier_curve::Geom2dEvalAHTBezierCurve;
mod archimedean_spiral_curve;
pub use archimedean_spiral_curve::Geom2dEvalArchimedeanSpiralCurve;
mod circle_involute_curve;
pub use circle_involute_curve::Geom2dEvalCircleInvoluteCurve;
mod logarithmic_spiral_curve;
pub use logarithmic_spiral_curve::Geom2dEvalLogarithmicSpiralCurve;
mod sine_wave_curve;
pub use sine_wave_curve::Geom2dEvalSineWaveCurve;
mod tbezier_curve;
pub use tbezier_curve::Geom2dEvalTBezierCurve;
