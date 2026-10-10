//! Child-first load/removal and activation preserve native mutation boundaries.
use super::{selection_manager::*, viewer_selector_activation::*, *};
use crate::prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::standard::{Failure, Handle};
impl SelectMgrSelectionManager {
    /// Load children, then this object's optional mode; repeated Load returns early; `manager.load(&object, 0)?`.
    pub fn load(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        if self.contains(object) {
            return Ok(());
        }
        for child in managed_children(object)? {
            self.load(&child, mode)?;
        }
        if !managed_object(object)?.has_own_presentations() {
            return Ok(());
        }
        self.global.write().add(object.clone());
        let selector = self.required_selector()?;
        if !selector.contains(object) && managed_object(object)?.has_own_presentations() {
            selector.add_selectable_object(object)?;
        }
        self.load_mode(object, mode)
    }
    /// Remove children, selector membership and computed entities; `manager.remove(&object)?`.
    pub fn remove(&self, object: &ManagedSelectable) -> Result<(), Failure> {
        for child in managed_children(object)? {
            self.remove(&child)?;
        }
        if !managed_object(object)?.has_own_presentations() {
            return Ok(());
        }
        if self.contains(object) {
            self.remove_from_selector(object)?;
            self.global.write().remove(object);
        }
        managed_object(object)?.clear_selections(false);
        Ok(())
    }
    fn remove_from_selector(&self, object: &ManagedSelectable) -> Result<(), Failure> {
        let selector = self.required_selector()?;
        if !selector.contains(object) {
            return Ok(());
        }
        for selection in managed_object(object)?.selections() {
            selector.remove_selection_of_object(object, &selection)?;
            selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Remove);
            deactivate_selector_mode(selector, &selection)?;
        }
        selector.remove_selectable_object(object)
    }
    /// Activate non-erased children then this mode if globally loaded; `manager.activate(&object, 0)?`.
    pub fn activate(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        if mode == -1 {
            return Ok(());
        }
        for child in managed_children(object)? {
            if managed_object(&child)?.display_status() != PrsMgrDisplayStatus::Erased {
                self.activate(&child, mode)?;
            }
        }
        if !managed_object(object)?.has_own_presentations() {
            return Ok(());
        }
        self.activate_own_mode(object, mode)
    }
    fn activate_own_mode(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        let old = managed_object(object)?.selection(mode);
        if old.get().is_none_or(|selection| selection.is_empty()) {
            self.load_mode(object, mode)?;
        }
        let selection = managed_object(object)?.selection(mode);
        self.update_activation_mode(object, &selection, mode)?;
        self.restore_activation_bvh(object, &selection)?;
        if self.contains(object) {
            activate_selector_mode(
                self.required_selector()?,
                &managed_object(object)?.selection(mode),
            );
        }
        Ok(())
    }
    fn update_activation_mode(
        &self,
        object: &ManagedSelectable,
        selection: &Handle<SelectMgrSelection>,
        mode: i32,
    ) -> Result<(), Failure> {
        let status = selection.update_status();
        if status == SelectMgrTypeOfUpdate::Full {
            if managed_object(object)?.has_selection(mode) {
                self.required_selector()?
                    .remove_selection_of_object(object, selection)?;
            }
            managed_object(object)?.recompute_primitives_of_mode(mode)?;
        }
        if status != SelectMgrTypeOfUpdate::None {
            managed_object(object)?.update_transformations(selection);
            self.required_selector()?.rebuild_objects_tree(false)?;
        }
        selection.set_update_status(SelectMgrTypeOfUpdate::None);
        Ok(())
    }
    fn restore_activation_bvh(
        &self,
        object: &ManagedSelectable,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<(), Failure> {
        let add = match selection.bvh_update_status() {
            SelectMgrTypeOfBvhUpdate::Add | SelectMgrTypeOfBvhUpdate::Renew => true,
            SelectMgrTypeOfBvhUpdate::Remove => {
                selection.get_selection_state() == SelectMgrStateOfSelection::Deactivated
            }
            _ => false,
        };
        if add {
            self.required_selector()?
                .add_selection_to_object(object, selection)?;
        }
        selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
        Ok(())
    }
    /// Deactivate children and matching modes, with -1 meaning all; `manager.deactivate(&object, -1)?`.
    pub fn deactivate(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        for child in managed_children(object)? {
            self.deactivate(&child, mode)?;
        }
        if !self.owns_presentations(object)? {
            return Ok(());
        }
        for selection in managed_modes(object, mode)? {
            deactivate_selector_mode(self.required_selector()?, &selection)?;
        }
        Ok(())
    }
    /// Find an active matching mode in children or this loaded object; `manager.is_activated(&object, -1)?`.
    pub fn is_activated(&self, object: &ManagedSelectable, mode: i32) -> Result<bool, Failure> {
        for child in managed_children(object)? {
            if self.is_activated(&child, mode)? {
                return Ok(true);
            }
        }
        if !self.owns_presentations(object)? {
            return Ok(false);
        }
        for selection in managed_modes(object, mode)? {
            if self.required_selector()?.selection_status(&selection)?
                == SelectMgrStateOfSelection::Activated
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
