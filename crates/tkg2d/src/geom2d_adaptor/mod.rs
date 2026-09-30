// Rust translation of ModelingData/TKG2d/Geom2dAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The Geom2dAdaptor package: Geom2d curves behind the Adaptor2d interface, and the
//! way back from an adapted curve to a Geom2d curve.
mod curve;
mod curve_eval;
mod curve_eval_order;
mod curve_query;
mod curve_shape;
mod curve_state;
pub mod knot_continuity;
mod make_curve;
pub mod span_cache;
pub use curve::Geom2dAdaptorCurve;
pub use make_curve::make_curve;
