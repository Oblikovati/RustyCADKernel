//! Actual caller selection slots are reread after a geometric Value callback replaces them.
use super::{
    StdSelectBrepSelectionTool, brep_contour_mutation_curve::BrepContourMutationCurveFake,
    brep_owner_slot_curve_callback::BrepCurveSlotReplacement,
};
use crate::select_mgr::{SelectMgrEntityOwner, SelectMgrSelection};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, SharedState};
use std::{fmt::Write, sync::Arc};
#[test]
fn native_curve_callback_replaces_const_selection_slot_before_sensitive_append() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let owner: Handle<dyn crate::select_mgr::SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(21).into_cast();
    let previous = Handle::new(SelectMgrSelection::new(0));
    let incoming = Handle::new(SelectMgrSelection::new(0));
    let slot = Arc::new(SharedState::new(previous.clone()));
    let rows = Arc::new(SharedState::new(String::new()));
    activate_selection_slot_hook(&curve, &slot, &incoming, &rows);
    let reference = HandleFieldRef::new(&slot, selection_slot_identity);
    StdSelectBrepSelectionTool::compute_sensitive(
        &edge, &owner, &reference, 0.1, 0.5, 9, 500., false,
    )
    .unwrap();
    record_selection_slot_finish(&previous, &incoming, owner.ref_count(), &rows);
    let native =
        include_str!("../../tests/fixtures/colored-shape-compute/native-selection-slot.txt");
    assert_eq!(*rows.read(), native);
}
fn activate_selection_slot_hook(
    curve: &BrepContourMutationCurveFake,
    slot: &Arc<SharedState<Handle<SelectMgrSelection>>>,
    incoming: &Handle<SelectMgrSelection>,
    rows: &Arc<SharedState<String>>,
) {
    let hook = BrepCurveSlotReplacement::new(slot, incoming, rows, "selection");
    *curve.slot_callback.write() = Some(Box::new(hook));
}
fn selection_slot_identity(slot: &Handle<SelectMgrSelection>) -> &Handle<SelectMgrSelection> {
    // This is the actual argument slot changed by the native curve callback.
    slot
}
fn record_selection_slot_finish(
    previous: &Handle<SelectMgrSelection>,
    incoming: &Handle<SelectMgrSelection>,
    owner_count: i32,
    rows: &SharedState<String>,
) {
    let counts = (previous.ref_count(), incoming.ref_count());
    let entities = (previous.entities().len(), incoming.entities().len());
    writeln!(
        &mut *rows.write(),
        "selection-finished {} {} {} {} {owner_count}",
        counts.0,
        counts.1,
        entities.0,
        entities.1
    )
    .unwrap();
}
