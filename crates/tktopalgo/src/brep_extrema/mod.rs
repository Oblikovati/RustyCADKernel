// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepExtrema: the minimum distance between shapes and the proximity
//! tools on their triangulations (docs/topalgo-design.md section 13).
//!
//! - [`BRepExtremaDistShapeShape`] finds the minimum distance between two shapes with
//!   its solutions, by [`BRepExtremaDistanceSS`] on every pair of sub-shapes, with the
//!   extrema classes [`BRepExtremaExtPC`], [`BRepExtremaExtPF`], [`BRepExtremaExtCC`],
//!   [`BRepExtremaExtCF`] and [`BRepExtremaExtFF`]; [`BRepExtremaPoly`] compares
//!   triangulation nodes.
//! - [`BRepExtremaTriangleSet`] holds the triangles of faces (and the segments of
//!   edge polygons) of a shape under a BVH; [`BRepExtremaOverlapTool`],
//!   [`BRepExtremaSelfIntersection`], [`BRepExtremaProximityDistTool`],
//!   [`BRepExtremaProximityValueTool`] and [`BRepExtremaShapeProximity`] traverse it.
mod dist_shape_shape;
mod distance_ss;
mod ext_cc;
mod ext_cf;
mod ext_ff;
mod ext_pc;
mod ext_pf;
mod extrema_capture;
mod poly;
mod solution_elem;
mod support_type;

pub use dist_shape_shape::BRepExtremaDistShapeShape;
pub use distance_ss::BRepExtremaDistanceSS;
pub use ext_cc::BRepExtremaExtCC;
pub use ext_cf::BRepExtremaExtCF;
pub use ext_ff::BRepExtremaExtFF;
pub use ext_pc::BRepExtremaExtPC;
pub use ext_pf::BRepExtremaExtPF;
pub use poly::{BRepExtremaPoly, BRepExtremaPolyDistance};
pub use solution_elem::BRepExtremaSolutionElem;
pub use support_type::{BRepExtremaSupportType, uncompatible_shape};
