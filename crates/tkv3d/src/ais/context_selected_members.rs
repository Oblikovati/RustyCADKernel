//! Context queries retain the selected container's own public cursor.
use super::{
    context_owner_walk::{AisOwnerList, AisOwnerWalk},
    *,
};
use crate::select_mgr::*;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    pub(super) fn selected_container(&self) -> Handle<dyn AisSelectionSource> {
        self.fields.read().selection.clone()
    }
    pub(super) fn selected_walk(&self) -> AisOwnerWalk<'static> {
        AisOwnerWalk::new(AisOwnerList::Selected(self.selected_container()))
    }
    pub(super) fn object_status(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Handle<AisGlobalStatus> {
        self.fields
            .read()
            .objects
            .seek(object)
            .cloned()
            .unwrap_or_default()
    }
    /// Count selected entries independently of their flags; `context.nb_selected()`.
    pub fn nb_selected(&self) -> i32 {
        self.selected_container().extent()
    }
    /// Reset the selection container's public cursor; `context.init_selected()`.
    pub fn init_selected(&self) {
        self.selected_container().init();
    }
    /// Test the selection container's public cursor; `context.more_selected()`.
    pub fn more_selected(&self) -> bool {
        self.selected_container().more()
    }
    /// Advance the selection container's public cursor; `context.next_selected()`.
    pub fn next_selected(&self) {
        self.selected_container().next();
    }
    /// Read the current owner or null at end; `context.selected_owner()?`.
    pub fn selected_owner(&self) -> Result<AisSelectedOwner, Failure> {
        if !self.selected_container().more() {
            return Ok(Handle::default());
        }
        Ok(self.selected_container().value()?.clone())
    }
    /// Downcast the current selected object's handle; `context.selected_interactive()?`.
    pub fn selected_interactive(&self) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let owner = self.selected_owner()?;
        Ok(owner
            .get()
            .map_or_else(Handle::default, |owner| owner.selectable().cast()))
    }
    /// Read the first selected object without moving the cursor; `context.first_selected_object()`.
    pub fn first_selected_object(&self) -> Handle<dyn AisInteractiveObject> {
        self.selected_walk()
            .current()
            .map_or_else(Handle::default, |owner| owner.selectable().cast())
    }
    /// Read the owner flag rather than membership; `context.is_selected_owner(&owner)`.
    pub fn is_selected_owner(&self, owner: &AisSelectedOwner) -> bool {
        owner.get().is_some_and(|owner| owner.is_selected())
    }
    /// Test the registered object's activated global selection mode; `context.is_selected_object(&object)`.
    pub fn is_selected_object(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        if object.is_null() {
            return false;
        }
        let status = self.object_status(object);
        if status.is_null() || !status.is_s_mode_in(object.global_selection_mode()) {
            return false;
        }
        self.is_selected_owner(&object.global_sel_owner())
    }
    /// Find an object among live detected ranks; `context.is_detected_object(&object)?`.
    pub fn is_detected_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<bool, Failure> {
        let mut index = 1;
        while index <= self.fields.read().detected.upper() {
            let rank = *self.fields.read().detected.value(index)?;
            let owner = self.detection_selector().picked(rank)?;
            let picked: Handle<dyn AisInteractiveObject> = owner
                .get()
                .map_or_else(Handle::default, |owner| owner.selectable().cast());
            if !picked.is_null() && picked == *object {
                return Ok(true);
            }
            index += 1;
        }
        Ok(false)
    }
    pub(super) fn selected_status(&self) -> AisStatusOfPick {
        match self.nb_selected() {
            0 => AisStatusOfPick::NothingSelected,
            1 => AisStatusOfPick::OneSelected,
            _ => AisStatusOfPick::SeveralSelected,
        }
    }
}
