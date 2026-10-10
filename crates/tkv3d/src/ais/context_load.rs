//! Register erased AIS objects before display or selection activation (OCCT 3d097a0).
use super::{AisInteractiveContext, AisInteractiveObject};
use crate::{prs_mgr::*, select_mgr::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::Graphic3dPresentationAttributesMembers;
impl AisInteractiveContext {
    /// Register an object without computing presentations or activating selection.
    /// Example: `context.load(&object, -1)?`.
    pub fn load(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        self.set_context_to_object(object)?;
        if self.object_status(object).is_null() {
            self.register_loaded_object(object, mode)?;
        }
        let manager = self.selection_manager().clone();
        if !manager.contains(&object.cast()) {
            manager.load(&object.cast(), -1)?;
        }
        Ok(())
    }
    /// Preserve the obsolete Load overload's ignored boolean; `context.load_legacy(&object, 0, true)?`.
    pub fn load_legacy(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
        _: bool,
    ) -> Result<(), Failure> {
        self.load(object, mode)
    }
    fn register_loaded_object(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        mode: i32,
    ) -> Result<(), Failure> {
        let (mut display, mut highlight, mut selection) = (0, -1, -1);
        self.get_def_modes(object, &mut display, &mut highlight, &mut selection);
        self.set_object_status(
            object,
            PrsMgrDisplayStatus::Erased,
            display,
            if mode != -1 { mode } else { selection },
        )?;
        object.view_affinity().set_visible(true);
        self.current_viewer()
            .structure_manager()
            .register_object(&object.cast(), &object.view_affinity());
        Ok(())
    }
    /// Resolve native defaults; null objects leave all three outputs unchanged.
    /// Example: `context.get_def_modes(&object, &mut display, &mut highlight, &mut selection)`.
    pub fn get_def_modes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        display: &mut i32,
        highlight: &mut i32,
        selection: &mut i32,
    ) {
        if object.is_null() {
            return;
        }
        *display = if object.has_display_mode() {
            object.display_mode()
        } else {
            self.accepted_default_mode(object)
        };
        *highlight = if object.has_hilight_mode() {
            object.hilight_mode()
        } else {
            *display
        };
        *selection = object.global_selection_mode();
    }
    fn accepted_default_mode(&self, object: &Handle<dyn AisInteractiveObject>) -> i32 {
        let candidate = self.default_drawer().display_mode();
        // The virtual acceptance callback may replace the drawer; reread native state afterward.
        if object.accept_display_mode(candidate) {
            self.default_drawer().display_mode()
        } else {
            0
        }
    }
}
