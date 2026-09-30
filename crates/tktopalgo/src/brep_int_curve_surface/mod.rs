// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepIntCurveSurface at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepIntCurveSurface: the intersections of a curve with the faces of a
//! shape. `BRepIntCurveSurface_Inter` is [`BRepIntCurveSurfaceInter`]
//! (docs/topalgo-design.md section 12): `IntCurveSurface_HInter` intersects the curve
//! with each face whose box meets it, and `BRepTopAdaptor_TopolTool` keeps the points
//! IN or ON the face.
mod find;
mod inter;

pub use inter::BRepIntCurveSurfaceInter;
