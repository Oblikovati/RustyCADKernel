//! Native transform references observe valid in-place pose edits across overridable callbacks.
use super::{AspectXRSession, AspectXRSessionFields};
use rustycad_tkernel::standard::{Failure, SharedState};
use rustycad_tkmath::gp::Transform3;
/// A native const transform alias, with explicit snapshots; `pose.snapshot()?` reads current fields.
/// Same-length rebasing preserves element aliases; storage reallocation creates native dangling-reference UB.
pub enum AspectXrTransformRef<'a> {
    /// A caller-owned ordinary transform; its Rust shared borrow keeps it valid.
    Borrowed(&'a Transform3),
    /// Actual retained head transform field.
    Head(&'a SharedState<AspectXRSessionFields>),
    /// Actual orientation field of a retained tracked pose.
    Tracked(&'a SharedState<AspectXRSessionFields>, usize, usize),
}
impl<'a> AspectXrTransformRef<'a> {
    /// Wrap the actual caller transform; `AspectXrTransformRef::borrowed(&pose)`.
    pub fn borrowed(pose: &'a Transform3) -> Self {
        // No handle or transform copy substitutes for the referenced argument.
        Self::Borrowed(pose)
    }
    /// Borrow an actual tracked orientation; `AspectXrTransformRef::tracked(&session,device)?`.
    pub fn tracked(session: &'a dyn AspectXRSession, device: i32) -> Result<Self, Failure> {
        let fields = &session.xr_session_base().fields;
        let actual = fields.read();
        let index = tracked_reference_device(&actual, device)?;
        let offset = (i64::from(index) - i64::from(actual.tracked_poses.lower())) as usize;
        let address = std::ptr::from_ref(&actual.tracked_poses[index].orientation) as usize;
        Ok(Self::Tracked(fields, offset, address))
    }
    /// Borrow the actual head field; `AspectXrTransformRef::head(&session)`.
    pub fn head(session: &'a dyn AspectXRSession) -> Self {
        // HeadPose is a native reference, while snapshots are explicit values.
        Self::Head(&session.xr_session_base().fields)
    }
    /// Explicitly read current transform values; `let values=reference.snapshot()?`.
    pub fn snapshot(&self) -> Result<Transform3, Failure> {
        match self {
            Self::Borrowed(value) => Ok(**value),
            Self::Head(fields) => Ok(fields.read().head_pose),
            Self::Tracked(fields, offset, address) => {
                let fields = fields.read();
                Ok(*retained_tracked_orientation(&fields, *offset, *address)?)
            }
        }
    }
    /// Observe actual referenced storage identity; `reference.object_address()?`.
    pub fn object_address(&self) -> Result<*const Transform3, Failure> {
        match self {
            Self::Borrowed(value) => Ok(*value as *const Transform3),
            Self::Head(fields) => Ok(&fields.read().head_pose as *const Transform3),
            Self::Tracked(fields, offset, address) => {
                let fields = fields.read();
                Ok(std::ptr::from_ref(retained_tracked_orientation(
                    &fields, *offset, *address,
                )?))
            }
        }
    }
    /// Compare real const-reference identity; `left.is_same_reference(&right)?`.
    pub fn is_same_reference(&self, other: &AspectXrTransformRef<'_>) -> Result<bool, Failure> {
        let left = self.object_address()?;
        let right = other.object_address()?;
        Ok(left == right)
    }
}
pub(super) fn tracked_reference_device(
    fields: &AspectXRSessionFields,
    device: i32,
) -> Result<i32, Failure> {
    fields
        .tracked_poses
        .value(device)
        .map(|_| device)
        .map_err(|error| error.into_failure("Aspect_XRSession::TrackedPose"))
}

fn retained_tracked_orientation(
    fields: &AspectXRSessionFields,
    offset: usize,
    address: usize,
) -> Result<&Transform3, Failure> {
    let pose = fields
        .tracked_poses
        .at(offset)
        .map_err(|error| error.into_failure("TrackedPose retained offset"))?;
    let current = std::ptr::from_ref(&pose.orientation) as usize;
    if current == address {
        return Ok(&pose.orientation);
    }
    Err(Failure::new(&format!(
        "TrackedPose offset={offset} address={current:#x}; expected retained orientation address={address:#x} without reallocation"
    )))
}
