// Rust translation of ModelingData/TKG2d/Geom2d (protected member access) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Access control for the native `protected` members of the abstract Geom2d classes.
//!
//! The shared state of each object is a TKernel
//! [`SharedState`](rustycad_tkernel::standard::SharedState); see docs/geom-design.md,
//! section 3.
/// Proof that a caller is inside this crate, for the native `protected` members
/// that abstract classes share with their subclasses (`Geom2d_Vector::gpVec2d`,
/// `Geom2d_Conic::pos`). Other crates cannot build a value of it.
pub struct ProtectedAccess {
    _private: (),
}
pub(crate) const PROTECTED: ProtectedAccess = ProtectedAccess { _private: () };
