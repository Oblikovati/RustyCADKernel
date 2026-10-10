//! Selection creation and native foreground/background prebuilding decisions.
use super::{selection_manager::*, *};
use crate::{select3d::Select3DSensitiveGroup, std_select::StdSelectBrepSelectionTool};
use rustycad_tkernel::standard::{Failure, Handle};
impl SelectMgrSelectionManager {
    pub(super) fn load_mode(&self, object: &ManagedSelectable, mode: i32) -> Result<(), Failure> {
        if mode == -1 {
            return Ok(());
        }
        let old = managed_object(object)?.selection(mode);
        if let Some(old) = old.get() {
            return self.reload_removed_mode(object, old, mode);
        }
        let selection = Handle::new(SelectMgrSelection::new(mode));
        managed_object(object)?.add_selection(&selection, mode)?;
        if self.contains(object) {
            self.required_selector()?
                .add_selection_to_object(object, &selection)?;
            selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::None);
        }
        self.build_selection_bvh(&selection)
    }
    fn reload_removed_mode(
        &self,
        object: &ManagedSelectable,
        old: &SelectMgrSelection,
        mode: i32,
    ) -> Result<(), Failure> {
        if !old.is_empty() || old.bvh_update_status() != SelectMgrTypeOfBvhUpdate::Remove {
            return Ok(());
        }
        let selection = Handle::new(SelectMgrSelection::new(mode));
        managed_object(object)?.add_selection(&selection, mode)?;
        selection.update_bvh_status(SelectMgrTypeOfBvhUpdate::Remove);
        selection.set_selection_state(SelectMgrStateOfSelection::Deactivated);
        self.build_selection_bvh(&selection)
    }
    fn build_selection_bvh(&self, selection: &SelectMgrSelection) -> Result<(), Failure> {
        let selector = self.required_selector()?;
        if !selector.to_prebuild_bvh() {
            return StdSelectBrepSelectionTool::pre_build_bvh(selection);
        }
        for sensitive in selection.entities() {
            let entity = sensitive.base_sensitive();
            selector.queue_bvh_build(&entity)?;
            if let Some(group) = entity.cast::<Select3DSensitiveGroup>().get() {
                for child in group.entities() {
                    selector.queue_bvh_build(&child)?;
                }
            }
        }
        Ok(())
    }
}
