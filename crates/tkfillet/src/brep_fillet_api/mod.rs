// Rust translation of the BRepFilletAPI package at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepFilletAPI: the fillet and chamfer algorithms of the API. The 2d algorithm on
//! planar faces and the contour interface of the 3d algorithms are ported; the 3d
//! `BRepFilletAPI_MakeFillet` and `BRepFilletAPI_MakeChamfer` wait for the ChFi3d
//! builders (docs/tkfillet-progress.md).
mod local_operation;
mod make_fillet2d;
#[cfg(test)]
mod tests;

pub use local_operation::BRepFilletAPILocalOperation;
pub use make_fillet2d::BRepFilletAPIMakeFillet2d;
