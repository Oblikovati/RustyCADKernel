//! Native redisplay sequencing over retained context membership and selection modes.
use super::{AisInteractiveContext, AisInteractiveObject};
use crate::prs_mgr::{
    PrsMgrDisplayStatus, PrsMgrPresentableObjectMembers, PrsMgrPresentableObjectUpdates,
};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
impl AisInteractiveContext {
    /// Append active status modes without clearing the caller's list; `context.activated_modes(&object,&mut modes)`.
    pub fn activated_modes(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        modes: &mut List<i32>,
    ) {
        let status = self.fields.read().objects.seek(object).cloned();
        if let Some(status) = status {
            for mode in status.selection_modes().iter() {
                modes.append(*mode);
            }
        }
    }
    /// Recompute presentation then selection, retaining partial changes on failure.
    /// Example: `context.redisplay(&object,false,true)?`.
    pub fn redisplay(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
        all_modes: bool,
    ) -> Result<(), Failure> {
        self.recompute_prs_only(object, update_viewer, all_modes)?;
        self.recompute_selection_only(object)
    }
    /// Mark all modes stale and update eligible presentations; `context.recompute_prs_only(&object,true,false)?`.
    pub fn recompute_prs_only(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
        all_modes: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.set_to_update_all();
        value.update_presentations(all_modes)?;
        if !update_viewer {
            return Ok(());
        }
        let viewer = self.current_viewer().clone();
        if self.is_registered_displayed(object) {
            viewer.update();
        }
        if let Some(view) = viewer.active_views().first() {
            value.recompute_transformation(&view.camera());
        }
        Ok(())
    }
    /// Temporarily deactivate recorded modes, recompute, and reactivate displayed objects.
    /// Example: `context.recompute_selection_only(&object)?`.
    pub fn recompute_selection_only(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
    ) -> Result<(), Failure> {
        if object.is_null() {
            return Ok(());
        }
        let mut modes = List::default();
        self.activated_modes(object, &mut modes);
        let manager = self.selection_manager().clone();
        let selectable = object.cast();
        for mode in modes.iter() {
            manager.deactivate(&selectable, *mode)?;
        }
        manager.recompute_selection(&selectable, false, -1)?;
        if self.is_registered_displayed(object) {
            for mode in modes.iter() {
                manager.activate(&selectable, *mode)?;
            }
        }
        Ok(())
    }
    /// Update existing presentations and selection state, then a registered displayed viewer.
    /// Example: `context.update(&object,true)?`.
    pub fn update(
        &self,
        object: &Handle<dyn AisInteractiveObject>,
        update_viewer: bool,
    ) -> Result<(), Failure> {
        let Some(value) = object.get() else {
            return Ok(());
        };
        value.update_presentations(false)?;
        let manager = self.selection_manager().clone();
        manager.update(&object.cast(), false)?;
        if update_viewer && self.is_registered_displayed(object) {
            self.update_current_viewer();
        }
        Ok(())
    }
    fn is_registered_displayed(&self, object: &Handle<dyn AisInteractiveObject>) -> bool {
        self.fields.read().objects.is_bound(object)
            && object.display_status() == PrsMgrDisplayStatus::Displayed
    }
}
