//! Real curve replacement reaches incoming selection through both ColoredShape visibility policies.
use super::{
    brep_contour_mutation_curve::BrepContourMutationCurveFake, brep_selection_slot_tests::*,
};
use crate::select3d::Select3DSensitiveEntityMembers;
use crate::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, SharedState};
use rustycad_tkmath::gp::Point3;
use std::{fmt::Write, sync::Arc};
#[test]
fn native_colored_primary_selection_forwards_original_slot_through_visible_and_hidden_paths() {
    let rows = Arc::new(SharedState::new(String::new()));
    for hidden in [false, true] {
        replay_colored_selection_slot(hidden, &rows);
    }
    let native = include_str!(
        "../../tests/fixtures/colored-shape-compute/native-colored-selection-slot.txt"
    );
    assert_eq!(*rows.read(), native);
}
fn replay_colored_selection_slot(hidden: bool, rows: &Arc<SharedState<String>>) {
    writeln!(
        &mut *rows.write(),
        "colored-selection-slot-case {}",
        i32::from(hidden)
    )
    .unwrap();
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let object = AisColoredShape::new(&edge);
    configure_colored_selection_slot(&object, hidden);
    let previous = Handle::new(SelectMgrSelection::new(0));
    let incoming = Handle::new(SelectMgrSelection::new(0));
    let slot = Arc::new(SharedState::new(previous.clone()));
    activate_selection_slot_hook(&curve, &slot, &incoming, rows);
    let reference = HandleFieldRef::new(&slot, selection_slot_identity);
    invoke_colored_primary_selection(&object, &reference);
    record_colored_selection_slot(&previous, &incoming, rows);
}
fn invoke_colored_primary_selection(
    object: &Handle<AisColoredShape>,
    selection: &dyn rustycad_tkernel::standard::HandleReference<SelectMgrSelection>,
) {
    let source = object.observe_as::<dyn SelectMgrSelectableObject>();
    source.compute_selection(selection, 0).unwrap();
}
fn configure_colored_selection_slot(object: &AisColoredShape, hidden: bool) {
    object.attributes().setup_own_defaults().unwrap();
    object.attributes().set_auto_triangulation(false);
    if !hidden {
        return;
    }
    let vertex = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex::new(Point3::new(
        9., 8., 7.,
    ))
    .unwrap()
    .vertex()
    .unwrap();
    object.custom_aspects(&vertex).unwrap().set_hidden(true);
}
fn record_colored_selection_slot(
    previous: &Handle<SelectMgrSelection>,
    incoming: &Handle<SelectMgrSelection>,
    rows: &SharedState<String>,
) {
    let old = previous.ref_count();
    let next = incoming.ref_count();
    let old_entities = previous.entities().len();
    let new_entities = incoming.entities().len();
    let owner = incoming.entities()[0].base_sensitive().owner_id();
    let linked = i32::from(owner.has_selectable());
    writeln!(
        &mut *rows.write(),
        "colored-selection-slot-finished {old} {next} {old_entities} {new_entities} {linked}"
    )
    .unwrap();
}
