//! Selected application owners follow the selection cursor, independently of detection.
use super::{AisInteractiveContext, AisInteractiveObject, AisInteractiveObjectMembers};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, exceptions::null_object};

impl AisInteractiveContext {
    /// Test the selected object's application owner; `context.has_applicative()?`.
    pub fn has_applicative(&self) -> Result<bool, Failure> {
        let object = selected_application_object(self)?;
        Ok(object.has_owner())
    }

    /// Retain the selected application owner independently of later edits; `context.applicative()?`.
    pub fn applicative(&self) -> Result<Handle<StandardTransient>, Failure> {
        let object = selected_application_object(self)?;
        let application = object.get_owner().clone();
        Ok(application)
    }
}

fn selected_application_object(
    context: &AisInteractiveContext,
) -> Result<Handle<dyn AisInteractiveObject>, Failure> {
    let object = context.selected_interactive()?;
    if object.is_null() {
        // Native dereferences this null result; Rust diagnoses the invalid precondition.
        return Err(null_object(
            "selected interactive address=0; expected a live AIS object",
        ));
    }
    Ok(object)
}
