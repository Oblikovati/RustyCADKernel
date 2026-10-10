//! Permanent context removal follows native detach, presentation, detection and selection ordering.
use super::*;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::{prs_mgr::PrsMgrDisplayStatus, select_mgr::*};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle, exceptions::program_error},
};
impl AisInteractiveContext {
    /// Detach and permanently unregister an object; `context.remove(&object, true)?`.
    pub fn remove(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        self.detach_removed_context(object)?;
        self.clear_global(object, update)
    }
    fn detach_removed_context(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if !object.has_interactive_context() {
            return Ok(());
        }
        let this = self.this.read().upgrade();
        if object.interactive_context().object_address() != this.object_address() {
            return Err(program_error(&format!(
                "object context {:?}; expected {:?}",
                object.interactive_context().object_address(),
                this.object_address()
            )));
        }
        object.set_context(&Handle::default());
        Ok(())
    }
    /// Clear detection, snapshot registrations, remove each and update once; `context.remove_all(true)?`.
    pub fn remove_all(&self, update: bool) -> Result<(), Failure> {
        self.clear_detected(false);
        let mut objects = List::default();
        self.objects_inside(&mut objects, AisKindOfInteractive::None, -1)?;
        for object in &objects {
            self.remove(object, false)?;
        }
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    /// Apply native protected removal without detaching the object's context; `context.clear_global(&object, false)?`.
    pub fn clear_global(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        let status = self.object_status(object);
        if object.is_null() || status.is_null() {
            let manager = self.selection_manager().clone();
            manager.remove(&object.cast())?;
            return Ok(());
        }
        self.clear_registered_object(object)?;
        if update && object.display_status() == PrsMgrDisplayStatus::Displayed {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn clear_registered_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        self.unselect_owners(object)?;
        self.detection_presentations().erase(&object.cast(), -1);
        object.erase_presentations(true);
        self.remove_detected_object(object)?;
        let manager = self.selection_manager().clone();
        manager.remove(&object.cast())?;
        self.set_object_status(object, PrsMgrDisplayStatus::None, -1, -1)?;
        object.view_affinity().set_visible(true);
        self.detection_viewer()
            .structure_manager()
            .unregister_object(&object.cast());
        self.clear_removed_last_pick(object);
        Ok(())
    }
    fn clear_removed_last_pick(&self, object: &Handle<dyn AisInteractiveObject>) {
        let picked = self.fields.read().last_picked.clone();
        if !picked.is_null() && picked.is_same_selectable(&object.cast()) {
            self.clear_dynamic_highlight();
            self.fields.write().last_picked = Handle::default();
        }
    }
    fn remove_detected_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        let mut index = 1;
        while index <= self.fields.read().detected.upper() {
            let rank = *self.fields.read().detected.value(index)?;
            let owner = self.detection_selector().picked(rank)?;
            let interactive = owner.get().map_or_else(Handle::default, |o| {
                o.selectable().cast::<dyn AisInteractiveObject>()
            });
            if !interactive.is_null() && &interactive == object {
                self.remove_detected_rank(index)?;
            } else {
                index += 1;
            }
        }
        Ok(())
    }
    fn remove_detected_rank(&self, index: usize) -> Result<(), Failure> {
        let mut fields = self.fields.write();
        fields.detected.remove(index)?;
        if fields.current_detected == index as i32 {
            fields.current_detected = (fields.detected.upper() as i32).min(index as i32);
        }
        if fields.current_highlighted == index as i32 {
            fields.current_highlighted = 0;
        }
        Ok(())
    }
}
