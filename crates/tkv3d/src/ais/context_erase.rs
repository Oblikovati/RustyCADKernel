//! Erase hides retained presentations and deactivates recorded modes without unloading objects.
use super::*;
use crate::{
    prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers},
    select_mgr::*,
};
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Hide an object, retaining its presentation and context membership; `context.erase(&object, true)?`.
    pub fn erase(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        if !object.is_auto_hilight() {
            object.clear_selected();
        }
        self.erase_global(object, false)?;
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// Apply the protected native erase transition without the public ClearSelected callback.
    /// Example: `context.erase_global(&object, false)?`.
    pub fn erase_global(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        let status = self.object_status(object);
        if object.is_null()
            || status.is_null()
            || object.display_status() == PrsMgrDisplayStatus::Erased
        {
            return Ok(());
        }
        self.hide_erased_presentations(object, &status)?;
        self.deactivate_erased_modes(object, &status)?;
        status.clear_selection_modes();
        object.set_display_status(PrsMgrDisplayStatus::Erased);
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn hide_erased_presentations(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        status: &AisGlobalStatus,
    ) -> Result<(), Failure> {
        let highlight = if object.has_hilight_mode() {
            object.hilight_mode()
        } else {
            0
        };
        self.unselect_owners(object)?;
        self.detection_presentations()
            .set_visibility(&object.cast(), status.display_mode(), false);
        self.clear_erased_dynamic_highlight(object);
        object.erase_presentations(false);
        if self.is_selected_object(object) && status.display_mode() != highlight {
            self.detection_presentations()
                .set_visibility(&object.cast(), highlight, false);
        }
        Ok(())
    }
    fn clear_erased_dynamic_highlight(&self, object: &Handle<dyn AisInteractiveObject>) {
        let owner = self.fields.read().last_picked.clone();
        if !owner.is_null() && owner.is_same_selectable(&object.cast()) {
            self.clear_dynamic_highlight();
        }
    }
    fn deactivate_erased_modes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        status: &AisGlobalStatus,
    ) -> Result<(), Failure> {
        let mut cursor = status.selection_modes().cursor();
        loop {
            let mode = cursor.value(&status.selection_modes()).copied();
            let Some(mode) = mode else {
                break;
            };
            let manager = self.selection_manager().clone();
            manager.deactivate(&object.cast(), mode)?;
            cursor.next(&status.selection_modes());
        }
        Ok(())
    }
    /// Toggle matching selected owners in their captured order; `context.unselect_owners(&object)?`.
    pub fn unselect_owners(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        let mut walk = self.selected_walk();
        let mut matching = Vec::new();
        while let Some(owner) = walk.current() {
            if owner.is_same_selectable(&object.cast()) {
                matching.push(owner);
            }
            walk.next();
        }
        for owner in matching {
            self.add_or_remove_selected_owner(&owner, false)?;
        }
        Ok(())
    }
}
