//! Detection iteration is independent from the owner currently highlighted.
use super::{AisContextHandle, AisInteractiveContext, AisInteractiveObject};
use crate::select_mgr::*;
use rustycad_tkernel::standard::{Failure, Handle};
impl AisInteractiveContext {
    /// Whether a dynamically detected owner exists; `context.has_detected()`.
    pub fn has_detected(&self) -> bool {
        !self.fields.read().last_picked.is_null()
    }
    /// Borrow the highlighted owner, independently of iteration; `context.detected_owner()`.
    pub fn detected_owner(&self) -> AisContextHandle<'_, dyn SelectMgrEntityOwnerSource> {
        self.borrow_handle(0, |fields, _| &fields.last_picked)
    }
    /// Return the highlighted interactive object; `context.detected_interactive()`.
    pub fn detected_interactive(&self) -> Handle<dyn AisInteractiveObject> {
        let owner = self.fields.read().last_picked.clone();
        owner
            .get()
            .map_or_else(Handle::default, |owner| owner.selectable().cast())
    }
    /// Check the native cycling bound, including its initial zero cursor; `context.has_next_detected()`.
    pub fn has_next_detected(&self) -> bool {
        let fields = self.fields.read();
        !fields.detected.is_empty() && fields.current_highlighted <= fields.detected.upper() as i32
    }
    /// Start iteration when nonempty, preserving the cursor otherwise; `context.init_detected()`.
    pub fn init_detected(&self) {
        let mut fields = self.fields.write();
        if !fields.detected.is_empty() {
            fields.current_detected = 1;
        }
    }
    /// Check the independent iteration cursor; `context.more_detected()`.
    pub fn more_detected(&self) -> bool {
        let fields = self.fields.read();
        fields.current_detected >= 1 && fields.current_detected <= fields.detected.upper() as i32
    }
    /// Advance even beyond the last detected entry; `context.next_detected()`.
    pub fn next_detected(&self) {
        self.fields.write().current_detected += 1;
    }
    /// Resolve the current rank through the live selector; `context.detected_current_owner()?`.
    pub fn detected_current_owner(
        &self,
    ) -> Result<Handle<dyn SelectMgrEntityOwnerSource>, Failure> {
        if !self.more_detected() {
            return Ok(Handle::default());
        }
        let rank = {
            let fields = self.fields.read();
            *fields.detected.value(fields.current_detected as usize)?
        };
        let selector = self.detection_selector();
        selector.picked(rank)
    }
    /// Return the interactive object at the iteration cursor; `context.detected_current_object()?`.
    pub fn detected_current_object(&self) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
        let owner = self.detected_current_owner()?;
        Ok(owner
            .get()
            .map_or_else(Handle::default, |owner| owner.selectable().cast()))
    }
    /// Clear detection and immediate highlighting in native order; `context.clear_detected(false)`.
    pub fn clear_detected(&self, redraw: bool) -> bool {
        self.reset_detected_cursors();
        let update = self.last_picked_has_selectable();
        if update {
            self.clear_dynamic_highlight();
        }
        self.fields.write().last_picked = Handle::default();
        self.detection_selector().clear_picked();
        if update && redraw {
            self.detection_viewer().redraw_immediate();
        }
        update
    }
    pub(super) fn reset_detected_cursors(&self) {
        let mut fields = self.fields.write();
        fields.current_detected = 0;
        fields.current_highlighted = 0;
        fields.detected.clear(None);
    }
    pub(super) fn last_picked_has_selectable(&self) -> bool {
        let owner = self.fields.read().last_picked.clone();
        owner.get().is_some_and(|owner| owner.has_selectable())
    }
}

impl AisInteractiveContext {
    pub(super) fn detection_selector(&self) -> Handle<dyn SelectMgrViewerSelectorSource> {
        self.main_selector().clone()
    }
    pub(super) fn detection_presentations(
        &self,
    ) -> Handle<crate::prs_mgr::PrsMgrPresentationManager> {
        self.fields.read().presentation_manager.clone()
    }
    pub(super) fn detection_viewer(&self) -> Handle<crate::v3d::V3dViewer> {
        self.fields.read().viewer.clone()
    }
}
