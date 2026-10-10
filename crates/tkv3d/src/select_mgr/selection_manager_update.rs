//! Native structure restoration and recomputation, including inactive modes.
use super::{selection_manager::*, *};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::{Failure, Handle, exceptions::program_error};
impl SelectMgrSelectionManager {
    /// Remove matching sensitives and mark them for addition; `manager.clear_selection_structures(&object, -1)?`.
    pub fn clear_selection_structures(
        &self,
        object: &ManagedSelectable,
        mode: i32,
    ) -> Result<(), Failure> {
        self.change_selection_structures(object, mode, false)
    }
    /// Re-add matching sensitives, then dirty object bounds; `manager.restore_selection_structures(&object, 0)?`.
    pub fn restore_selection_structures(
        &self,
        object: &ManagedSelectable,
        mode: i32,
    ) -> Result<(), Failure> {
        self.change_selection_structures(object, mode, true)
    }
    fn change_selection_structures(
        &self,
        object: &ManagedSelectable,
        mode: i32,
        restore: bool,
    ) -> Result<(), Failure> {
        for child in managed_children(object)? {
            self.change_selection_structures(&child, mode, restore)?;
        }
        if !self.owns_presentations(object)? {
            return Ok(());
        }
        let selector = self.required_selector()?;
        for selection in managed_modes(object, mode)? {
            if restore {
                selector.add_selection_to_object(object, &selection)?;
                selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
            } else {
                selector.remove_selection_of_object(object, &selection)?;
                selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Add);
            }
        }
        selector.rebuild_objects_tree(false)
    }
    /// Protected native per-mode recomputation; `manager.recompute_selection_mode(&object, &selection, 0)?`.
    pub fn recompute_selection_mode(
        &self,
        object: &ManagedSelectable,
        selection: &SelectMgrSelection,
        mode: i32,
    ) -> Result<(), Failure> {
        selection.set_update_status(SelectMgrTypeOfUpdate::Full);
        self.clear_selection_structures(object, mode)?;
        managed_object(object)?.recompute_primitives_of_mode(mode)?;
        self.restore_selection_structures(object, mode)?;
        selection.set_update_status(SelectMgrTypeOfUpdate::None);
        selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
        Ok(())
    }
    /// Recompute stored modes; native non-forced mode also visits inactive selections.
    /// Example: `manager.recompute_selection(&object, false, -1)?`.
    pub fn recompute_selection(
        &self,
        object: &ManagedSelectable,
        force: bool,
        mode: i32,
    ) -> Result<(), Failure> {
        if force {
            return self.force_recompute(object, mode);
        }
        for child in managed_children(object)? {
            self.recompute_selection(&child, false, mode)?;
        }
        if !self.owns_presentations(object)? {
            return Ok(());
        }
        for selection in managed_modes(object, mode)? {
            self.recompute_selection_mode(object, &selection, selection.mode())?;
        }
        Ok(())
    }
    fn force_recompute(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        let selectable = managed_object(object)?;
        if mode != -1 && !selectable.has_selection(mode) {
            return Ok(());
        }
        self.clear_selection_structures(object, mode)?;
        if mode == -1 {
            selectable.recompute_primitives()?;
        } else {
            selectable.recompute_primitives_of_mode(mode)?;
        }
        selectable.update_transformation()?;
        self.restore_selection_structures(object, mode)
    }
    /// Apply pending updates to active modes, or all when forced; `manager.update(&object, true)?`.
    pub fn update(&self, object: &ManagedSelectable, force: bool) -> Result<(), Failure> {
        for child in managed_children(object)? {
            self.update(&child, force)?;
        }
        if !managed_object(object)?.has_own_presentations() {
            return Ok(());
        }
        for selection in managed_object(object)?.selections() {
            if force
                || self.required_selector()?.selection_status(&selection)?
                    == SelectMgrStateOfSelection::Activated
            {
                self.apply_mode_update(object, &selection)?;
            }
        }
        Ok(())
    }
    fn apply_mode_update(
        &self,
        object: &ManagedSelectable,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<(), Failure> {
        let status = selection.update_status();
        if status == SelectMgrTypeOfUpdate::Full {
            self.clear_selection_structures(object, selection.mode())?;
            managed_object(object)?.recompute_primitives_of_mode(selection.mode())?;
            self.restore_selection_structures(object, selection.mode())?;
        }
        if status != SelectMgrTypeOfUpdate::None {
            managed_object(object)?.update_transformations(selection);
            self.required_selector()?.rebuild_objects_tree(false)?;
        }
        selection.set_update_status(SelectMgrTypeOfUpdate::None);
        selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
        Ok(())
    }
    /// Mark every local selection without descending into children; `manager.set_update_mode(&object, SelectMgrTypeOfUpdate::Full)?`.
    pub fn set_update_mode(
        &self,
        object: &ManagedSelectable,
        status: SelectMgrTypeOfUpdate,
    ) -> Result<(), Failure> {
        for selection in managed_object(object)?.selections() {
            selection.set_update_status(status);
        }
        Ok(())
    }
    /// Mark one existing local mode, with no special treatment of -1; `manager.set_mode_update(&object, 0, status)?`.
    pub fn set_mode_update(
        &self,
        object: &ManagedSelectable,
        mode: i32,
        status: SelectMgrTypeOfUpdate,
    ) -> Result<(), Failure> {
        if let Some(selection) = managed_object(object)?.selection(mode).get() {
            selection.set_update_status(status);
        }
        Ok(())
    }
    /// Change a mode's sensitivity and its retained tolerance count; `manager.set_selection_sensitivity(&object, 0, 5)?`.
    pub fn set_selection_sensitivity(
        &self,
        object: &ManagedSelectable,
        mode: i32,
        sensitivity: i32,
    ) -> Result<(), Failure> {
        if sensitivity < 0 {
            return Err(program_error(&format!(
                "selection sensitivity {sensitivity}; expected a nonnegative value"
            )));
        }
        let Some(object_value) = object.get() else {
            return Ok(());
        };
        let selection = object_value.selection(mode);
        let Some(selection) = selection.get() else {
            return Ok(());
        };
        let previous = selection.sensitivity();
        selection.set_sensitivity(sensitivity)?;
        if self.contains(object) && self.required_selector()?.contains(object) {
            let mut tolerances = self
                .required_selector()?
                .viewer_selector_base()
                .tolerances
                .write();
            tolerances.decrement(previous)?;
            tolerances.add(sensitivity);
        }
        Ok(())
    }
    /// Reclassify persistence only when present in both registries; `manager.update_selection(&object)?`.
    pub fn update_selection(&self, object: &ManagedSelectable) -> Result<(), Failure> {
        if self.contains(object) && self.required_selector()?.contains(object) {
            self.required_selector()?.move_selectable_object(object)?;
        }
        Ok(())
    }
}
