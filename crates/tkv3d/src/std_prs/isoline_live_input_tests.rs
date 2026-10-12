//! Actual reference alias changes affect later iso calls, V-axis count and per-segment transformation.
use super::{
    StdPrsIsolineTriangulationInputs, StdPrsIsolines, isoline_alias_surface_fake::*,
    isoline_triangle_fixture::*, isoline_triangle_tests::isoline_presentation_snapshot,
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
use rustycad_tkmath::toploc::Location;
use std::sync::Arc;
const NATIVE_LIVE: &str = include_str!("../../tests/fixtures/isolines/native-live-inputs.txt");
#[test]
fn native_const_input_aliases_are_observed_after_actual_surface_callback() {
    let cells = live_isoline_input_cells();
    let source = Handle::new(IsolineAliasSurfaceFake::new(&cells));
    *cells.source.write() = source.clone().cast();
    let mesh = SharedState::new(isoline_triangle_grid(&isoline_triangle_surface(1), 0));
    let inputs = live_triangle_input_references(&mesh, &cells);
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    StdPrsIsolines::add_on_triangulation_parameters(&prs, &drawer, inputs).unwrap();
    compare_oracle_snapshot(
        &isoline_presentation_snapshot(&prs, &events, 0, false),
        NATIVE_LIVE.lines().next().unwrap(),
    );
    compare_live_isoline_mutations(&cells, &source);
}
fn compare_live_isoline_mutations(cells: &IsolineLiveInputs, source: &IsolineAliasSurfaceFake) {
    let state = [
        1.,
        f64::from(*source.calls.read()),
        cells.parameters[0].read().length() as f64,
        cells.parameters[1].read().length() as f64,
        f64::from(cells.location.read().is_identity()),
    ];
    compare_oracle_snapshot(&state, NATIVE_LIVE.lines().nth(1).unwrap());
}
pub(super) fn live_isoline_input_cells() -> Arc<IsolineLiveInputs> {
    let parameters = [
        SharedState::new(Sequence::default()),
        SharedState::new(Sequence::default()),
    ];
    parameters[0].write().append(0.25);
    parameters[0].write().append(0.5);
    parameters[1].write().append(0.2);
    Arc::new(IsolineLiveInputs {
        source: SharedState::new(Handle::default()),
        location: SharedState::new(Location::default()),
        parameters,
    })
}

fn live_triangle_input_references<'a>(
    mesh: &'a SharedState<Handle<rustycad_tkmath::poly::Triangulation>>,
    cells: &'a IsolineLiveInputs,
) -> StdPrsIsolineTriangulationInputs<'a> {
    StdPrsIsolineTriangulationInputs {
        mesh,
        surface: &cells.source,
        location: &cells.location,
        parameters: cells.parameters.each_ref(),
    }
}
