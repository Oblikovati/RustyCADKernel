// Rust translation of ModelingAlgorithms/TKPrim/BRepPreviewAPI at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package BRepPreviewAPI: the preview box maker, which builds a vertex, an
//! edge or a face when the box is flat in some directions
//! (docs/prim-design.md section 5).
mod make_box;

pub use make_box::BRepPreviewAPIMakeBox;
