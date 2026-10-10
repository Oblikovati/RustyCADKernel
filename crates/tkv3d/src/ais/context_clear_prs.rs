//! Presentation-only erasure leaves registered selection modes active, as in OCCT 3d097a0.
use super::*;
use crate::prs_mgr::PrsMgrDisplayStatus;
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Remove one presentation without deactivating selection; `context.clear_prs(&object, 0, true)?`.
    pub fn clear_prs(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        update: bool,
    ) -> Result<(), Failure> {
        self.clear_global_prs(object, mode, update)
    }
    /// Apply the native protected presentation erase; `context.clear_global_prs(&object, 0, false)?`.
    pub fn clear_global_prs(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        update: bool,
    ) -> Result<(), Failure> {
        let status = self.fields.read().objects.seek(object).map(Handle::observe);
        let Some(status) = status else {
            return Ok(());
        };
        if status.display_mode() == mode {
            self.erase_registered_presentation(object, mode)?;
        }
        if object.display_status() == PrsMgrDisplayStatus::Displayed && update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn erase_registered_presentation(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        let highlight = if object.has_hilight_mode() {
            object.hilight_mode()
        } else {
            0
        };
        if highlight == mode
            && self
                .detection_presentations()
                .is_highlighted(&object.cast(), mode)
        {
            self.unhighlight_global(object)?;
        }
        self.detection_presentations().erase(&object.cast(), mode);
        Ok(())
    }
    /// Accept the ignored legacy decomposition flag; `context.display_legacy(&object, 0, -1, true, false, state)?`.
    pub fn display_legacy(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        selection: i32,
        update: bool,
        _allow_decomposition: bool,
        status: PrsMgrDisplayStatus,
    ) -> Result<(), Failure> {
        self.display_with_modes(object, mode, selection, update, status)
    }
}
