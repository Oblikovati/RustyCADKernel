//! Debug presentation cleanup follows live sequence links without holding callback locks.
use super::SelectMgrViewerSelectorBase;
use crate::v3d::V3dView;
use rustycad_tkernel::{
    ncollection::SequenceCursor,
    standard::{Failure, Handle, exceptions::null_object},
};
use rustycad_tkservice::graphic3d::{Graphic3dStructureNetwork, Graphic3dStructureSource};
pub(super) fn clear_selector_sensitive(
    base: &SelectMgrViewerSelectorBase,
    view: &Handle<V3dView>,
) -> Result<(), Failure> {
    let mut cursor = SequenceCursor::new(&base.fields.read().debug_structures, true);
    while cursor.more() {
        let structure = cursor
            .value(&base.fields.read().debug_structures)
            .cloned()
            .unwrap_or_default();
        let structure = structure.get().ok_or_else(|| {
            null_object("debug structure is null; expected a live structure at the sequence cursor")
        })?;
        structure.erase();
        structure.clear(true);
        structure.remove();
        cursor.next(&base.fields.read().debug_structures);
    }
    base.fields.write().debug_structures.clear(None);
    if let Some(view) = view.get() {
        view.update()?;
    }
    Ok(())
}
