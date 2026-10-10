//! SelectionManager's native friend operations; no public activation shortcut.
use super::*;
use rustycad_tkernel::standard::Failure;

pub(super) fn activate_selector_mode(
    selector: &dyn SelectMgrViewerSelectorSource,
    selection: &SelectMgrSelection,
) {
    for entity in selection.entities() {
        entity.set_active_for_selection();
    }
    if selection.get_selection_state() == SelectMgrStateOfSelection::Activated {
        return;
    }
    selection.set_selection_state(SelectMgrStateOfSelection::Activated);
    selector
        .viewer_selector_base()
        .tolerances
        .write()
        .add(selection.sensitivity());
}
pub(super) fn deactivate_selector_mode(
    selector: &dyn SelectMgrViewerSelectorSource,
    selection: &SelectMgrSelection,
) -> Result<(), Failure> {
    for entity in selection.entities() {
        entity.reset_selection_active_status();
    }
    if selection.get_selection_state() != SelectMgrStateOfSelection::Activated {
        return Ok(());
    }
    selection.set_selection_state(SelectMgrStateOfSelection::Deactivated);
    selector
        .viewer_selector_base()
        .tolerances
        .write()
        .decrement(selection.sensitivity())
}
