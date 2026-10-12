//! Array-object references remain live across resize; retained element references have stricter validity.
use super::{AspectTrackedDevicePose, AspectXRSessionFields};
use rustycad_tkernel::{
    ncollection::Array1,
    standard::{Failure, SharedState},
};
/// Alias actual TrackedPoses storage; `reference.lower()` reads its current signed bound.
pub struct AspectXrTrackedPosesRef<'a> {
    fields: &'a SharedState<AspectXRSessionFields>,
}
impl<'a> AspectXrTrackedPosesRef<'a> {
    pub(super) fn new(fields: &'a SharedState<AspectXRSessionFields>) -> Self {
        // Native const Array1& names the array object, whose identity survives Resize.
        Self { fields }
    }
    /// Read the current signed lower bound; `reference.lower()`.
    pub fn lower(&self) -> i32 {
        let fields = self.fields.read();
        fields.tracked_poses.lower()
    }
    /// Read the current signed upper bound; `reference.upper()`.
    pub fn upper(&self) -> i32 {
        let fields = self.fields.read();
        fields.tracked_poses.upper()
    }
    /// Read current length; `reference.len()`.
    pub fn len(&self) -> usize {
        let fields = self.fields.read();
        fields.tracked_poses.len()
    }
    /// Inspect the current empty range; `reference.is_empty()`.
    pub fn is_empty(&self) -> bool {
        let fields = self.fields.read();
        fields.tracked_poses.is_empty()
    }
    /// Explicit copied element, with native bounds errors; `reference.pose_snapshot(device)?`.
    pub fn pose_snapshot(&self, device: i32) -> Result<AspectTrackedDevicePose, Failure> {
        let fields = self.fields.read();
        fields
            .tracked_poses
            .value(device)
            .copied()
            .map_err(|error| error.into_failure("Aspect_XRSession::TrackedPoses.Value"))
    }
    /// Explicit owning array copy, preserving bounds; `let copy=reference.snapshot()`.
    pub fn snapshot(&self) -> Array1<'static, AspectTrackedDevicePose> {
        let fields = self.fields.read();
        fields.tracked_poses.clone()
    }
    /// Identify the actual array object; `reference.object_address()`.
    pub fn object_address(&self) -> *const Array1<'static, AspectTrackedDevicePose> {
        let fields = self.fields.read();
        &fields.tracked_poses as *const Array1<'static, AspectTrackedDevicePose>
    }
    /// Compare native array-reference identity; `left.is_same_reference(&right)`.
    pub fn is_same_reference(&self, other: &AspectXrTrackedPosesRef<'_>) -> bool {
        let left = self.object_address();
        let right = other.object_address();
        left == right
    }
}
