//! Native sensitive light origin/sphere and light-switch owner.
use super::*;
use crate::{prs_mgr::*, select_mgr::*, select3d::*};
use rustycad_tkernel::{
    ncollection::Vec2,
    standard::{Failure, Handle, StandardTransient},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::{
    aspect::{AspectVKeyFlags, AspectVKeyMouse},
    graphic3d::*,
};
pub(super) fn light_selection<O: AisLightSourceSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
    mode: i32,
) -> Result<(), Failure> {
    if mode != 0 {
        return Ok(());
    }
    let owner = AisLightSourceOwner::new(&object.presentable_base().this().cast(), 15);
    if object.light().light_type() == Graphic3dTypeOfLightSource::Directional {
        light_sensitive_sphere(object, selection, &owner.cast())?;
    }
    let point = Handle::new(Select3DSensitivePoint::new(
        &owner.cast(),
        Point3::default(),
    ));
    point.set_sensitivity_factor(light_point_sensitivity(object))?;
    selection.add(&point.cast())
}
fn light_point_sensitivity<O: AisLightSourceSource + ?Sized>(object: &O) -> i32 {
    let persistence = object.presentable_base().read().transform_persistence;
    if persistence
        .get()
        .is_some_and(|pers| pers.is_trihedron_or_2d())
    {
        12.max((object.size() * 0.5) as i32)
    } else {
        12
    }
}

fn light_sensitive_sphere<O: AisLightSourceSource + ?Sized>(
    object: &O,
    selection: &Handle<SelectMgrSelection>,
    owner: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> Result<(), Failure> {
    let sphere = Handle::new(Select3DSensitiveSphere::new(
        owner,
        Point3::default(),
        object.size() * 0.5,
    ));
    object.light_source_base().visual.write().sensitive_sphere = sphere.clone();
    selection.add(&sphere.cast())
}
/// Native light selection owner; `AisLightSourceOwner::new(&light.cast(),5)`.
pub struct AisLightSourceOwner {
    transient: StandardTransient,
    base: SelectMgrEntityOwnerBase,
}
rustycad_tkernel::impl_transient!(AisLightSourceOwner,"AIS_LightSourceOwner",dyn SelectMgrEntityOwnerSource,transient,
    interfaces:[dyn SelectMgrEntityOwnerSource]);
impl AisLightSourceOwner {
    /// Observe a selectable light without retaining it; `AisLightSourceOwner::new(&light.cast(),15)`.
    pub fn new(object: &Handle<dyn SelectMgrSelectableObject>, priority: i32) -> Handle<Self> {
        let owner = Handle::new(Self {
            transient: StandardTransient::default(),
            base: SelectMgrEntityOwnerBase::new(object, priority),
        });
        select_mgr_entity_owner_bind(&owner.cast());
        owner
    }
    /// Native click routing with typed aspect errors; `owner.handle_light_click(button,modifiers)?`.
    pub fn handle_light_click(
        &self,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
    ) -> Result<bool, Failure> {
        let light = self.selectable().cast::<dyn AisLightSourceSource>();
        let Some(object) = light.get() else {
            return Ok(false);
        };
        if !object.to_switch_on_click()
            || button != rustycad_tkservice::aspect::ASPECT_VKEY_MOUSE_LEFT_BUTTON
            || modifiers != 0
        {
            return Ok(false);
        }
        object.light().set_enabled(!object.light().is_enabled());
        object.update_light_aspects()?;
        Ok(true)
    }
}
impl SelectMgrEntityOwnerSource for AisLightSourceOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        &self.base
    }
    fn handle_mouse_click(
        &self,
        _: Vec2<i32>,
        button: AspectVKeyMouse,
        modifiers: AspectVKeyFlags,
        _: bool,
    ) -> bool {
        super::light_source_object::native_light_bool_result(
            self.handle_light_click(button, modifiers),
        )
    }
    fn is_forced_hilight(&self) -> bool {
        self.selectable()
            .cast::<dyn AisLightSourceSource>()
            .get()
            .is_some_and(|light| {
                light.light().light_type() == Graphic3dTypeOfLightSource::Directional
            })
    }
    fn hilight_with_color(
        &self,
        manager: &Handle<PrsMgrPresentationManager>,
        style: &Handle<crate::prs3d::Prs3dDrawer>,
        mode: i32,
    ) -> Result<(), Failure> {
        super::light_source_highlight::light_owner_highlight(self, manager, style, mode)
    }
}
