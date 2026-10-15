// Rust translation of the BRepFilletAPI package at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BRepFilletAPI: the fillet and chamfer algorithms of the API: the 3d fillets and
//! chamfers of the edges of a shape over the ChFi3d builders, the 2d fillets and
//! chamfers of a planar face, and the contour interface they share.
mod local_api;
mod local_operation;
mod make_chamfer;
mod make_fillet;
mod make_fillet2d;
#[cfg(test)]
mod tests;

pub use local_operation::BRepFilletAPILocalOperation;
pub use make_chamfer::BRepFilletAPIMakeChamfer;
pub use make_fillet::BRepFilletAPIMakeFillet;
pub use make_fillet2d::BRepFilletAPIMakeFillet2d;
