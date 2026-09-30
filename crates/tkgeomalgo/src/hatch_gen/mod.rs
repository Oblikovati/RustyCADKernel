// Rust translation of ModelingAlgorithms/TKGeomAlgo/HatchGen (HatchGen_ErrorStatus.hxx,
// HatchGen_IntersectionType.hxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package HatchGen: the intersection points and domains that a generic hatcher
//! (Geom2dHatch) computes on its hatchings.
//!
//! [`IntersectionPoint`] is the abstract native base of [`PointOnElement`] and
//! [`PointOnHatching`]; they embed it and reach its members through `Deref`, and
//! its pure virtual `Dump` is the trait [`IntersectionPointDump`]. The `Dump`
//! members write the native `std::cout` text to a `fmt::Write`.
mod domain;
mod intersection_point;
mod point_on_element;
mod point_on_hatching;
pub(crate) mod text;

pub use domain::Domain;
pub use intersection_point::{IntersectionPoint, IntersectionPointDump};
pub use point_on_element::PointOnElement;
pub use point_on_hatching::PointOnHatching;

/// HatchGen_ErrorStatus: the status of a hatching after the trimming and the
/// domain computation; `ErrorStatus::IncompatibleStates as i32 == 4`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ErrorStatus {
    /// HatchGen_NoProblem.
    #[default]
    NoProblem,
    /// HatchGen_TrimFailure: an intersection with an element failed.
    TrimFailure,
    /// HatchGen_TransitionFailure: a global transition could not be computed.
    TransitionFailure,
    /// HatchGen_IncoherentParity: the states of the points do not alternate.
    IncoherentParity,
    /// HatchGen_IncompatibleStates: the states around a point do not match.
    IncompatibleStates,
}

/// HatchGen_IntersectionType: the kind of intersection between the hatching and an
/// element; `IntersectionType::Undetermined as i32 == 3`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum IntersectionType {
    /// HatchGen_TRUE: a crossing inside the element.
    True,
    /// HatchGen_TOUCH: a crossing at an end of the element.
    Touch,
    /// HatchGen_TANGENT: the hatching is tangent to the element.
    Tangent,
    /// HatchGen_UNDETERMINED.
    #[default]
    Undetermined,
}
