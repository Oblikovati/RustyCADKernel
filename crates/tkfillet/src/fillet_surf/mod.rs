// Rust translation of FilletSurf_StatusType.hxx, FilletSurf_StatusDone.hxx and
// FilletSurf_ErrorTypeStatus.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! FilletSurf: the fillet surfaces of a contour without building the solid
//! ([`FilletSurfBuilder`]), computed by a ChFi3d fillet builder that only walks the
//! contour ([`FilletSurfInternalBuilder`]).
mod builder;
mod internal_add;
mod internal_builder;
mod internal_ops;

pub use builder::FilletSurfBuilder;
pub use internal_builder::FilletSurfInternalBuilder;

/// Where the ends of a fillet surface lie (`FilletSurf_StatusType`);
/// `StatusType::NoExtremityOnEdge as i32 == 2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusType {
    /// `FilletSurf_TwoExtremityOnEdge`: both ends on edges of the faces.
    TwoExtremityOnEdge = 0,
    /// `FilletSurf_OneExtremityOnEdge`: one end on an edge.
    OneExtremityOnEdge = 1,
    /// `FilletSurf_NoExtremityOnEdge`: no end on an edge.
    NoExtremityOnEdge = 2,
}

/// The result of the computation (`FilletSurf_StatusDone`);
/// `StatusDone::IsPartial as i32 == 2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusDone {
    /// `FilletSurf_IsOk`.
    IsOk = 0,
    /// `FilletSurf_IsNotOk`.
    IsNotOk = 1,
    /// `FilletSurf_IsPartial`: some fillets are computed.
    IsPartial = 2,
}

/// Why the computation failed (`FilletSurf_ErrorTypeStatus`);
/// `ErrorTypeStatus::PbFilletCompute as i32 == 5`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorTypeStatus {
    /// `FilletSurf_EmptyList`: no edge given.
    EmptyList = 0,
    /// `FilletSurf_EdgeNotG1`: the edges are not G1.
    EdgeNotG1 = 1,
    /// `FilletSurf_FacesNotG1`: the faces are not G1.
    FacesNotG1 = 2,
    /// `FilletSurf_EdgeNotOnShape`: an edge is not on the shape.
    EdgeNotOnShape = 3,
    /// `FilletSurf_NotSharpEdge`: an edge is not sharp.
    NotSharpEdge = 4,
    /// `FilletSurf_PbFilletCompute`: the fillet computation failed.
    PbFilletCompute = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_follow_the_native_order() {
        assert_eq!(StatusType::NoExtremityOnEdge as i32, 2);
        assert_eq!(StatusDone::IsNotOk as i32, 1);
        assert_eq!(ErrorTypeStatus::NotSharpEdge as i32, 4);
    }
}
