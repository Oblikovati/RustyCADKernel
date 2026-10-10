//! Context transforms keep selection and dynamic highlights synchronized (OCCT 3d097a0).
use super::{AisInteractiveContext, AisInteractiveObject};
use crate::prs_mgr::{PrsMgrPresentableObjectMembers, PrsMgrPresentationManager};
use crate::select_mgr::{SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource};
use crate::v3d::V3dViewer;
use rustycad_tkernel::standard::{Failure, Handle, exceptions::null_object};
use rustycad_tkmath::toploc::Location;

impl AisInteractiveContext {
    /// Replace local placement and update selection without redrawing; `context.set_location(&object, &location)?`.
    pub fn set_location(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        location: &Location,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        if location.is_identity() {
            return if object.has_transformation() {
                self.reset_location(object)
            } else {
                Ok(())
            };
        }
        self.apply_location_transform(object, location)
    }

    fn apply_location_transform(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        location: &Location,
    ) -> Result<(), Failure> {
        if object.has_transformation() {
            object.reset_transformation()?;
        }
        object.set_local_transformation(&location.transformation())?;
        let manager = self.selection_manager().clone();
        manager.update(&object.cast(), false)?;
        self.refresh_location_highlight(object)
    }

    /// Clear local placement and update selection, even when already identity; `context.reset_location(&object)?`.
    pub fn reset_location(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        object.reset_transformation()?;
        let manager = self.selection_manager().clone();
        manager.update(&object.cast(), false)
    }

    /// Query effective placement including parents; `context.has_location(&object)`.
    pub fn has_location(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        !object.is_null() && object.has_transformation()
    }

    /// Copy absolute placement into a fresh datum; `let location = context.location(&object)?`.
    pub fn location(&self, object: &Handle<dyn AisInteractiveObject>) -> Result<Location, Failure> {
        let object = object.get().ok_or_else(|| {
            null_object("location object is null; expected a live AIS interactive object")
        })?;
        Ok(Location::new(object.transformation()))
    }

    fn refresh_location_highlight(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        let (owner, viewer, manager) = self.location_highlight_handles();
        if owner.is_null() || !owner.is_same_selectable(&object.cast()) {
            return Ok(());
        }
        let mode = if object.has_hilight_mode() {
            object.hilight_mode()
        } else {
            0
        };
        owner.update_highlight_trsf(&viewer, &manager, mode)
    }

    fn location_highlight_handles(
        &self,
    ) -> (
        Handle<dyn SelectMgrEntityOwnerSource>,
        Handle<V3dViewer>,
        Handle<PrsMgrPresentationManager>,
    ) {
        // Release the context guard before virtual callbacks that may reenter the context.
        let fields = self.fields.read();
        (
            fields.last_picked.clone(),
            fields.viewer.clone(),
            fields.presentation_manager.clone(),
        )
    }
}
