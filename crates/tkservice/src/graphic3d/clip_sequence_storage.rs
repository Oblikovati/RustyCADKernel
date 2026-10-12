//! Actual native clipping nodes, independent of borrowed field and cursor storage leases.
#![allow(unsafe_code)]
use super::{Graphic3dClipPlaneHandleField, Graphic3dClipPlaneSource};
use rustycad_tkernel::ncollection::{Sequence, SequenceClearProgress, SequenceNodeRetirement};
use rustycad_tkernel::standard::{Failure, IntrusiveReferenceSlot, SharedState};
use std::sync::Arc;
/// Protected native item sequence; `sequence.clip_plane_items.read().length()`.
pub type ClipPlaneNativeItems = Arc<SharedState<Sequence<Graphic3dClipPlaneHandleField>>>;
pub(super) type ClipPlaneNodeLease = Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSource>>;

pub(super) fn retire_clip_node(
    items: &ClipPlaneNativeItems,
    node: SequenceNodeRetirement<Graphic3dClipPlaneHandleField>,
) {
    // SAFETY: this sequence's native current resource must accept its existing node allocations.
    // Resource replacement uses the same compatibility contract as NCollection_Sequence::Clear.
    unsafe { node.retire_using(|| items.read().allocator().storage_lease()) };
}
pub(super) fn clear_native_clip_items(items: &ClipPlaneNativeItems) -> Result<(), Failure> {
    let mut progress = items.write().begin_clear_progress();
    while let Some(field) = visit_clip_retirement(items, &mut progress)? {
        // First remains registered during the plane destructor, and Next was saved beforehand.
        field.clear()?;
        let node = items.write().detach_clear_node(&mut progress)?;
        retire_clip_node(items, node);
    }
    items.write().finish_clear_progress(progress)
}
fn visit_clip_retirement(
    items: &ClipPlaneNativeItems,
    progress: &mut SequenceClearProgress,
) -> Result<Option<ClipPlaneNodeLease>, Failure> {
    // Only field bytes escape this guard: the plane's native owner remains the actual payload.
    Ok(items
        .read()
        .visit_clear_node(progress)?
        .map(|field| field.storage_lease()))
}
