//! Native const owner references reread a slot replaced by a real curve Value callback.
use super::{
    StdSelectBrepSelectionTool, brep_contour_mutation_curve::BrepContourMutationCurveFake,
    brep_owner_slot_curve_callback::BrepCurveSlotReplacement,
};
use crate::{select_mgr::*, select3d::*};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, HandleReference, SharedState};
use std::{fmt::Write, sync::Arc};
#[test]
fn native_curve_callback_replaces_const_owner_slot_before_sensitive_construction() {
    let mut rows = String::new();
    for compute in [false, true] {
        rows.push_str(&owner_slot_case(compute));
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/colored-shape-compute/native-owner-slot.txt")
    );
}
fn owner_slot_case(compute: bool) -> String {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let previous: Handle<dyn SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(19).into_cast();
    let incoming: Handle<dyn SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(21).into_cast();
    let slot = Arc::new(SharedState::new(previous.clone()));
    let scenario = i32::from(compute);
    let rows = Arc::new(SharedState::new(format!("slot-case {scenario}\n")));
    *curve.slot_callback.write() = Some(Box::new(BrepCurveSlotReplacement::new(
        &slot, &incoming, &rows, "slot",
    )));
    let reference = HandleFieldRef::new(&slot, owner_slot_identity);
    let selection = Handle::new(SelectMgrSelection::new(0));
    let entity = invoke_owner_slot_selection(compute, &edge, &reference, &selection);
    record_owner_slot_finish(&entity, &previous, &incoming, &selection, &rows);
    rows.read().clone()
}

fn owner_slot_identity(
    slot: &Handle<dyn SelectMgrEntityOwnerSource>,
) -> &Handle<dyn SelectMgrEntityOwnerSource> {
    // The native fixture passes the actual slot, which the Value callback later replaces.
    slot
}
fn invoke_owner_slot_selection(
    compute: bool,
    edge: &rustycad_tkbrep::topods::Edge,
    reference: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    selection: &Handle<SelectMgrSelection>,
) -> Handle<dyn Select3DSensitiveEntity> {
    if compute {
        StdSelectBrepSelectionTool::compute_sensitive(
            edge, reference, selection, 0.1, 0.5, 9, 500., false,
        )
        .unwrap();
        return selection.entities()[0].base_sensitive();
    }
    owner_slot_direct_edge(edge, reference, selection)
}
fn owner_slot_direct_edge(
    edge: &rustycad_tkbrep::topods::Edge,
    reference: &dyn HandleReference<dyn SelectMgrEntityOwnerSource>,
    selection: &Handle<SelectMgrSelection>,
) -> Handle<dyn Select3DSensitiveEntity> {
    let mut output = Handle::default();
    StdSelectBrepSelectionTool::get_edge_sensitive(
        edge,
        reference,
        selection,
        0.1,
        0.5,
        9,
        500.,
        &mut output,
    )
    .unwrap();
    output
}

fn record_owner_slot_finish(
    entity: &Handle<dyn Select3DSensitiveEntity>,
    previous: &Handle<dyn SelectMgrEntityOwnerSource>,
    incoming: &Handle<dyn SelectMgrEntityOwnerSource>,
    selection: &SelectMgrSelection,
    rows: &SharedState<String>,
) {
    let owner = entity.owner_id();
    let same = i32::from(owner.object_address() == incoming.object_address());
    let priority = owner.priority();
    drop(owner);
    let counts = (previous.ref_count(), incoming.ref_count());
    let (old_count, new_count) = counts;
    let count = selection.entities().len();
    writeln!(
        &mut *rows.write(),
        "slot-finished {priority} {same} {old_count} {new_count} {count}"
    )
    .unwrap();
}
