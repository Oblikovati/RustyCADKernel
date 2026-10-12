//! A real empty user presentation observes camera callbacks without changing scene bounds.
use super::{AisInteractiveObject, AisInteractiveObjectBase};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{
    Failure, Handle, SharedState, StandardTransient, raise_native_failure,
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dStructureMembers, Graphic3dStructureSource,
};
use std::sync::atomic::{AtomicI32, Ordering};
pub(super) struct ActionCallbackObject {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    pub(super) calls: AtomicI32,
    pub(super) fail_kind: AtomicI32,
    pub(super) last_center: SharedState<Point3>,
}
rustycad_tkernel::impl_transient!(ActionCallbackObject,"ActionCallbackObject",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl ActionCallbackObject {
    pub(super) fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            calls: AtomicI32::new(0),
            fail_kind: AtomicI32::new(0),
            last_center: SharedState::default(),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl PrsMgrPresentableObject for ActionCallbackObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        // Context registration and actual presentation storage belong to this object.
        &self.base.presentable
    }
    fn compute(
        &self,
        _: &dyn HandleReference<PrsMgrPresentationManager>,
        presentation: &dyn HandleReference<dyn Graphic3dStructureSource>,
        _: i32,
    ) -> Result<(), Failure> {
        presentation.observe().new_group()?;
        Ok(())
    }
    fn recompute_transformation(&self, camera: &Handle<Graphic3dCamera>) {
        self.calls.fetch_add(1, Ordering::Relaxed);
        *self.last_center.write() = camera.center();
        match self.fail_kind.load(Ordering::Relaxed) {
            1 => raise_native_failure(Err(Failure::new(
                "camera callback value=1; expected successful transformation",
            ))),
            2 => std::panic::panic_any(73_u32),
            _ => (),
        }
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for ActionCallbackObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        // Empty selection keeps this legitimate user presentation out of picking.
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        _: &dyn HandleReference<SelectMgrSelection>,
        _: i32,
    ) -> Result<(), Failure> {
        // The native observer has no sensitive entities either.
        Ok(())
    }
}
impl AisInteractiveObject for ActionCallbackObject {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        // Preserve actual context ownership, display and callbacks.
        &self.base
    }
}
