//! Protected subshape construction projects the original typed BRepOwner slot through geometry callbacks.
use super::{
    StdSelectBRepOwner, brep_contour_mutation_curve::BrepContourMutationCurveFake,
    brep_owner_slot_curve_callback::BrepCurveSlotReplacement,
};
use crate::{
    ais::*, prs_mgr::PrsMgrPresentableObjectMembers, select_mgr::*,
    select3d::Select3DSensitiveEntityMembers,
};
use rustycad_tkernel::standard::{Handle, HandleFieldRef, SharedState};
use rustycad_tkg3d::top_abs::ShapeEnum;
use std::{fmt::Write, sync::Arc};
#[test]
fn native_subshape_construction_uses_rebound_typed_owner_without_owning_upcast() {
    let curve = BrepContourMutationCurveFake::new();
    let edge = BrepContourMutationCurveFake::edge_fixture(&curve);
    let object = AisColoredShape::new(&edge);
    object.attributes().set_auto_triangulation(false);
    let previous = StdSelectBRepOwner::with_shape(&edge, 19, false);
    let incoming = StdSelectBRepOwner::with_shape(&edge, 27, false);
    let slot = Arc::new(SharedState::new(previous.clone()));
    let rows = Arc::new(SharedState::new(String::new()));
    let hook = BrepCurveSlotReplacement::new(&slot, &incoming, &rows, "subshape-owner");
    *curve.slot_callback.write() = Some(Box::new(hook));
    let selection = invoke_subshape_owner_slot(&object, &edge, &slot);
    record_subshape_owner_slot(&previous, &incoming, &selection, &rows);
    let native =
        include_str!("../../tests/fixtures/colored-shape-compute/native-subshape-owner-slot.txt");
    assert_eq!(*rows.read(), native);
}
fn invoke_subshape_owner_slot(
    object: &AisColoredShape,
    shape: &rustycad_tkbrep::topods::Shape,
    slot: &SharedState<Handle<StdSelectBRepOwner>>,
) -> Handle<SelectMgrSelection> {
    let selection = Handle::new(SelectMgrSelection::new(0));
    let parent = Handle::new(AisColoredDrawer::new(&object.attributes()));
    let styles = AisShapeDrawerMap::default();
    let owner = HandleFieldRef::new(slot, subshape_owner_slot_field);
    invoke_subshape_owner_arguments(object, (&parent, &styles, shape, &owner, &selection));
    selection
}
fn subshape_owner_slot_field(owner: &Handle<StdSelectBRepOwner>) -> &Handle<StdSelectBRepOwner> {
    // The upcast must observe this typed caller field again after real Value callbacks.
    owner
}
fn record_subshape_owner_slot(
    previous: &Handle<StdSelectBRepOwner>,
    incoming: &Handle<StdSelectBRepOwner>,
    selection: &SelectMgrSelection,
    rows: &SharedState<String>,
) {
    let entity = selection.entities()[0].base_sensitive();
    let owner = entity.owner_id();
    let old = previous.ref_count();
    let next = incoming.ref_count();
    let priority = owner.priority();
    let same = i32::from(owner.object_address() == incoming.object_address());
    let count = selection.entities().len();
    writeln!(
        &mut *rows.write(),
        "subshape-owner-finished {old} {next} {priority} {same} {count}"
    )
    .unwrap();
}

type SubshapeOwnerArguments<'a> = (
    &'a Handle<AisColoredDrawer>,
    &'a AisShapeDrawerMap,
    &'a rustycad_tkbrep::topods::Shape,
    &'a dyn rustycad_tkernel::standard::HandleReference<StdSelectBRepOwner>,
    &'a Handle<SelectMgrSelection>,
);
fn invoke_subshape_owner_arguments(
    object: &AisColoredShape,
    arguments: SubshapeOwnerArguments<'_>,
) {
    let (parent, styles, shape, owner, selection) = arguments;
    ais_colored_shape_compute_subshape_selection(
        object,
        parent,
        styles,
        shape,
        owner,
        selection,
        ShapeEnum::Shape,
        42,
        0.1,
        0.5,
    )
    .unwrap();
}
