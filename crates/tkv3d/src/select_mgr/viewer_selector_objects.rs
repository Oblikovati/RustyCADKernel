//! Object and selection membership, with native update ordering.
use super::*;
use crate::select3d::Select3DSensitiveEntityMembers;
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle, exceptions::null_object},
    tcollection::AsciiString,
};
use rustycad_tkmath::bvh::BvhObjectTransientSource;

/// Nonvirtual lifecycle members; `selector.add_selection_to_object(&object, &selection)?`.
pub trait SelectMgrViewerSelectorObjectMembers: SelectMgrViewerSelectorSource {
    /// Test object containment; `selector.contains(&object)`.
    fn contains(&self, object: &Handle<dyn SelectMgrSelectableObject>) -> bool {
        self.viewer_selector_base()
            .selectable_objects
            .contains(object)
    }
    /// Add an object once with a sensitive set using the current builder; `selector.add_selectable_object(&object)?`.
    fn add_selectable_object(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<(), Failure> {
        let base = self.viewer_selector_base();
        if base.object_sensitives.read().is_bound(object) {
            return Ok(());
        }
        base.selectable_objects.append(object)?;
        let set = Handle::new(SelectMgrSensitiveEntitySet::new(self.entity_set_builder()));
        base.object_sensitives.write().bind(object.clone(), set);
        Ok(())
    }
    /// Append the selection and build its sensitive BVH immediately; `selector.add_selection_to_object(&object, &selection)?`.
    fn add_selection_to_object(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<(), Failure> {
        self.add_selectable_object(object)?;
        let set = selector_sensitive_set(self, object)?;
        set.append_selection(selection)?;
        set.bvh()?;
        Ok(())
    }
    /// Reclassify persistence while retaining membership; `selector.move_selectable_object(&object)?`.
    fn move_selectable_object(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<(), Failure> {
        self.viewer_selector_base()
            .selectable_objects
            .change_subset(object)
    }
    /// Unbind the sensitive set, remove picks, then remove the object; `selector.remove_selectable_object(&object)?`.
    fn remove_selectable_object(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<(), Failure> {
        if self
            .viewer_selector_base()
            .object_sensitives
            .write()
            .unbind(object)
        {
            self.remove_picked(object)?;
            self.viewer_selector_base()
                .selectable_objects
                .remove(object)?;
        }
        Ok(())
    }
    /// Remove selection sensitives from an existing set; `selector.remove_selection_of_object(&object, &selection)?`.
    fn remove_selection_of_object(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<(), Failure> {
        let set = self
            .viewer_selector_base()
            .object_sensitives
            .read()
            .seek(object)
            .cloned();
        if let Some(set) = set {
            set.remove(selection)?;
        }
        Ok(())
    }
    /// Dirty every object subset, optionally rebuilding now; `selector.rebuild_objects_tree(true)?`.
    fn rebuild_objects_tree(&self, force: bool) -> Result<(), Failure> {
        let base = self.viewer_selector_base();
        base.selectable_objects.mark_dirty();
        if force {
            let volume = base.selecting_volume.snapshot();
            base.selectable_objects.update_bvh(
                &volume.camera(),
                volume.window_size().unwrap_or((0, 0)).into(),
            )?;
        }
        Ok(())
    }
    /// Dirty one contained object's sensitive set; `selector.rebuild_sensitives_tree(&object, true)?`.
    fn rebuild_sensitives_tree(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        force: bool,
    ) -> Result<(), Failure> {
        if !self.contains(object) {
            return Ok(());
        }
        let set = selector_sensitive_set(self, object)?;
        set.mark_dirty();
        if force {
            set.bvh()?;
        }
        Ok(())
    }
    /// Append matching modes even for an absent object; bool reports containment.
    /// Example: `selector.modes(&object, &mut modes, SelectMgrStateOfSelection::Any)?`.
    fn modes(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        modes: &mut List<i32>,
        wanted: SelectMgrStateOfSelection,
    ) -> Result<bool, Failure> {
        for selection in selector_object(object)?.selections() {
            if wanted == SelectMgrStateOfSelection::Any || wanted == selection.get_selection_state()
            {
                modes.append(selection.mode());
            }
        }
        Ok(self.contains(object))
    }
    /// Check activation only for a contained object; `selector.is_active(&object, 0)?`.
    fn is_active(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        mode: i32,
    ) -> Result<bool, Failure> {
        if !self.contains(object) {
            return Ok(false);
        }
        let selection = select_mgr_selection_of(selector_object(object)?, mode);
        Ok(selection.get().is_some_and(|selection| {
            selection.get_selection_state() == SelectMgrStateOfSelection::Activated
        }))
    }
    /// Check that a contained selection has a known state; `selector.is_inside(&object, 0)?`.
    fn is_inside(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
        mode: i32,
    ) -> Result<bool, Failure> {
        if !self.contains(object) {
            return Ok(false);
        }
        let selection = select_mgr_selection_of(selector_object(object)?, mode);
        Ok(selection.get().is_some_and(|selection| {
            selection.get_selection_state() != SelectMgrStateOfSelection::Unknown
        }))
    }
    /// Return the selection's state independently of containment; `selector.selection_status(&selection)?`.
    fn selection_status(
        &self,
        selection: &Handle<SelectMgrSelection>,
    ) -> Result<SelectMgrStateOfSelection, Failure> {
        Ok(selection
            .get()
            .ok_or_else(|| null_object("selection is null; expected non-null selection"))?
            .get_selection_state())
    }
    /// Return native textual mode status; `selector.object_status(&object)?`.
    fn object_status(
        &self,
        object: &Handle<dyn SelectMgrSelectableObject>,
    ) -> Result<AsciiString, Failure> {
        let mut status = String::from("Status Object :\n\t");
        for selection in selector_object(object)?.selections() {
            append_selection_status(&mut status, &selection);
        }
        if !self.contains(object) {
            status.push_str("Not Present in the selector\n\n");
        }
        Ok(AsciiString::new(&status))
    }
    /// Clear active sensitive flags, retaining mode states and tolerance counts; `selector.reset_selection_activation_status()?`.
    fn reset_selection_activation_status(&self) -> Result<(), Failure> {
        for set in selector_sensitive_sets(self) {
            for index in 0..set.size() {
                set.get_sensitive_by_id(index)?
                    .reset_selection_active_status();
            }
        }
        Ok(())
    }
    /// Append owners of active sensitives, retaining duplicates and null owners; `selector.active_owners(&mut owners)?`.
    fn active_owners(
        &self,
        owners: &mut List<Handle<dyn SelectMgrEntityOwnerSource>>,
    ) -> Result<(), Failure> {
        for set in selector_sensitive_sets(self) {
            for index in 0..set.size() {
                let sensitive = set.get_sensitive_by_id(index)?;
                if sensitive.is_active_for_selection() {
                    owners.append(sensitive.base_sensitive().owner_id());
                }
            }
        }
        Ok(())
    }
}
impl<T: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorObjectMembers for T {}
fn selector_object(
    object: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<&dyn SelectMgrSelectableObject, Failure> {
    object
        .get()
        .ok_or_else(|| null_object("selectable object is null; expected non-null object"))
}
pub(super) fn selector_sensitive_set<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
    object: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<Handle<SelectMgrSensitiveEntitySet>, Failure> {
    Ok(selector
        .viewer_selector_base()
        .object_sensitives
        .read()
        .find(object)?
        .clone())
}
fn selector_sensitive_sets<S: SelectMgrViewerSelectorSource + ?Sized>(
    selector: &S,
) -> Vec<Handle<SelectMgrSensitiveEntitySet>> {
    selector
        .viewer_selector_base()
        .object_sensitives
        .read()
        .items()
        .map(|(_, set)| set.clone())
        .collect()
}
fn append_selection_status(status: &mut String, selection: &SelectMgrSelection) {
    let state = selection.get_selection_state();
    if state == SelectMgrStateOfSelection::Unknown {
        return;
    }
    let suffix = if state == SelectMgrStateOfSelection::Activated {
        " Active \n\t"
    } else {
        " Inactive \n\t"
    };
    status.push_str(&format!("Mode {} present - {suffix}", selection.mode()));
}
