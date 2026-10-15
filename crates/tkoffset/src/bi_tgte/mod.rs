// Rust translation of the BiTgte package of ModelingAlgorithms/TKOffset at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BiTgte`: the rolling-ball blend (`BiTgte_Blend`) with its curve on edge adaptor
//! (`BiTgte_CurveOnEdge`) and contact types (`BiTgte_ContactType`).
mod blend;
mod curve_on_edge;

pub use blend::BiTgteBlend;
pub use curve_on_edge::BiTgteCurveOnEdge;

/// `BiTgte_ContactType`: the kinds of the two shapes a blend surface touches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum BiTgteContactType {
    /// `BiTgte_FaceFace`.
    FaceFace,
    /// `BiTgte_FaceEdge`.
    FaceEdge,
    /// `BiTgte_FaceVertex`.
    FaceVertex,
    /// `BiTgte_EdgeEdge`.
    EdgeEdge,
    /// `BiTgte_EdgeVertex`.
    EdgeVertex,
    /// `BiTgte_VertexVertex`.
    VertexVertex,
}
