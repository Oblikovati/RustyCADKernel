// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_RepCurveDesc.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_RepCurveDesc`: descriptors that redirect the evaluation of a curve
//! to another curve (its "evaluation representation").
//!
//! The native descriptors expose public mutable fields through shared handles;
//! here each descriptor keeps them behind a lock with getters and setters.
rustycad_tkg2d::define_rep_curve_descriptors!(dyn crate::geom::GeomCurve);
