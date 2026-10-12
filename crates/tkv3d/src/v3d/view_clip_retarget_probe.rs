//! Native-style virtual clipping callbacks use actual owned slots and the real protected myView field.
use super::V3dViewSource;
use super::view_camera_alias_fixture::BorrowedCameraView;
use rustycad_tkernel::standard::{
    Handle, HandleFieldRef, HandleReference, IntrusiveReferenceSlot, SharedState, WeakHandle,
    raise_native_failure,
};
use rustycad_tkservice::graphic3d::{
    Graphic3dCView, Graphic3dClipPlaneSequenceSource, Graphic3dClipSequenceFieldRef,
    Graphic3dSequenceOfHClipPlane,
};
use std::sync::Arc;
#[derive(Default)]
pub(super) struct ViewClipRetargetProbe {
    pub rows: SharedState<Vec<String>>,
    pub owner: SharedState<WeakHandle<dyn V3dViewSource>>,
    pub replacement: SharedState<Handle<dyn Graphic3dCView>>,
    pub replace_once: SharedState<bool>,
}
pub(super) type OwnedClipProbeSlot =
    Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSequenceSource>>;
pub(super) fn new_owned_clip_probe_slot() -> OwnedClipProbeSlot {
    let value = Handle::new(Graphic3dSequenceOfHClipPlane::default()).into_cast();
    Arc::new(IntrusiveReferenceSlot::from_owned(value))
}
pub(super) fn clip_probe_getter(
    view: &BorrowedCameraView,
    probe: &ViewClipRetargetProbe,
) -> Box<dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>> {
    let slot = view
        .clip_probe_slot
        .as_ref()
        .expect("clipping probe slot=None; expected an actual owned native field");
    probe.rows.write().push(format!(
        "get {} {} {}",
        view.identity,
        view.transient.ref_count(),
        slot.observe().ref_count()
    ));
    replace_probe_driver_view(probe);
    Box::new(Graphic3dClipSequenceFieldRef::new(slot))
}
fn replace_probe_driver_view(probe: &ViewClipRetargetProbe) {
    let replace = std::mem::take(&mut *probe.replace_once.write());
    if !replace {
        return;
    }
    let owner = probe.owner.read().observe();
    let incoming = HandleFieldRef::new(&probe.replacement, |slot| slot);
    raise_native_failure(owner.view_base().driver_view_slot.assign(&incoming));
}
pub(super) fn clip_probe_setter(
    view: &BorrowedCameraView,
    probe: &ViewClipRetargetProbe,
    incoming: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) {
    let before = incoming.observe().ref_count();
    let slot = view
        .clip_probe_slot
        .as_ref()
        .expect("clipping probe slot=None; expected an actual owned native field");
    raise_native_failure(slot.assign(incoming));
    probe.rows.write().push(format!(
        "set {} {} {} {}",
        view.identity,
        view.transient.ref_count(),
        before,
        incoming.observe().ref_count()
    ));
}
