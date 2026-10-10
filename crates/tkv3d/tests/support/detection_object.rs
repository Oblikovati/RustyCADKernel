//! Named interactive fixture observes callbacks and replaces the native drawer slots.
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkservice::graphic3d::{
    Graphic3dPresentationAttributesMembers, Graphic3dPresentationAttributesSource,
    Graphic3dStructureMembers, Graphic3dStructureSource,
};
use rustycad_tkv3d::{ais::*, prs_mgr::*, prs3d::Prs3dDrawer, select_mgr::*};
use std::sync::Arc;
#[derive(Default)]
pub struct DetectionTrace {
    pub events: Vec<i32>,
    pub accept: bool,
    pub mutation: i32,
    pub forced: bool,
    pub failing: i32,
    pub mask: i32,
}
pub type DetectionEvents = Arc<SharedState<DetectionTrace>>;
pub struct DetectionObject {
    transient: StandardTransient,
    base: AisInteractiveObjectBase,
    pub trace: DetectionEvents,
}
rustycad_tkernel::impl_transient!(DetectionObject,"DetectionObject",dyn AisInteractiveObject,transient,interfaces:[dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl DetectionObject {
    pub fn new(trace: DetectionEvents) -> Handle<Self> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisInteractiveObjectBase::new(PrsMgrTypeOfPresentation3d::AllView),
            trace,
        });
        prs_mgr_presentable_bind(&object.cast());
        object
    }
}
impl PrsMgrPresentableObject for DetectionObject {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.presentable
    }
    fn bounding_box(&self, bounds: &mut rustycad_tkmath::bnd::BoundingBox3) {
        bounds.update_bounds([-1., -1., -3.], [1., 1., 1.]);
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        let (accept, mutation) = {
            let mut trace = self.trace.write();
            trace.events.push(100 + mode);
            (trace.accept, trace.mutation)
        };
        let context = self.get_context();
        let _ = context.has_detected();
        if mutation != 0 {
            let style = Handle::new(Prs3dDrawer::new());
            style.set_display_mode(9);
            if mutation == 1 {
                self.set_dynamic_hilight_attributes(&style);
            }
            if mutation == 2 {
                context.set_dynamic_highlight_style(&style);
            }
        }
        accept
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        presentation: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.trace.write().events.push(400 + mode);
        presentation.new_group()?;
        Ok(())
    }
    rustycad_tkv3d::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for DetectionObject {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.selectable
    }
    fn compute_selection(&self, _: &Handle<SelectMgrSelection>, _: i32) -> Result<(), Failure> {
        Ok(())
    }
    fn clear_dynamic_highlight(&self, manager: &Handle<PrsMgrPresentationManager>) {
        self.trace.write().events.push(300);
        let _ = self.get_context().has_detected();
        manager.clear_immediate_draw();
    }
}
impl AisInteractiveObject for DetectionObject {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base
    }
}
pub struct DetectionOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
    pub trace: DetectionEvents,
}
rustycad_tkernel::impl_transient!(DetectionOwner,"DetectionOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl DetectionOwner {
    pub fn new_handle(
        object: &Handle<dyn SelectMgrSelectableObject>,
        id: i32,
        trace: DetectionEvents,
    ) -> Handle<dyn SelectMgrEntityOwnerSource> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(object, id),
            trace,
        })
        .cast();
        select_mgr_entity_owner_bind(&owner);
        owner
    }
}
impl SelectMgrEntityOwnerSource for DetectionOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn is_forced_hilight(&self) -> bool {
        self.trace.read().forced
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        self.trace.write().events.extend([
            200 + self.priority(),
            mode,
            style.display_mode(),
            style.z_layer(),
            i32::from(manager.is_immediate_mode_on()),
        ]);
        if self.trace.read().failing == self.priority() + 1 {
            return Err(Failure::with_message(
                "Standard_Failure",
                "failing owner; expected successful highlight",
            ));
        }
        select_mgr_owner_hilight_with_color(self, manager, style, mode)
    }
}
pub struct DetectionFilter {
    transient: StandardTransient,
    trace: DetectionEvents,
}
rustycad_tkernel::impl_transient!(DetectionFilter,"DetectionFilter",dyn SelectMgrFilter,transient,interfaces:[dyn SelectMgrFilter]);
impl DetectionFilter {
    pub fn new_handle(trace: DetectionEvents) -> Handle<dyn SelectMgrFilter> {
        Handle::new(Self {
            transient: StandardTransient::default(),
            trace,
        })
        .cast()
    }
}
impl SelectMgrFilter for DetectionFilter {
    fn is_ok(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) -> Result<bool, Failure> {
        let mut trace = self.trace.write();
        trace.events.push(500 + owner.priority());
        if trace.failing == owner.priority() + 4 {
            return Err(Failure::with_message(
                "Standard_Failure",
                "failing filter; expected accepted or rejected owner",
            ));
        }
        Ok(trace.mask & (1 << owner.priority()) != 0)
    }
}
