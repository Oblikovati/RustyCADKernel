//! IMeshData_Status and IMeshData_StatusOwner, including exact-mask comparisons.
use rustycad_tkernel::standard::SharedState;
/// Native status values; `Status::ReMesh.0 == 8`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Status(pub i32);
#[allow(non_upper_case_globals)]
impl Status {
    /// Successful generation.
    pub const NoError: Self = Self(0);
    /// Open boundary.
    pub const OpenWire: Self = Self(1);
    /// Boundary self intersection.
    pub const SelfIntersectingWire: Self = Self(2);
    /// Mesh generation failed.
    pub const Failure: Self = Self(4);
    /// Deflection reduced after interference.
    pub const ReMesh: Self = Self(8);
    /// Incorrect boundary orientation.
    pub const UnorientedWire: Self = Self(16);
    /// Insufficient boundary points.
    pub const TooFewPoints: Self = Self(32);
    /// Existing triangulation too coarse.
    pub const Outdated: Self = Self(64);
    /// Existing triangulation retained.
    pub const Reused: Self = Self(128);
    /// User cancellation.
    pub const UserBreak: Self = Self(256);
}
/// Shared status bitmask; `StatusOwner::default().is_equal(Status::NoError)`.
#[derive(Default)]
pub struct StatusOwner {
    mask: SharedState<i32>,
}
impl StatusOwner {
    /// Exact comparison; `owner.is_equal(Status::ReMesh)`.
    pub fn is_equal(&self, value: Status) -> bool {
        self.get_status_mask() == value.0
    }
    /// Any common bit; `owner.is_set(Status::OpenWire)`.
    pub fn is_set(&self, value: Status) -> bool {
        self.get_status_mask() & value.0 != 0
    }
    /// Add bits; `owner.set_status(Status::Failure)`.
    pub fn set_status(&self, value: Status) {
        *self.mask.write() |= value.0;
    }
    /// Remove bits; `owner.unset_status(Status::Failure)`.
    pub fn unset_status(&self, value: Status) {
        *self.mask.write() &= !(value.0);
    }
    /// Complete mask; `owner.get_status_mask()`.
    pub fn get_status_mask(&self) -> i32 {
        *self.mask.read()
    }
}
