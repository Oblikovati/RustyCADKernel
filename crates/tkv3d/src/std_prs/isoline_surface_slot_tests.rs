//! Retained application objects make const-handle-slot replacement defined and observable during sampling.
use super::{
    StdPrsIsolineSurfaceSource, StdPrsIsolines, isoline_adaptor_fake::IsolineAdaptorFake,
    isoline_face_fixture::isoline_fixture_face,
    isoline_triangle_fixture::isoline_triangle_location,
    isoline_triangle_tests::isoline_presentation_snapshot,
};
use crate::{
    ais::xr_tests::{
        xr_recording_group::XrRecordingEvents, xr_recording_structure::xr_recording_presentation,
    },
    prs3d::Prs3dDrawer,
    select3d::compare_oracle_snapshot,
};
use rustycad_tkernel::{
    ncollection::Sequence,
    standard::{Handle, RuntimeTyped, SharedState},
};
use std::sync::Arc;
const NATIVE_SLOT: &str = include_str!("../../tests/fixtures/isolines/native-surface-slot.txt");
#[test]
fn native_const_surface_handle_slot_replacement_preserves_live_dispatch() {
    let first = Handle::new(IsolineAdaptorFake::new());
    let second = translated_surface_slot_source();
    *first.fail.write() = false;
    *second.fail.write() = false;
    let slot = Arc::new(SharedState::new(
        first.clone().cast::<dyn StdPrsIsolineSurfaceSource>(),
    ));
    first.replace_source_on_kind_query(&slot, &second.clone().cast());
    replay_surface_slot_groups(&slot);
    compare_surface_slot_state(&slot, &first, &second);
}
fn compare_surface_slot_state(
    slot: &SharedState<Handle<dyn StdPrsIsolineSurfaceSource>>,
    first: &IsolineAdaptorFake,
    second: &Handle<IsolineAdaptorFake>,
) {
    let snapshot = [
        1.,
        f64::from(*first.points.read()),
        f64::from(*second.points.read()),
        f64::from(slot.read().object_address() == second.object_address()),
    ];
    compare_oracle_snapshot(&snapshot, NATIVE_SLOT.lines().nth(1).unwrap());
    assert!(std::ptr::eq(
        <dyn StdPrsIsolineSurfaceSource>::static_type(),
        rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface::static_type()
    ));
}
fn translated_surface_slot_source() -> Handle<IsolineAdaptorFake> {
    let mut face = isoline_fixture_face(0);
    face.set_location(&isoline_triangle_location(true).snapshot(), false)
        .unwrap();
    let source = Handle::new(IsolineAdaptorFake::new());
    source.reinitialize_core(&face);
    source
}
fn replay_surface_slot_groups(slot: &SharedState<Handle<dyn StdPrsIsolineSurfaceSource>>) {
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    let parameters = surface_slot_parameters();
    StdPrsIsolines::add_on_surface_references(&prs, slot, &drawer, 0.2, parameters.each_ref())
        .unwrap();
    compare_oracle_snapshot(
        &isoline_presentation_snapshot(&prs, &events, 0, false),
        NATIVE_SLOT.lines().next().unwrap(),
    );
}
fn surface_slot_parameters() -> [SharedState<Sequence<f64>>; 2] {
    let parameters = [
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    ];
    for value in [0., 1.] {
        parameters[0].write().append(value);
    }
    for value in [0., 1., 2.] {
        parameters[1].write().append(value);
    }
    parameters
}
