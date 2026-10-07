// Rust translation of ModelingAlgorithms/TKFillet/BlendFunc at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BlendFunc: the blend functions between two surfaces (constant and evolutive radius
//! fillets, chamfers) and their inverses, with the package functions that shape a section.
//!
//! Every function implements the [`crate::blend`] traits. The native caches of
//! `ComputeValues` are kept, including the `static thread_local` scratch variables shared
//! by all objects of a class in a thread.
mod choice;
pub(crate) mod const_rad;
mod const_rad_inv;
mod evol_rad;
mod evol_rad_inv;
pub(crate) mod fillet_inverse;
mod guide;
pub(crate) mod linalg;
mod normal_terms;
pub(crate) mod package;
mod section_solve;
mod tensor;
pub(crate) mod two_surfaces;

pub use const_rad::ConstRad;
pub use const_rad_inv::ConstRadInv;
pub use evol_rad::EvolRad;
pub use evol_rad_inv::EvolRadInv;
pub use package::{
    SectionCounts, SectionShape, compute_d_normal, compute_normal, get_minimal_weights, get_shape,
    next_shape,
};
pub use tensor::Tensor;
