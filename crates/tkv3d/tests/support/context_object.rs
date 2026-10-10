//! Named interactive object records callbacks and reads back context state during them.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::{
    Graphic3dCamera, Graphic3dStructureMembers, Graphic3dStructureSource,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*, select3d::Select3DSensitivePoint};
use std::sync::atomic::{AtomicI32, Ordering};
pub struct ContextObject {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    pub events: SharedState<Vec<i32>>,
    pub fail_mode: AtomicI32,
}
rustycad_tkernel::impl_transient!(ContextObject,"ContextObject",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl ContextObject {
    pub fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            events: SharedState::default(),
            fail_mode: AtomicI32::new(-999),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    fn observe_context(&self, code: i32) {
        self.events.write().push(code);
        if let Some(context) = self.get_context().get() {
            let _ = context.main_prs_mgr().object_address();
        }
    }
}
impl PrsMgrPresentableObject for ContextObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.observe_context(100 + mode);
        if self.fail_mode.load(Ordering::Acquire) == mode {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!("presentation mode {mode}; expected a non-failing fixture mode"),
            ));
        }
        presentation.new_group()?;
        Ok(())
    }
    fn recompute_transformation(&self, _: &Handle<Graphic3dCamera>) {
        self.observe_context(300);
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for ContextObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.observe_context(200 + mode);
        if self.fail_mode.load(Ordering::Acquire) == mode + 10 {
            return Err(Failure::with_message(
                "Standard_Failure",
                &format!("selection mode {mode}; expected a non-failing fixture mode"),
            ));
        }
        let owner =
            SelectMgrEntityOwner::with_selectable(&self.presentable_base().this().cast(), mode);
        selection.add(
            &Handle::new(Select3DSensitivePoint::new(
                &owner.cast(),
                Point3::default(),
            ))
            .cast(),
        )?;
        Ok(())
    }
}
impl AisInteractiveObject for ContextObject {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}
