//! Genuine derived face-adaptor hooks retain partial surface output on checked failure, then repair in place.
use super::{
    StdPrsIsolines, isoline_adaptor_fake::IsolineAdaptorFake,
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
use std::sync::Arc;
const NATIVE_ADAPTOR: &str = include_str!("../../tests/fixtures/isolines/native-adaptor.txt");
#[test]
fn native_derived_virtual_failure_preserves_completed_curves_then_repairs() {
    let source = Handle::new(IsolineAdaptorFake::new());
    let parameters = derived_isoline_parameters();
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    for (step, row) in NATIVE_ADAPTOR.lines().enumerate() {
        *source.points.write() = 0;
        *source.fail.write() = step == 0;
        sample_derived_isoline_step(&prs, &source, &drawer, &parameters);
        let mut actual = isoline_presentation_snapshot(&prs, &events, step as i32, false);
        actual[1] = f64::from(*source.points.read());
        compare_oracle_snapshot(&actual, row);
    }
}
fn derived_isoline_parameters() -> [SharedState<Sequence<f64>>; 2] {
    let inputs: [SharedState<Sequence<f64>>; 2] =
        std::array::from_fn(|_| SharedState::new(Sequence::default()));
    for value in [0., 1.] {
        inputs[0].write().append(value);
    }
    for value in [0., 1., 2.] {
        inputs[1].write().append(value);
    }
    inputs
}

fn sample_derived_isoline_step(
    prs: &Handle<dyn rustycad_tkservice::graphic3d::Graphic3dStructureSource>,
    source: &Handle<IsolineAdaptorFake>,
    drawer: &Handle<Prs3dDrawer>,
    parameters: &[SharedState<Sequence<f64>>; 2],
) {
    let parameters = parameters.each_ref();
    StdPrsIsolines::add_on_surface_parameters(prs, source, drawer, 0.2, parameters).unwrap();
}

#[test]
fn surface_sampling_resumes_unrelated_application_panic() {
    let source = Handle::new(IsolineAdaptorFake::new());
    *source.unrelated_panic.write() = true;
    let drawer = Handle::new(Prs3dDrawer::default());
    drawer.setup_own_defaults().unwrap();
    let events = Arc::new(XrRecordingEvents::new());
    let prs = xr_recording_presentation(&events);
    let parameters = derived_isoline_parameters();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sample_derived_isoline_step(&prs, &source, &drawer, &parameters);
    }));
    let payload = outcome.unwrap_err();
    assert_eq!(
        payload.downcast_ref::<&str>(),
        Some(&"Martensite application surface panic")
    );
}
