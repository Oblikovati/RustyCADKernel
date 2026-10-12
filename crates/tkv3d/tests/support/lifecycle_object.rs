//! Named callback fixture allows context mutation without holding a Rust collection guard.
use super::context_object::ContextObject;
use rustycad_tkernel::standard::HandleReference;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::{ais::*, prs_mgr::*, select_mgr::*};
type LifecycleCallback = Box<dyn Fn(&LifecycleObject) + Send + Sync>;
pub struct LifecycleObject {
    transient: StandardTransient,
    inner: Handle<ContextObject>,
    pub on_kind: SharedState<Option<LifecycleCallback>>,
    pub on_compute: SharedState<Option<LifecycleCallback>>,
    pub on_clear: SharedState<Option<LifecycleCallback>>,
}
rustycad_tkernel::impl_transient!(LifecycleObject,"Martensite_LifecycleObject",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl LifecycleObject {
    pub fn new() -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            inner: ContextObject::new(),
            on_kind: SharedState::default(),
            on_compute: SharedState::default(),
            on_clear: SharedState::default(),
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
    fn callback(&self, slot: &SharedState<Option<LifecycleCallback>>) {
        let action = slot.write().take();
        if let Some(action) = action {
            action(self);
        }
    }
}
impl PrsMgrPresentableObject for LifecycleObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        self.inner.presentable_base()
    }
    fn compute(
        &self,
        manager: &dyn HandleReference<PrsMgrPresentationManager>,
        presentation: &dyn HandleReference<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.callback(&self.on_compute);
        self.inner.compute(manager, presentation, mode)
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for LifecycleObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        self.inner.selectable_base()
    }
    fn compute_selection(
        &self,
        selection: &dyn HandleReference<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.inner.compute_selection(selection, mode)
    }
    fn clear_selected(&self) {
        self.callback(&self.on_clear);
        self.inner.clear_selected();
    }
}
impl AisInteractiveObject for LifecycleObject {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        self.inner.interactive_base()
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        self.callback(&self.on_kind);
        AisKindOfInteractive::Shape
    }
    fn signature(&self) -> i32 {
        7
    }
}
