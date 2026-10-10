//! First display activates unconditionally; redisplay records only newly activated modes.
use super::{AisGlobalStatus, AisInteractiveContext, AisInteractiveObject};
use crate::select_mgr::SelectMgrSelectionManager;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    pub(super) fn activate_display_selection(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        status: Option<&AisGlobalStatus>,
    ) -> Result<(), Failure> {
        if mode == -1 {
            return Ok(());
        }
        self.load_display_selection(object)?;
        if let Some(status) = status {
            if self
                .display_selection_manager()
                .is_activated(&object.cast(), mode)?
            {
                return Ok(());
            }
            status.add_selection_mode(mode);
        }
        self.display_selection_manager()
            .activate(&object.cast(), mode)
    }
    fn load_display_selection(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if !self.display_selection_manager().contains(&object.cast()) {
            self.display_selection_manager().load(&object.cast(), -1)?;
        }
        Ok(())
    }
    fn display_selection_manager(&self) -> Handle<SelectMgrSelectionManager> {
        // Release context guards before callbacks; native protected manager replacement remains observable.
        self.fields.read().selection_manager.clone()
    }
}
