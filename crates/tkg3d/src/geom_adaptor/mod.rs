// Rust translation of ModelingData/TKG3d/GeomAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The GeomAdaptor package: Geom curves and surfaces behind the Adaptor3d
//! interfaces, the swept and transformed adaptors, and the way back from an adapted
//! curve or surface to a Geom one.
mod curve;
mod curve_eval;
mod curve_eval_order;
mod curve_query;
mod curve_shape;
mod curve_state;
pub use curve::GeomAdaptorCurve;
mod surface;
mod surface_eval;
mod surface_eval_order;
mod surface_query;
mod surface_shape;
mod surface_span;
mod surface_state;
pub use surface::GeomAdaptorSurface;
pub use surface_state::LoadRange as GeomAdaptorSurfaceRange;
mod extrusion_kind;
mod surface_of_linear_extrusion;
pub use surface_of_linear_extrusion::GeomAdaptorSurfaceOfLinearExtrusion;
mod revolution_kind;
mod surface_of_revolution;
pub use surface_of_revolution::GeomAdaptorSurfaceOfRevolution;
mod transformed_surface;
pub use transformed_surface::GeomAdaptorTransformedSurface;
mod make;
pub use make::{make_curve, make_surface};
mod transformed_curve;
pub use transformed_curve::GeomAdaptorTransformedCurve;
