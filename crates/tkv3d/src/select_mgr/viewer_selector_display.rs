//! Debug display preserves structure append, redisplay, and view-update order.
use super::*;
use crate::v3d::V3dViewSource;
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, v3d::V3dView};
use rustycad_tkernel::{
    ncollection::SequenceCursor,
    standard::{Failure, Handle},
};
use rustycad_tkmath::gp::Transform3;
use rustycad_tkservice::graphic3d::{
    Graphic3dDisplayPriority, Graphic3dStructure, Graphic3dStructureDisplay,
    Graphic3dStructureSource,
};
/// Sensitive debug rendering; `selector.display_sensitive(&view)?`.
pub trait SelectMgrViewerSelectorDisplayMembers: SelectMgrViewerSelectorSource {
    /// Append active selections for each object, redisplay every retained structure, and update.
    /// Example: `selector.display_sensitive(&view)?`.
    fn display_sensitive(&self, view: &V3dView) -> Result<(), Failure> {
        let base = self.viewer_selector_base();
        let mut objects = SelectMgrSelectableObjectSetIterator::new(&base.selectable_objects);
        while objects.more() {
            let structure =
                Graphic3dStructure::new(&view.viewer().structure_manager(), &Handle::default())?;
            display_object_geometry(&structure, &objects.value()?)?;
            base.fields.write().debug_structures.append(structure);
            objects.next();
        }
        display_retained_sensitive(base)?;
        view.update()
    }
    /// Optionally clear retained debug structures, append this selection, then update the view.
    /// Example: `selector.display_sensitive_selection(&selection, &location, &view, true)?`.
    fn display_sensitive_selection(
        &self,
        selection: &Handle<SelectMgrSelection>,
        location: &Transform3,
        view: &Handle<dyn V3dViewSource>,
        clear_others: bool,
    ) -> Result<(), Failure> {
        if clear_others {
            self.clear_sensitive(view)?;
        }
        let structure =
            Graphic3dStructure::new(&view.viewer().structure_manager(), &Handle::default())?;
        SelectMgr::compute_sensitive_prs(
            &structure.cast(),
            selection,
            location,
            &Handle::default(),
        )?;
        let base = self.viewer_selector_base();
        base.fields
            .write()
            .debug_structures
            .append(structure.clone());
        display_sensitive_structure(&structure)?;
        view.update()
    }
}
impl<S: SelectMgrViewerSelectorSource + ?Sized> SelectMgrViewerSelectorDisplayMembers for S {}
fn display_object_geometry(
    structure: &Handle<Graphic3dStructure>,
    object: &Handle<dyn SelectMgrSelectableObject>,
) -> Result<(), Failure> {
    let mut index = 0;
    while let Some(selection) = display_selection_at(object, index) {
        if selection.get_selection_state() == SelectMgrStateOfSelection::Activated {
            SelectMgr::compute_sensitive_prs(
                &structure.cast(),
                &selection,
                &object.transformation(),
                &object.transform_persistence(),
            )?;
        }
        index += 1;
    }
    Ok(())
}
fn display_selection_at(
    object: &Handle<dyn SelectMgrSelectableObject>,
    index: usize,
) -> Option<Handle<SelectMgrSelection>> {
    // Only retain the current selection across callbacks; future selections may be removed.
    object
        .selectable_base()
        .fields
        .read()
        .selections
        .get(index)
        .cloned()
}
fn display_retained_sensitive(base: &SelectMgrViewerSelectorBase) -> Result<(), Failure> {
    let mut cursor = SequenceCursor::new(&base.fields.read().debug_structures, true);
    while cursor.more() {
        let structure = cursor
            .value(&base.fields.read().debug_structures)
            .cloned()
            .unwrap_or_default();
        display_sensitive_structure(&structure)?;
        cursor.next(&base.fields.read().debug_structures);
    }
    Ok(())
}
fn display_sensitive_structure(structure: &Handle<Graphic3dStructure>) -> Result<(), Failure> {
    structure.set_display_priority(Graphic3dDisplayPriority::Topmost)?;
    structure.display();
    Ok(())
}
