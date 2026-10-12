//! Native primitive buffers, bounds, disordered triangle insertion and surface-length fitting replay without OCCT.
use super::{StdPrsIsolineTriangulationInputs, StdPrsIsolines, isoline_triangle_fixture::*};
use crate::{
    ais::xr_tests::{
        xr_recording_group::{XrRecordingEvents, XrRecordingGroup},
        xr_recording_structure::xr_recording_presentation,
    },
    prs3d::Prs3dDrawer,
    select3d::compare_oracle_snapshot,
};
use rustycad_tkernel::standard::{Handle, SharedState};
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
use std::sync::Arc;
const NATIVE_TRIANGLES: &str =
    include_str!("../../tests/fixtures/isolines/native-triangulation.txt");
#[test]
fn native_mesh_interpolation_order_locations_and_retained_group_buffers() {
    let mut rows = NATIVE_TRIANGLES.lines();
    for scenario in 0..18 {
        compare_oracle_snapshot(&isoline_triangle_row(scenario), rows.next().unwrap());
    }
    assert!(rows.next().is_none());
}
fn isoline_triangle_row(scenario: i32) -> Vec<f64> {
    let surface = SharedState::new(isoline_triangle_surface(scenario / 6));
    let mesh = SharedState::new(isoline_triangle_grid(&surface.snapshot(), scenario % 3));
    let location = isoline_triangle_location(scenario % 6 >= 3);
    let parameters = isoline_triangle_parameters();
    let inputs = StdPrsIsolineTriangulationInputs {
        mesh: &mesh,
        surface: &surface,
        location: &location,
        parameters: parameters.each_ref(),
    };
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    let failed = StdPrsIsolines::add_on_triangulation_parameters(&prs, &drawer, inputs).is_err();
    isoline_presentation_snapshot(&prs, &events, scenario, failed)
}
pub(super) fn isoline_presentation_snapshot(
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    events: &XrRecordingEvents,
    scenario: i32,
    failed: bool,
) -> Vec<f64> {
    let groups = prs.groups();
    let mut row = vec![f64::from(scenario), f64::from(failed), groups.len() as f64];
    for group in groups {
        row.extend(group.cast::<XrRecordingGroup>().snapshot());
    }
    let callbacks = events.callbacks.read();
    row.push(callbacks.len() as f64);
    row.extend(callbacks.iter().map(|value| f64::from(*value)));
    row
}
