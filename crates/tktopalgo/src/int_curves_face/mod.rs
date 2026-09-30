// Rust translation of ModelingAlgorithms/TKTopAlgo/IntCurvesFace at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package IntCurvesFace: the intersection of a line or a 3d curve with a face
//! ([`IntCurvesFaceIntersector`]) or with the faces of a shape
//! ([`IntCurvesFaceShapeIntersector`]), through IntCurveSurface_HInter and the
//! BRepTopAdaptor classification of the UV points (docs/topalgo-design.md section 4).
mod internal_call;
mod intersector;
mod perform;
mod sample_pars;
mod shape_intersector;

pub use intersector::IntCurvesFaceIntersector;
pub use shape_intersector::IntCurvesFaceShapeIntersector;
