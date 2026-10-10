//! Native context membership and object display flags are independent state.
use super::{
    AisGlobalStatus, AisInteractiveContext, AisInteractiveObject, AisInteractiveObjectMembers,
};
use crate::prs_mgr::{PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers};
use rustycad_tkernel::standard::{Failure, Handle, exceptions::null_object};
impl AisInteractiveContext {
    /// Set protected native status recursively, replacing each registered global status.
    /// Example: `context.set_object_status(&object, PrsMgrDisplayStatus::Erased, 0, -1)?`.
    pub fn set_object_status(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        state: PrsMgrDisplayStatus,
        display: i32,
        selection: i32,
    ) -> Result<(), Failure> {
        let value = object
            .get()
            .ok_or_else(|| null_object("status object is null; expected a live AIS object"))?;
        value.set_display_status(state);
        self.bind_object_status(object, state, display, selection);
        for child in value.children() {
            let interactive = child.cast::<dyn AisInteractiveObject>();
            if !interactive.is_null() {
                self.set_object_status(&interactive, state, display, selection)?;
            }
        }
        Ok(())
    }
    fn bind_object_status(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        state: PrsMgrDisplayStatus,
        display: i32,
        selection: i32,
    ) {
        if state == PrsMgrDisplayStatus::None {
            self.fields.write().objects.unbind(object);
            return;
        }
        let status = Handle::new(AisGlobalStatus::new());
        status.set_display_mode(display);
        if selection != -1 {
            status.add_selection_mode(selection);
        }
        self.fields.write().objects.bind(object.clone(), status);
    }
    /// Return None for null or unregistered objects; `context.display_status(&object)`.
    pub fn display_status(&self, object: &Handle<dyn AisInteractiveObject>) -> PrsMgrDisplayStatus {
        if object.is_null() || self.object_status(object).is_null() {
            return PrsMgrDisplayStatus::None;
        }
        object.display_status()
    }
    /// Require context membership as well as the object's displayed flag; `context.is_displayed(&object)`.
    pub fn is_displayed(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        self.display_status(object) == PrsMgrDisplayStatus::Displayed
    }
    /// Require the registered display mode too; `context.is_displayed_mode(&object, 0)`.
    pub fn is_displayed_mode(&self, object: &Handle<dyn AisInteractiveObject>, mode: i32) -> bool {
        self.is_displayed(object) && self.object_status(object).display_mode() == mode
    }
}
