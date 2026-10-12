//! Actual old-owner destruction reads the original output pointer, not the incoming entity.
use super::{StdSelectBrepSelectionTool, brep_edge_output_owner::*};
use crate::{
    select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrSelection},
    select3d::*,
};
use rustycad_tkernel::standard::{Handle, IntrusiveReferenceSlot, SharedState};
use rustycad_tkmath::gp::Point3;
use std::{fmt::Write, sync::Arc};
#[test]
fn native_edge_output_keeps_prior_pointer_visible_until_owner_destructor_returns() {
    let incoming: Handle<dyn SelectMgrEntityOwnerSource> =
        SelectMgrEntityOwner::new(21).into_cast();
    let slot = Arc::new(IntrusiveReferenceSlot::default());
    let rows = Arc::new(SharedState::new(String::new()));
    seed_retiring_output(&slot, &incoming, &rows);
    replace_with_native_segment(&slot, &incoming);
    record_finished_output(&slot, &incoming, &rows);
    let native = include_str!("../../tests/fixtures/colored-shape-compute/native-edge-output.txt");
    assert_eq!(*rows.read(), native);
}
fn seed_retiring_output(
    slot: &Arc<IntrusiveReferenceSlot<dyn Select3DSensitiveEntity>>,
    incoming: &Handle<dyn SelectMgrEntityOwnerSource>,
    rows: &Arc<SharedState<String>>,
) {
    let input = Arc::new(EdgeOutputRetirementInput {
        slot: Arc::downgrade(slot),
        incoming: incoming.downgrade(),
        previous: Default::default(),
        rows: rows.clone(),
    });
    let owner = EdgeOutputRetiringOwner::new(&input).into_cast();
    let entity: Handle<dyn Select3DSensitiveEntity> =
        Handle::new(Select3DSensitivePoint::new(&owner, Point3::new(9., 8., 7.))).cast();
    *input.previous.write() = entity.downgrade();
    slot.assign(&entity).unwrap();
}
fn replace_with_native_segment(
    slot: &IntrusiveReferenceSlot<dyn Select3DSensitiveEntity>,
    incoming: &Handle<dyn SelectMgrEntityOwnerSource>,
) {
    let edge = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_points(
        Point3::new(0., 0., 0.),
        Point3::new(2., 0., 0.),
    )
    .unwrap()
    .edge()
    .unwrap();
    let selection = Handle::new(SelectMgrSelection::new(0));
    StdSelectBrepSelectionTool::get_edge_sensitive_into(
        &edge, incoming, &selection, 0.1, 0.5, 9, 500., slot,
    )
    .unwrap();
}
fn record_finished_output(
    slot: &IntrusiveReferenceSlot<dyn Select3DSensitiveEntity>,
    incoming: &Handle<dyn SelectMgrEntityOwnerSource>,
    rows: &Arc<SharedState<String>>,
) {
    let entity = slot.observe();
    let segment = i32::from(entity.is_kind(
        <Select3DSensitiveSegment as rustycad_tkernel::standard::RuntimeTyped>::static_type(),
    ));
    let count = entity.ref_count();
    let owners = incoming.ref_count();
    let same = i32::from(entity.owner_id() == *incoming);
    writeln!(
        &mut *rows.write(),
        "output-finished {segment} {count} {owners} {same}"
    )
    .unwrap();
}
