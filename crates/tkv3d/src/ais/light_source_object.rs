//! Concrete native light construction with retained aspects and virtual drawing hooks.
use super::*;
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkservice::graphic3d::*;
/// A native interactive light presentation; `AisLightSource::new(&light)?`.
pub struct AisLightSource {
    transient: StandardTransient,
    base: AisLightSourceBase,
}
rustycad_tkernel::impl_transient!(AisLightSource,"AIS_LightSource",dyn AisLightSourceSource,transient,
    interfaces:[dyn AisLightSourceSource,dyn AisInteractiveObject,dyn SelectMgrSelectableObject,dyn PrsMgrPresentableObject]);
impl AisLightSource {
    /// Retain the live light and create its native drawers/aspects; `AisLightSource::new(&light)?`.
    pub fn new(light: &Handle<Graphic3dCLight>) -> Result<Handle<Self>, Failure> {
        let object = Handle::new(Self {
            transient: StandardTransient::default(),
            base: AisLightSourceBase::new(light),
        });
        super::light_source_aspects::ais_light_source_initialize(&*object)?;
        prs_mgr_presentable_bind(&object.cast());
        Ok(object)
    }
}
impl AisLightSourceSource for AisLightSource {
    fn light_source_base(&self) -> &AisLightSourceBase {
        &self.base
    }
    fn update_light_aspects(&self) -> Result<(), Failure> {
        super::light_source_aspects::refresh_light_aspects(self)
    }
}
impl AisInteractiveObject for AisLightSource {
    fn interactive_base(&self) -> &AisInteractiveObjectBase {
        &self.base.interactive
    }
    fn kind_of_interactive(&self) -> AisKindOfInteractive {
        AisKindOfInteractive::LightSource
    }
    fn process_dragging(
        &self,
        context: &Handle<AisInteractiveContext>,
        view: &Handle<crate::v3d::V3dView>,
        owner: &Handle<dyn SelectMgrEntityOwnerSource>,
        from: Vec2<i32>,
        to: Vec2<i32>,
        action: AisDragAction,
    ) -> bool {
        native_light_bool_result(ais_light_source_process_dragging(
            self, context, view, owner, from, to, action,
        ))
    }
}
impl PrsMgrPresentableObject for AisLightSource {
    fn presentable_base(&self) -> &PrsMgrPresentableObjectBase {
        &self.base.interactive.presentable
    }
    fn accept_display_mode(&self, mode: i32) -> bool {
        mode == 0 || mode == 1
    }
    fn compute(
        &self,
        _: &Handle<PrsMgrPresentationManager>,
        prs: &Handle<dyn Graphic3dStructureSource>,
        mode: i32,
    ) -> Result<(), Failure> {
        ais_light_source_compute(self, prs, mode)
    }
    fn set_local_transformation_datum(
        &self,
        datum: &Handle<rustycad_tkmath::toploc::Datum3>,
    ) -> Result<(), Failure> {
        ais_light_source_set_local_transform(self, datum)
    }
    crate::ais_presentable_overrides!();
}
impl SelectMgrSelectableObject for AisLightSource {
    fn selectable_base(&self) -> &SelectMgrSelectableObjectBase {
        &self.base.interactive.selectable
    }
    fn compute_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::light_source_selection::light_selection(self, selection, mode)
    }
}

// Native bool APIs unwind Standard_Failure; retaining its type avoids erasing native errors.
pub(super) fn native_light_bool_result(result: Result<bool, Failure>) -> bool {
    rustycad_tkernel::standard::raise_native_failure(result)
}
