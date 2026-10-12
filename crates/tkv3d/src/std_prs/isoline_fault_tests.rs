//! Native checked node faults preserve existing groups and resume through repair on the same object.
use super::{
    StdPrsIsolineTriangulationInputs, StdPrsIsolines, isoline_triangle_fixture::*,
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
    standard::{Handle, SharedState},
};
use rustycad_tkmath::{
    poly::{Triangle, Triangulation},
    toploc::Location,
};
use std::sync::Arc;
const NATIVE_FAULTS: &str = include_str!("../../tests/fixtures/isolines/native-faults.txt");
#[test]
fn native_checked_triangle_node_failure_then_repair_preserves_retained_presentation() {
    let mut rows = NATIVE_FAULTS.lines();
    for scenario in 0..2 {
        replay_isoline_node_fault(scenario, &mut rows);
    }
    assert!(rows.next().is_none());
}
fn replay_isoline_node_fault(scenario: i32, rows: &mut std::str::Lines<'_>) {
    let source = SharedState::new(isoline_triangle_surface(1));
    let mesh = SharedState::new(isoline_triangle_grid(&source.snapshot(), 0));
    let location = SharedState::new(Location::default());
    let parameters = fault_isoline_parameters();
    let inputs = fault_triangle_input_references(&mesh, &source, &location, &parameters);
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    for step in 0..3 {
        update_isoline_fault_triangle(&mesh.snapshot(), scenario, step);
        let failed =
            StdPrsIsolines::add_on_triangulation_parameters(&prs, &drawer, inputs).is_err();
        let actual = isoline_presentation_snapshot(&prs, &events, scenario * 3 + step, failed);
        compare_oracle_snapshot(&actual, rows.next().unwrap());
    }
}
fn update_isoline_fault_triangle(mesh: &Triangulation, scenario: i32, step: i32) {
    let node = if step == 1 {
        if scenario == 0 { 0 } else { 10 }
    } else {
        1
    };
    let triangle = Triangle::new(node, 2, 5);
    mesh.set_triangle(1, triangle).unwrap();
}
fn fault_isoline_parameters() -> [SharedState<Sequence<f64>>; 2] {
    let parameters = [
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    ];
    parameters[0].write().append(0.25);
    parameters[1].write().append(0.5);
    parameters
}

fn fault_triangle_input_references<'a>(
    mesh: &'a SharedState<Handle<Triangulation>>,
    source: &'a SharedState<Handle<dyn rustycad_tkg3d::geom::GeomSurface>>,
    location: &'a SharedState<Location>,
    parameters: &'a [SharedState<Sequence<f64>>; 2],
) -> StdPrsIsolineTriangulationInputs<'a> {
    StdPrsIsolineTriangulationInputs {
        mesh,
        surface: source,
        location,
        parameters: parameters.each_ref(),
    }
}
