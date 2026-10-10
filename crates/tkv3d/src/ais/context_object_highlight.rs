//! Public highlight state and out-parameters retain native distinctions.
use super::*;
use crate::{
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    prs3d::Prs3dDrawer,
    select_mgr::*,
};
use rustycad_tkernel::standard::{Failure, Handle, exceptions::program_error};
impl AisInteractiveContext {
    /// Assign this context recursively, rejecting a foreign context; `context.set_context_to_object(&object)?`.
    pub fn set_context_to_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        let this = self.this.read().upgrade();
        if object.has_interactive_context()
            && object.interactive_context().object_address() != this.object_address()
        {
            return Err(program_error(&format!(
                "object context {:?}; expected {:?}",
                object.interactive_context().object_address(),
                this.object_address()
            )));
        }
        if !object.has_interactive_context() {
            object.set_context(&this);
        }
        self.assign_context_children(object)
    }
    fn assign_context_children(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        let mut index = 0;
        while let Some(child) = context_child_at(object, index) {
            let interactive = child.cast::<dyn AisInteractiveObject>();
            if !interactive.is_null() {
                self.set_context_to_object(&interactive)?;
            }
            index += 1;
        }
        Ok(())
    }
    /// Set registered global highlighting, drawing only displayed objects; `context.hilight_with_color(&object,&style,true)?`.
    pub fn hilight_with_color(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &Handle<Prs3dDrawer>,
        update: bool,
    ) -> Result<(), Failure> {
        self.hilight_object_style_slot(object, &|| style.clone(), update)
    }
    pub(super) fn hilight_object_style_slot(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &impl Fn() -> Handle<Prs3dDrawer>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        self.set_context_to_object(object)?;
        if self.object_status(object).is_null() {
            return Ok(());
        }
        self.object_status(object).set_hilight_status(true);
        if object.display_status() == PrsMgrDisplayStatus::Displayed {
            self.highlight_global_style_slot(
                object,
                style,
                self.object_status(object).display_mode(),
            )?;
            self.object_status(object).set_hilight_style(&style());
        }
        if update {
            self.detection_viewer().update();
        }
        Ok(())
    }
    /// Clear registered global highlighting, drawing only displayed objects; `context.unhilight(&object,true)?`.
    pub fn unhilight(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() || self.object_status(object).is_null() {
            return Ok(());
        }
        self.object_status(object).set_hilight_status(false);
        self.object_status(object)
            .set_hilight_style(&Handle::default());
        if object.display_status() == PrsMgrDisplayStatus::Displayed {
            self.unhighlight_global(object)?;
        }
        if update {
            self.detection_viewer().update();
        }
        Ok(())
    }
    /// Read a registered object's global highlight flag; `context.is_hilighted_object(&object)`.
    pub fn is_hilighted_object(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        self.object_status(object)
            .get()
            .is_some_and(|status| status.is_hilighted())
    }
    /// Read global status for a global owner, otherwise ask its presentation; `context.is_hilighted_owner(&owner)`.
    pub fn is_hilighted_owner(&self, owner: &AisSelectedOwner) -> bool {
        if owner.get().is_none_or(|owner| !owner.has_selectable()) {
            return false;
        }
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        if object.global_sel_owner() == *owner {
            return self.is_hilighted_object(&object);
        }
        let style = self.get_sel_style(&object, owner);
        let mode = self.highlight_mode_from_slot(&object, &|| style.read(), -1);
        owner.is_hilighted(&self.detection_presentations(), mode)
    }
    /// Return the highlighted global style, clearing the output otherwise; `context.object_highlight_style(&object,&mut style)`.
    pub fn object_highlight_style(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        style: &mut Handle<Prs3dDrawer>,
    ) -> bool {
        let status = self.object_status(object);
        if status.get().is_some_and(|status| status.is_hilighted()) {
            *style = status.hilight_style().clone();
            return true;
        }
        *style = Handle::default();
        false
    }
    /// Read an owner's highlight style; invalid owners leave the output untouched.
    /// Example: `context.owner_highlight_style(&owner,&mut style)`.
    pub fn owner_highlight_style(
        &self,
        owner: &AisSelectedOwner,
        style: &mut Handle<Prs3dDrawer>,
    ) -> bool {
        if owner.get().is_none_or(|owner| !owner.has_selectable()) {
            return false;
        }
        if !self.is_hilighted_owner(owner) {
            *style = Handle::default();
            return false;
        }
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        *style = if object.global_sel_owner() == *owner {
            self.object_status(&object).hilight_style().clone()
        } else {
            self.get_sel_style(&object, owner).read()
        };
        true
    }
}
fn context_child_at(
    object: &Handle<dyn AisInteractiveObject>,
    index: usize,
) -> Option<Handle<dyn crate::prs_mgr::PrsMgrPresentableObject>> {
    object
        .presentable_base()
        .fields
        .read()
        .children
        .get(index)
        .cloned()
}
