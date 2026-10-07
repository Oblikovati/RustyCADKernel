// Rust translation of ModelingAlgorithms/TKFillet/ChFiDS at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! ChFiDS: the data structures of the fillet and chamfer builders.
//!
//! The value classes (`ChFiDS_CommonPoint`, `ChFiDS_FaceInterference`, ...) are plain
//! structs whose `Change*` accessors are `&mut` accessors. The transient classes that the
//! builders share and mutate through several handles (`ChFiDS_SurfData`,
//! `ChFiDS_Stripe`, `ChFiDS_Spine` and its subclasses) are reached through [`Shared`]
//! handles; `ChFiDS_ElSpine` is a curve adaptor, a `Handle<ElSpine>` with its state behind
//! a lock like every adaptor. The inheritance `ChFiDS_Spine` -> `ChFiDS_FilSpine` /
//! `ChFiDS_ChamfSpine` is the [`SpineObject`] trait with `Deref` to the [`Spine`] part.
mod chamf_spine;
mod circ_section;
mod common_point;
mod el_spine;
mod enums;
pub(crate) mod errors;
mod face_interference;
mod fil_spine;
mod maps;
mod regul;
mod shared;
mod spine;
mod stripe;
mod surf_data;

pub use chamf_spine::ChamfSpine;
pub use circ_section::CircSection;
pub use common_point::CommonPoint;
pub use el_spine::ElSpine;
pub use enums::{ChamfMethod, ChamfMode, ErrorStatus, State, TypeOfConcavity};
pub use face_interference::FaceInterference;
pub use fil_spine::FilSpine;
pub use maps::{Map, StripeMap};
pub use regul::Regul;
pub use shared::{Shared, lock, same_object, shared};
pub use spine::{Spine, SpineHandle, SpineObject};
pub use stripe::{Stripe, StripeHandle, SurfDataSequenceHandle};
pub use surf_data::{SurfData, SurfDataHandle};
