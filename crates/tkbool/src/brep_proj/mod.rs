// Rust translation of ModelingAlgorithms/TKBool/BRepProj/BRepProj_Projection.hxx/.cxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepProj`: the cylindrical (along a direction) and conical (from a point)
//! projections of a wire on a shape, as the section of the shape by the swept wire.
mod projection;

pub use projection::BRepProjProjection;
