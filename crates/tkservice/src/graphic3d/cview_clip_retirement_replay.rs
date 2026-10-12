//! Actual retained field retirement preserves pointer aliases and captures the caller before callbacks.
use super::*;
use rustycad_tkernel::standard::{
    Handle, HandleReference, IntrusiveReferenceSlot, SharedState, Transient, WeakHandle,
};
use std::sync::Arc;
struct CViewClipRetiringPlayback {
    base: Graphic3dSequenceOfHClipPlane,
    view: WeakHandle<Graphic3dRetainedView>,
    caller: Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSequenceSource>>,
    replacement: WeakHandle<dyn Graphic3dClipPlaneSequenceSource>,
    alias_address: usize,
    rows: Arc<SharedState<Vec<String>>>,
}
rustycad_tkernel::impl_transient!(CViewClipRetiringPlayback,
    "Martensite_CViewClipRetiringPlayback", Graphic3dSequenceOfHClipPlane, base,
    interfaces:[dyn Graphic3dClipPlaneSequenceSource]);
impl Graphic3dClipPlaneSequenceSource for CViewClipRetiringPlayback {
    fn clip_sequence_base(&self) -> &Graphic3dSequenceOfHClipPlane {
        &self.base
    }
}
impl Drop for CViewClipRetiringPlayback {
    fn drop(&mut self) {
        let view = self.view.observe();
        let fresh = view.clip_planes_ref();
        self.rows.write().push(format!(
            "retire {} {} {} {}",
            i32::from(fresh.object_address() == std::ptr::from_ref(self).cast::<()>()),
            self.transient_base().ref_count(),
            i32::from(fresh.reference_address() as usize == self.alias_address),
            self.caller.observe().ref_count()
        ));
        // This changes the real caller field after the outer assignment captured its raw target.
        self.caller.assign(&*self.replacement.observe()).unwrap();
    }
}
struct CViewClipRetirementActors {
    view: Handle<Graphic3dRetainedView>,
    original: Handle<dyn Graphic3dClipPlaneSequenceSource>,
    replacement: Handle<dyn Graphic3dClipPlaneSequenceSource>,
    caller: Arc<IntrusiveReferenceSlot<dyn Graphic3dClipPlaneSequenceSource>>,
    rows: Arc<SharedState<Vec<String>>>,
}
fn clip_retirement_actors() -> CViewClipRetirementActors {
    let view = create_clip_retirement_view();
    let original = Handle::new(Graphic3dSequenceOfHClipPlane::default())
        .into_cast::<dyn Graphic3dClipPlaneSequenceSource>();
    let replacement = Handle::new(Graphic3dSequenceOfHClipPlane::default())
        .into_cast::<dyn Graphic3dClipPlaneSequenceSource>();
    let caller = Arc::new(IntrusiveReferenceSlot::from_owned(original.clone()));
    CViewClipRetirementActors {
        view,
        original,
        replacement,
        caller,
        rows: Arc::default(),
    }
}
fn create_clip_retirement_view() -> Handle<Graphic3dRetainedView> {
    let driver = crate::null_driver::NullGraphicDriver::new_handle();
    let manager = Handle::new(Graphic3dStructureManager::new(&driver).unwrap());
    Handle::new(Graphic3dRetainedView::new(&manager, &[]).unwrap())
}
fn install_retiring_collection(
    actors: &CViewClipRetirementActors,
    alias: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) {
    let old = Handle::new(CViewClipRetiringPlayback {
        base: Graphic3dSequenceOfHClipPlane::default(),
        view: actors.view.downgrade(),
        caller: actors.caller.clone(),
        replacement: actors.replacement.downgrade(),
        alias_address: alias.reference_address() as usize,
        rows: actors.rows.clone(),
    })
    .into_cast::<dyn Graphic3dClipPlaneSequenceSource>();
    actors.view.set_clip_planes(&old);
}
fn slot_state(
    label: &str,
    actors: &CViewClipRetirementActors,
    alias: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) -> String {
    format!(
        "{label} {} {} {} {}",
        i32::from(alias.reference_address() == actors.view.clip_planes_ref().reference_address()),
        i32::from(alias.object_address() == actors.original.object_address()),
        actors.original.ref_count(),
        actors.replacement.ref_count()
    )
}
fn record_slot_state(
    label: &str,
    actors: &CViewClipRetirementActors,
    alias: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) {
    actors.rows.write().push(slot_state(label, actors, alias));
}
fn clear_and_restore_clip_slot(
    actors: &CViewClipRetirementActors,
    alias: &dyn HandleReference<dyn Graphic3dClipPlaneSequenceSource>,
) {
    actors
        .view
        .set_clip_planes(&Handle::<dyn Graphic3dClipPlaneSequenceSource>::default());
    actors.rows.write().push(format!(
        "null {} {} {}",
        i32::from(alias.is_null()),
        actors.original.ref_count(),
        actors.replacement.ref_count()
    ));
    actors.view.set_clip_planes(&*actors.caller);
    actors.rows.write().push(format!(
        "restore {} {} {}",
        i32::from(alias.object_address() == actors.replacement.object_address()),
        actors.replacement.ref_count(),
        actors.original.ref_count()
    ));
}
#[test]
fn actual_default_clipping_assignment_replays_retirement_aliases_and_caller_retarget() {
    let actors = clip_retirement_actors();
    let first_alias = actors.view.clip_planes_ref();
    let second_alias = actors.view.clip_planes_ref();
    install_retiring_collection(&actors, first_alias.as_ref());
    actors.view.set_clip_planes(&*actors.caller);
    record_slot_state("assigned", &actors, first_alias.as_ref());
    actors.view.set_clip_planes(second_alias.as_ref());
    record_slot_state("self", &actors, first_alias.as_ref());
    clear_and_restore_clip_slot(&actors, first_alias.as_ref());
    assert_eq!(
        actors.rows.read().join("\n") + "\n",
        include_str!("../../tests/fixtures/cview-clip-slot-retirement-native.txt")
    );
}
