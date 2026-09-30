// Rust translation of ModelingAlgorithms/TKPrim/Sweep at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package Sweep: the indexed directing shapes of the swept primitives: a numbered
//! edge or vertex ([`SweepNumShape`]), its tool ([`SweepNumShapeTool`]) and its
//! iterator ([`SweepNumShapeIterator`]). BRepSweep sweeps a TopoDS shape along such a
//! directing edge (docs/prim-design.md section 1).
mod num_shape;
mod num_shape_iterator;
mod num_shape_tool;

pub use num_shape::SweepNumShape;
pub use num_shape_iterator::SweepNumShapeIterator;
pub use num_shape_tool::SweepNumShapeTool;
