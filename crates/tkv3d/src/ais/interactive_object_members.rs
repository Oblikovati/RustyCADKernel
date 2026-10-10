//! Native nonvirtual object members use its real interactive context and presentation manager.
use super::{AisInteractiveContext, AisInteractiveObject, AisInteractiveObjectFields};
use crate::{
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    prs3d::*,
    select_mgr::select_mgr_selectable_dump_json,
};
use rustycad_tkernel::standard::{
    Failure, Handle, StandardTransient, WeakHandle, dump::DumpStream,
};
use rustycad_tkservice::{
    graphic3d::{
        Graphic3dAspectsSource, Graphic3dPresentationAttributesMembers, Graphic3dStructureMembers,
        Graphic3dStructureSource,
    },
    json_dump::{base_class, class_begin, field_pointer},
};
use std::{ops::Deref, sync::RwLockReadGuard};
/// Borrow the actual application-owner handle; `object.get_owner().is_null()`.
pub struct AisApplicationOwner<'a>(RwLockReadGuard<'a, AisInteractiveObjectFields>);
impl Deref for AisApplicationOwner<'_> {
    type Target = Handle<StandardTransient>;
    fn deref(&self) -> &Self::Target {
        &self.0.owner
    }
}
/// Nonvirtual interactive-object operations; `object.redisplay(false)?`.
pub trait AisInteractiveObjectMembers: AisInteractiveObject {
    /// Test the retained pointer identity, including an expired weak pointer; `object.has_interactive_context()`.
    fn has_interactive_context(&self) -> bool {
        !self.interactive_base().fields.read().context.is_null()
    }
    /// Copy a non-owning context observer; `object.interactive_context().object_address()`.
    fn interactive_context(&self) -> WeakHandle<AisInteractiveContext> {
        self.interactive_base().fields.read().context.clone()
    }
    /// Retain the current live context; `object.get_context()`.
    fn get_context(&self) -> Handle<AisInteractiveContext> {
        self.interactive_base().fields.read().context.upgrade()
    }
    /// Test application-owner presence; `object.has_owner()`.
    fn has_owner(&self) -> bool {
        !self.interactive_base().fields.read().owner.is_null()
    }
    /// Borrow the retained application-owner handle; `object.get_owner()`.
    fn get_owner(&self) -> AisApplicationOwner<'_> {
        AisApplicationOwner(self.interactive_base().fields.read())
    }
    /// Retain any transient application owner; `object.set_owner(&owner)`.
    fn set_owner(&self, owner: &Handle<StandardTransient>) {
        self.interactive_base().fields.write().owner = owner.clone();
    }
    /// Clear the application owner; `object.clear_owner()`.
    fn clear_owner(&self) {
        self.interactive_base().fields.write().owner = Handle::default();
    }
    /// Protected display status assignment; `object.set_display_status(PrsMgrDisplayStatus::Erased)`.
    fn set_display_status(&self, status: PrsMgrDisplayStatus) {
        self.presentable_base().fields.write().display_status = status;
    }
    /// Redisplay through the current context, without viewer update; `object.redisplay(true)?`.
    fn redisplay(&self, all_modes: bool) -> Result<(), Failure> {
        let context = self.get_context();
        let Some(context) = context.get() else {
            return Ok(());
        };
        context.redisplay(&self.presentable_base().this().cast(), false, all_modes)
    }
    /// Check this context's presentation for the drawer's display mode; `object.has_presentation()`.
    fn has_presentation(&self) -> bool {
        let context = self.get_context();
        let Some(context) = context.get() else {
            return false;
        };
        let manager = context.main_prs_mgr().clone();
        manager.has_presentation(
            &self.presentable_base().this(),
            self.attributes().display_mode(),
        )
    }
    /// Retrieve the real presentation without creating one; `object.presentation()?`.
    fn presentation(&self) -> Result<Handle<dyn Graphic3dStructureSource>, Failure> {
        let context = self.get_context();
        let Some(context) = context.get() else {
            return Ok(Handle::default());
        };
        let manager = context.main_prs_mgr().clone();
        Ok(manager
            .presentation(
                &self.presentable_base().this(),
                self.attributes().display_mode(),
                false,
                &Handle::default(),
            )?
            .cast())
    }
    /// Apply a supported aspect to the last existing group; `object.set_aspect(&line.cast())?`.
    fn set_aspect(&self, aspect: &Handle<dyn Prs3dBasicAspect>) -> Result<(), Failure> {
        if !self.has_presentation() {
            return Ok(());
        }
        let presentation = self.presentation()?;
        let Some(group) = presentation.groups().last().cloned() else {
            return Ok(());
        };
        if let Some(aspect) = interactive_graphic_aspect(aspect) {
            group.set_group_primitives_aspect(&aspect);
        }
        Ok(())
    }
}
impl<T: AisInteractiveObject + ?Sized> AisInteractiveObjectMembers for T {}
fn interactive_graphic_aspect(
    aspect: &Handle<dyn Prs3dBasicAspect>,
) -> Option<Handle<dyn Graphic3dAspectsSource>> {
    if let Some(shading) = aspect.cast::<Prs3dShadingAspect>().get() {
        return Some(shading.aspect().cast());
    }
    if let Some(line) = aspect.cast::<Prs3dLineAspect>().get() {
        return Some(line.aspect().cast());
    }
    if let Some(point) = aspect.cast::<Prs3dPointAspect>().get() {
        return Some(point.aspect().cast());
    }
    aspect
        .cast::<Prs3dTextAspect>()
        .get()
        .map(|text| text.aspect().cast())
}
/// Native AIS DumpJson override; `ais_interactive_object_dump_json(&object,&mut stream,-1)?`.
pub fn ais_interactive_object_dump_json<O: AisInteractiveObject + ?Sized>(
    object: &O,
    stream: &mut DumpStream,
    depth: i32,
) -> std::io::Result<()> {
    class_begin(stream, "AIS_InteractiveObject")?;
    base_class(stream, depth, &|stream, depth| {
        select_mgr_selectable_dump_json(object, stream, depth)
    })?;
    let fields = object.interactive_base().fields.read();
    field_pointer(stream, "myCTXPtr", fields.context.object_address() as usize)?;
    field_pointer(stream, "myOwner", fields.owner.object_address() as usize)
}
/// Preserve inherited selectable overrides and the AIS dump override in a concrete object.
/// Example: `ais_presentable_overrides!();` inside its `PrsMgrPresentableObject` implementation.
#[macro_export]
macro_rules! ais_presentable_overrides {
    () => {
        $crate::select_mgr_presentable_overrides!(
            dump = $crate::ais::ais_interactive_object_dump_json
        );
    };
}
