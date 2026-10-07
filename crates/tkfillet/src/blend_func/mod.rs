// Rust translation of ModelingAlgorithms/TKFillet/BlendFunc at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! BlendFunc: the blend functions between two surfaces (constant and evolutive radius
//! fillets, chamfers) and their inverses, with the package functions that shape a section.
//!
//! Every function implements the [`crate::blend`] traits. The native caches of
//! `ComputeValues` are kept, including the `static thread_local` scratch variables shared
//! by all objects of a class in a thread.
mod ch_asym;
mod ch_asym_inv;
mod chamf_inv;
mod chamfer;
mod chamfer_macro;
mod chamfer_plane;
pub(crate) mod choice;
pub(crate) mod const_rad;
mod const_rad_inv;
mod const_throat;
mod const_throat_inv;
mod corde;
mod evol_rad;
mod evol_rad_inv;
pub(crate) mod fillet_inverse;
mod gen_chamf_inv;
pub(crate) mod gen_chamfer;
mod guide;
mod inverse_macro;
pub(crate) mod linalg;
mod normal_terms;
pub(crate) mod package;
mod section_solve;
mod tensor;
#[cfg(test)]
mod test_fixtures;
pub(crate) mod two_surfaces;

pub use ch_asym::ChAsym;
pub use ch_asym_inv::ChAsymInv;
pub use chamf_inv::ChamfInv;
pub use chamfer::Chamfer;
pub use const_rad::ConstRad;
pub use const_rad_inv::ConstRadInv;
pub use const_throat::{ConstThroat, ConstThroatWithPenetration};
pub use const_throat_inv::{ConstThroatInv, ConstThroatWithPenetrationInv};
pub use corde::Corde;
pub use evol_rad::EvolRad;
pub use evol_rad_inv::EvolRadInv;
pub use gen_chamf_inv::GenChamfInvFunction;
pub use gen_chamfer::GenChamferFunction;
pub use package::{
    SectionCounts, SectionShape, compute_d_normal, compute_normal, get_minimal_weights, get_shape,
    next_shape,
};
pub use tensor::Tensor;
