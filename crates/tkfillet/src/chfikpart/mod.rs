// Rust translation of the ChFiKPart helpers (ChFiKPart_ComputeData_Fcts.cxx,
// ChFiKPart_ComputeData_CS.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ChFiKPart: the fillets and chamfers of the particular cases computed in closed form.
//! The helpers that do not write into the TopOpeBRepDS data structure are ported; the
//! `ChFiKPart_ComputeData` cases and the `Make*` functions wait for TKBool
//! (docs/tkfillet-progress.md).
mod corner_spine;
mod fcts;

pub use corner_spine::corner_spine;
pub use fcts::{in_period, pcurve, proj_pc};
