//! Rust-only native alias replay includes derived viewFitAll replacement, failure and dispatch counts.
use super::super::{view_cube_interaction_fixture::*, *};
use super::{view_cube_animation_source_fixture::*, view_cube_fit_reference_fixture::*};
use crate::v3d::V3dTypeOfOrientation;
use rustycad_tkernel::standard::Handle;
const FIT_REFERENCE: &str = include_str!("../../tests/fixtures/view-cube-fit-reference.txt");
#[test]
fn native_virtual_fit_arguments_are_live_noncounting_slots_through_replacement_and_failure() {
    for policy in 0..=5 {
        replay_fit_reference_case(policy);
    }
}
fn replay_fit_reference_case(policy: usize) {
    let scene = WidgetScene::new(false);
    let trace = fit_reference_trace(policy);
    let cube = CubeFitReferenceProbe::new(policy, &trace, &scene.view);
    cube.set_context(&scene.context);
    *trace.cube.write() = cube.cast::<dyn AisViewCubeSource>().downgrade();
    let owners = fit_reference_animations(&trace);
    cube.set_view_animation(&owners[0].cast());
    let owner = AisViewCubeOwner::with_source(&cube.cast(), V3dTypeOfOrientation::Xpos, 5);
    *trace.clicked_owner.write() = owner.downgrade();
    let result = cube.start_animation(&owner);
    record_fit_reference_failure(&trace, policy, result);
    append_fit_reference_final(&cube);
    assert_fit_reference_rows(policy, &trace);
    assert_fit_reference_camera(policy, &cube);
}
fn fit_reference_animations(
    trace: &std::sync::Arc<CubeTransportTrace>,
) -> [Handle<CubeTransportAnimation>; 3] {
    let owners = std::array::from_fn(|id| CubeTransportAnimation::new(trace, id as i32 + 1));
    for owner in &owners {
        owner.set_own_duration(0.5);
    }
    *trace.probes.write() = owners.iter().map(Handle::downgrade).collect();
    owners
}
fn append_fit_reference_final(cube: &CubeFitReferenceProbe) {
    let animation = cube.view_animation_ref().clone_handle();
    let fields = cube.interaction_base().fields.read();
    let actual = [
        cube.trace.slot(),
        animation.is_stopped() as i32,
        animation.target_view_ref().is_null() as i32,
        (animation.camera_start_ref().object_address() == fields.start.object_address()) as i32,
        (animation.camera_end_ref().object_address() == fields.end.object_address()) as i32,
    ];
    cube.trace.rows.write().push(format!(
        "fit-final {} {}",
        cube.policy,
        join_transport_channels(&actual)
    ));
}
fn assert_fit_reference_rows(policy: usize, trace: &CubeTransportTrace) {
    let expected = FIT_REFERENCE
        .lines()
        .filter(|line| {
            let row = line.split_whitespace().collect::<Vec<_>>();
            let id = row[1].parse::<usize>().unwrap();
            (row[0] == "event" && id == 14 + policy)
                || (row[0] != "event" && row[0] != "fit-camera" && id == policy)
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(*trace.rows.read(), expected, "fit policy={policy}");
}
fn assert_fit_reference_camera(policy: usize, cube: &CubeFitReferenceProbe) {
    let row = widget_rows(FIT_REFERENCE, "fit-camera")
        .find(|row| row[0] == policy.to_string())
        .unwrap();
    assert_widget_camera(&cube.interaction_base().fields.read().end, &row);
}

fn record_fit_reference_failure(
    trace: &CubeTransportTrace,
    policy: usize,
    result: Result<(), rustycad_tkernel::standard::Failure>,
) {
    if let Err(failure) = result {
        assert_eq!(failure.exception_type(), "Standard_Failure");
        trace
            .rows
            .write()
            .push(format!("fit-failure {policy} {}", failure.message()));
    }
}
