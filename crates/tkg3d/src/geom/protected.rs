// Rust translation of ModelingData/TKG3d/Geom (protected members of the abstract classes, see docs/geom-design.md) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Proof that a caller is inside this crate, for the native `protected` members
//! that abstract classes share with their subclasses (`Geom_Vector::gpVec`,
//! `Geom_Conic::pos`, `Geom_ElementarySurface::pos`...). Other crates cannot
//! build a value of it.

/// The access token of the protected trait members; only this crate builds one.
pub struct ProtectedAccess {
    _private: (),
}
pub(crate) const PROTECTED: ProtectedAccess = ProtectedAccess { _private: () };
