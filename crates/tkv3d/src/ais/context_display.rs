//! AIS display transitions preserve callback and partial-failure ordering from OCCT 3d097a0.
use super::*;
use crate::prs_mgr::PrsMgrDisplayStatus;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Display using object/context defaults and the automatic selection policy.
    /// Example: `context.display(&object, true)?`.
    pub fn display(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update: bool,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        let (mut display, mut highlight, mut selection) = (0, -1, -1);
        self.get_def_modes(object, &mut display, &mut highlight, &mut selection);
        if !self.get_auto_activate_selection() {
            selection = -1;
        }
        self.display_with_modes(
            object,
            display,
            selection,
            update,
            PrsMgrDisplayStatus::None,
        )
    }
    /// Display explicit modes, or load an erased object without activating selection.
    /// Example: `context.display_with_modes(&object, 0, -1, true, PrsMgrDisplayStatus::Displayed)?`.
    pub fn display_with_modes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display: i32,
        selection: i32,
        update: bool,
        state: PrsMgrDisplayStatus,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        if state == PrsMgrDisplayStatus::Erased {
            return self.display_as_erased(object, display, selection, update);
        }
        self.display_registered_or_new(object, display, selection)?;
        if update {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn display_as_erased(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display: i32,
        selection: i32,
        update: bool,
    ) -> Result<(), Failure> {
        self.erase(object, update)?;
        self.load(object, selection)?;
        if let Some(status) = self.object_status(object).get() {
            status.set_display_mode(display);
        }
        Ok(())
    }
    fn display_registered_or_new(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display: i32,
        selection: i32,
    ) -> Result<(), Failure> {
        self.set_context_to_object(object)?;
        let status = self.object_status(object);
        let Some(status) = status.get() else {
            return self.display_new_object(object, display, selection);
        };
        self.switch_display_presentation(object, status, display)?;
        self.activate_display_selection(object, selection, Some(status))
    }
    fn display_new_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display: i32,
        selection: i32,
    ) -> Result<(), Failure> {
        self.set_object_status(object, PrsMgrDisplayStatus::Displayed, display, selection)?;
        self.register_context_affinity(object);
        self.detection_presentations()
            .display(&object.cast(), display)?;
        self.activate_display_selection(object, selection, None)
    }
    fn switch_display_presentation(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        status: &AisGlobalStatus,
        display: i32,
    ) -> Result<(), Failure> {
        let previous = status.display_mode();
        if previous != display {
            self.hide_previous_display_mode(object, previous)?;
        }
        status.set_display_mode(display);
        object.set_display_status(PrsMgrDisplayStatus::Displayed);
        self.detection_presentations()
            .display(&object.cast(), display)?;
        if status.is_hilighted() {
            self.highlight_global_style_slot(object, &|| status.hilight_style().clone(), display)?;
        }
        Ok(())
    }
    fn hide_previous_display_mode(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        if self
            .detection_presentations()
            .is_highlighted(&object.cast(), mode)
        {
            self.unhighlight_global(object)?;
        }
        self.detection_presentations()
            .set_visibility(&object.cast(), mode, false);
        Ok(())
    }
}
