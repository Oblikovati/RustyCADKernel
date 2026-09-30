// Rust translation of ModelingAlgorithms/TKTopAlgo/BRepExtrema/BRepExtrema_SupportType.hxx and
// BRepExtrema_UnCompatibleShape.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepExtrema_SupportType`, the kind of sub-shape that carries a solution point, and
//! the `BRepExtrema_UnCompatibleShape` exception.
use rustycad_tkernel::standard::Failure;

/// `BRepExtrema_SupportType`: a solution point is a vertex, lies on an edge or lies
/// inside a face; `assert_eq!(BRepExtremaSupportType::IsInFace as i32, 2)`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum BRepExtremaSupportType {
    /// `BRepExtrema_IsVertex`.
    #[default]
    IsVertex = 0,
    /// `BRepExtrema_IsOnEdge`.
    IsOnEdge = 1,
    /// `BRepExtrema_IsInFace`.
    IsInFace = 2,
}

/// `BRepExtrema_UnCompatibleShape` (a `Standard_DomainError`): a parameter accessor
/// asked for a solution whose support has another kind;
/// `uncompatible_shape("ParOnEdgeS1 is impossible without EDGE (support IsVertex)")`.
pub fn uncompatible_shape(message: &str) -> Failure {
    Failure::with_type("BRepExtrema_UnCompatibleShape", message, "")
}
