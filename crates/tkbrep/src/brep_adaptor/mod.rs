// Rust translation of ModelingData/TKBRep/BRepAdaptor at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepAdaptor: edges, p-curves, faces and wires seen by the geometric algorithms.
//!
//! [`BRepAdaptorCurve`] (an edge as a 3D curve), [`BRepAdaptorCurve2d`] (an edge on a
//! face as a 2D curve) and [`BRepAdaptorSurface`] (a face as a surface) embed the
//! GeomAdaptor class they derive from natively and forward its members;
//! [`BRepAdaptorCompCurve`] is a wire seen as one 3D curve.
mod comp_curve;
mod curve;
mod curve2d;
mod forward;
mod surface;

pub use comp_curve::{BRepAdaptorCompCurve, CompCurveEdge};
pub use curve::BRepAdaptorCurve;
pub use curve2d::BRepAdaptorCurve2d;
pub use surface::BRepAdaptorSurface;
