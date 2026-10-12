//! Genuine owner override mutates actual context fields after receiving live viewer/manager references.
use super::*;
use crate::{
    prs_mgr::PrsMgrPresentationManager, prs3d::Prs3dDrawer, select_mgr::*, v3d::V3dViewer,
};
use rustycad_tkernel::standard::{
    Failure, Handle, HandleReference, SharedState, StandardTransient, WeakHandle,
    raise_native_failure,
};
use std::sync::atomic::{AtomicBool, Ordering};
pub(super) struct ViewerManagerCallbackOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    context: WeakHandle<AisInteractiveContext>,
    replacement: WeakHandle<V3dViewer>,
    replacement_manager: WeakHandle<PrsMgrPresentationManager>,
    mode: i32,
    armed: AtomicBool,
    pub records: SharedState<[i32; 8]>,
}
rustycad_tkernel::impl_transient!(ViewerManagerCallbackOwner,"ViewerManagerCallbackOwner",dyn SelectMgrEntityOwnerSource,transient,
    interfaces:[dyn SelectMgrEntityOwnerSource]);
impl ViewerManagerCallbackOwner {
    pub(super) fn new(
        context: &Handle<AisInteractiveContext>,
        replacement: &Handle<V3dViewer>,
        manager: &Handle<PrsMgrPresentationManager>,
        mode: i32,
    ) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(&Handle::default(), 0),
            context: context.downgrade(),
            replacement: replacement.downgrade(),
            replacement_manager: manager.downgrade(),
            mode,
            armed: AtomicBool::new(true),
            records: SharedState::new([0; 8]),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
    fn replace_actual_context_fields(&self) {
        let context = self.context.observe();
        let mut fields = context.fields.write();
        fields.viewer = self.replacement.observe().clone();
        fields.presentation_manager = self.replacement_manager.observe().clone();
        drop(fields);
        context.set_default_drawer(&Handle::new(Prs3dDrawer::default()));
    }
    fn record_before(
        &self,
        viewer: &dyn HandleReference<V3dViewer>,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
    ) {
        let context = self.context.observe();
        let records = [
            i32::from(
                viewer.reference_address() == context.current_viewer_ref().reference_address(),
            ),
            i32::from(
                manager.reference_address() == context.main_prs_mgr_ref().reference_address(),
            ),
            viewer.observe().ref_count(),
            manager.observe().ref_count(),
        ];
        self.records.write()[..4].copy_from_slice(&records);
    }
    fn record_after(
        &self,
        viewer: &dyn HandleReference<V3dViewer>,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
    ) {
        let records = [
            i32::from(viewer.object_address() == self.replacement.object_address()),
            i32::from(manager.object_address() == self.replacement_manager.object_address()),
            viewer.observe().ref_count(),
            manager.observe().ref_count(),
        ];
        self.records.write()[4..].copy_from_slice(&records);
    }
    fn raise_viewer_manager_callback_failure(&self) {
        match self.mode {
            1 => raise_native_failure(Err(Failure::new(
                "viewer manager callback fault=1; expected success",
            ))),
            2 => std::panic::panic_any(73_u32),
            _ => {}
        }
    }
}
impl SelectMgrEntityOwnerSource for ViewerManagerCallbackOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn update_highlight_trsf(
        &self,
        viewer: &dyn HandleReference<V3dViewer>,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        _mode: i32,
    ) -> Result<(), Failure> {
        if !self.armed.swap(false, Ordering::Relaxed) {
            return Ok(());
        }
        self.record_before(viewer, manager);
        self.replace_actual_context_fields();
        self.record_after(viewer, manager);
        self.raise_viewer_manager_callback_failure();
        Ok(())
    }
}
