//! Owner toggles distinguish selected flags, container membership and filtering.
use super::*;
use crate::select_mgr::*;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle, exceptions::program_error},
};
impl AisInteractiveContext {
    /// Toggle a registered object's global owner; `context.add_or_remove_selected_object(&object,true)?`.
    pub fn add_or_remove_selected_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() || self.object_status(object).is_null() {
            return Ok(());
        }
        let owner = object.global_sel_owner();
        if owner.get().is_some_and(|owner| owner.has_selectable()) {
            self.add_or_remove_selected_owner(&owner, update)?;
        }
        Ok(())
    }
    /// Toggle an owner, allowing removal even when the filter rejects it; `context.add_or_remove_selected_owner(&owner,true)?`.
    pub fn add_or_remove_selected_owner(
        &self,
        owner: &AisSelectedOwner,
        update: bool,
    ) -> Result<(), Failure> {
        if owner.get().is_none_or(|owner| !owner.has_selectable()) {
            return Ok(());
        }
        let filter = self.fields.read().filters.clone();
        if !filter.is_ok(owner)? && !owner.is_selected() {
            return Ok(());
        }
        let object = self.toggle_owner_membership(owner)?;
        if self.automatic_hilight() {
            if self.object_status(&object).is_null() {
                return Ok(());
            }
            self.refresh_toggled_owner(owner, &object)?;
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn toggle_owner_membership(
        &self,
        owner: &AisSelectedOwner,
    ) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let scheme = if owner.is_selected() {
            AisSelectionScheme::Remove
        } else {
            AisSelectionScheme::Add
        };
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        let filter = self.fields.read().filters.cast();
        self.selected_container().select(
            owner,
            &filter,
            scheme,
            self.is_detected_object(&object)?,
        )?;
        Ok(object)
    }
    fn refresh_toggled_owner(
        &self,
        owner: &AisSelectedOwner,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if owner.is_selected() {
            return self.highlight_selected(owner);
        }
        let mut owners = List::default();
        owners.append(owner.clone());
        self.unhighlight_owners(&owners, false)?;
        self.object_status(object)
            .set_hilight_style(&Handle::default());
        Ok(())
    }
    /// Set membership, retaining the native automatic/nonautomatic paths; `context.set_selected_state(&owner,true)?`.
    pub fn set_selected_state(
        &self,
        owner: &AisSelectedOwner,
        selected: bool,
    ) -> Result<bool, Failure> {
        if owner.is_null() {
            return Err(program_error(
                "SetSelectedState owner=null; expected a selectable owner",
            ));
        }
        if !owner.has_selectable() || self.selected_container().is_selected(owner) == selected {
            return Ok(false);
        }
        if owner.is_auto_hilight() {
            self.add_or_remove_selected_owner(owner, false)?;
            return Ok(true);
        }
        self.set_nonautomatic_selected_state(owner, selected)
    }
    fn set_nonautomatic_selected_state(
        &self,
        owner: &AisSelectedOwner,
        selected: bool,
    ) -> Result<bool, Failure> {
        let status = if selected {
            self.selected_container().add_select(owner)?
        } else {
            self.remove_selected_state(owner)?
        };
        owner.set_selected(selected);
        Ok(status
            == if selected {
                AisSelectStatus::Added
            } else {
                AisSelectStatus::Removed
            })
    }
    fn remove_selected_state(&self, owner: &AisSelectedOwner) -> Result<AisSelectStatus, Failure> {
        let object: Handle<dyn AisInteractiveObject> = owner.selectable().cast();
        let filter = self.fields.read().filters.cast();
        self.selected_container().select(
            owner,
            &filter,
            AisSelectionScheme::Remove,
            self.is_detected_object(&object)?,
        )
    }
}
