//! MainSelector binds the original manager's actual slot, without a context lock.
use super::AisInteractiveContext;
use crate::select_mgr::{SelectMgrSelectionManager, SelectMgrViewerSelectorSource};
use rustycad_tkernel::standard::{Handle, HandleObserver, HandleReference};
use std::ops::Deref;
/// Retain a noncounting storage lease of the original selector owner.
/// Example: `let reference=context.main_selector(); reference.observe().nb_picked()`.
pub struct AisMainSelectorRef {
    manager: HandleObserver<SelectMgrSelectionManager>,
}
impl AisMainSelectorRef {
    pub(super) fn from_context(context: &AisInteractiveContext) -> Self {
        // OCCT 3d097a0 returns mgrSelector->Selector(), not a projection through the context.
        let manager = context.fields.read().selection_manager.observe();
        Self { manager }
    }
}
impl Deref for AisMainSelectorRef {
    type Target = Handle<dyn SelectMgrViewerSelectorSource>;
    fn deref(&self) -> &Self::Target {
        // Projecting through the current context would silently retarget this native reference.
        self.manager.selector()
    }
}
impl HandleReference<dyn SelectMgrViewerSelectorSource> for AisMainSelectorRef {
    fn observe(&self) -> HandleObserver<dyn SelectMgrViewerSelectorSource> {
        // Native borrowing increments neither manager nor selector ownership.
        self.manager.selector().observe()
    }
    fn reference_address(&self) -> *const () {
        // Equal targets can occupy distinct handle slots, so target identity is insufficient.
        std::ptr::from_ref(self.manager.selector()).cast()
    }
}

#[cfg(test)]
#[path = "context_selector_reference_tests.rs"]
mod tests;
