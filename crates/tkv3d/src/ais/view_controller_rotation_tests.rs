//! Frozen native camera, controller and request state verifies full orbit/view rotation behavior.
use super::view_controller_camera_fixture::PrimitiveScene;
use super::view_controller_rotation_fixture::*;
use rustycad_tkmath::gp::Point3;
const ROTATION_NATIVE_CHUNKS: [&str; 15] = [
    include_str!("../../tests/fixtures/controller-camera-rotation-0-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-1-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-2-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-3-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-4-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-5-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-6-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-7-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-8-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-9-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-10-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-11-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-12-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-13-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-rotation-14-native.txt"),
];

#[test]
fn real_camera_rotations_match_native_retained_starts_roll_limits_and_xr() {
    for chunk in ROTATION_NATIVE_CHUNKS {
        replay_rotation_chunk(chunk);
    }
}
fn replay_rotation_chunk(chunk: &str) {
    let mut scene = rotation_scene(0);
    for line in chunk.lines() {
        replay_rotation_line(&mut scene, line);
    }
}
fn replay_rotation_line(scene: &mut PrimitiveScene, line: &str) {
    let row: Vec<f64> = line
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    let (index, step, group) = (
        row[0] as usize % 10000,
        row[1] as usize,
        row[0] as usize / 10000,
    );
    if step == 0 {
        reset_rotation_scene(scene, index, group);
    }
    prepare_rotation_frame(scene, step);
    let failed = step != 0 && dispatch_rotation_fixture(index, step, group, scene).is_err();
    assert_rotation_snapshot(&rotation_snapshot(scene, failed), &row);
}
fn assert_rotation_snapshot(actual: &[f64], row: &[f64]) {
    assert_eq!(row.len(), 202);
    assert_eq!(actual.len(), 200);
    for (column, (actual, expected)) in actual.iter().zip(&row[2..]).enumerate() {
        assert_rotation_column(row, column, *actual, *expected);
    }
}
fn assert_rotation_column(row: &[f64], column: usize, actual: f64, expected: f64) {
    if !(4..85).contains(&column) || matches!(column, 25 | 47 | 84) {
        assert_eq!(
            actual, expected,
            "case {} frame {} col {column}",
            row[0], row[1]
        );
        return;
    }
    assert!(
        (actual - expected).abs() <= 2e-9 * expected.abs().max(1.0),
        "case {} frame {} col {column}: {actual} != {expected}",
        row[0],
        row[1]
    );
}
#[test]
fn deterministic_retained_rotation_defaults_do_not_read_uninitialized_roll() {
    use super::AisViewController;
    use rustycad_tkmath::gp::{Direction3, Point3};
    let c = AisViewController::new();
    assert_eq!(c.rotation_state.current_roll, 0.0);
    assert_eq!(c.rotation_state.pivot, Point3::default());
    assert_eq!(c.rotation_state.start_up, Direction3::X);
    assert_eq!(c.rotation_state.start_direction, Direction3::X);
}

fn reset_rotation_scene(scene: &mut PrimitiveScene, index: usize, group: usize) {
    *scene = rotation_scene(index);
    if group == 3 {
        set_degenerate_rotation_starts(scene);
    }
    if group == 1 {
        super::view_controller_camera_failure::attach_primitive_stop_failure(scene);
    }
}
fn dispatch_rotation_fixture(
    index: usize,
    step: usize,
    group: usize,
    scene: &mut PrimitiveScene,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    if group == 4 {
        return apply_passive_rotation(scene);
    }
    if group == 2 {
        return apply_mixed_rotation(index, step, scene);
    }
    apply_rotation(index, scene)
}
fn apply_mixed_rotation(
    index: usize,
    step: usize,
    scene: &mut PrimitiveScene,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    if step == 2 {
        return scene.controller.handle_view_rotation(
            &scene.view,
            0.15,
            -0.25,
            -0.14,
            !index.is_multiple_of(2),
        );
    }
    scene.controller.handle_orbit_rotation(
        &scene.view,
        &Point3::new(9.0, -5.0, 12.0),
        rotation_options(index).is_multiple_of(2),
    )
}
#[test]
fn actual_rotation_failures_preserve_pre_stop_state_and_retry_camera() {
    let fixture =
        include_str!("../../tests/fixtures/controller-camera-rotation-failure-native.txt");
    replay_rotation_chunk(fixture);
    assert_eq!(fixture.lines().count(), 64);
}
#[test]
fn orbit_view_orbit_calls_share_real_retained_rotation_vectors() {
    let fixture = include_str!("../../tests/fixtures/controller-camera-rotation-mixed-native.txt");
    replay_rotation_chunk(fixture);
    assert_eq!(fixture.lines().count(), 80);
}

fn set_degenerate_rotation_starts(scene: &mut PrimitiveScene) {
    use rustycad_tkmath::gp::Direction3;
    let c = &mut scene.controller;
    c.rotation_state.start_up = Direction3::X;
    c.rotation_state.start_direction = Direction3::X;
    c.render_input.orbit_rotation.to_start = false;
    c.render_input.view_rotation.to_start = false;
    c.render_input.orbit_rotation.to_rotate = true;
    c.render_input.view_rotation.to_rotate = true;
}
#[test]
fn degenerate_retained_axes_preserve_native_partial_camera_and_roll_failures() {
    let fixture =
        include_str!("../../tests/fixtures/controller-camera-rotation-degenerate-native.txt");
    replay_rotation_chunk(fixture);
    assert_eq!(fixture.lines().count(), 32);
}

fn apply_passive_rotation(
    scene: &mut PrimitiveScene,
) -> Result<(), rustycad_tkernel::standard::Failure> {
    let roll = scene.controller.rotation_state.current_roll;
    scene
        .controller
        .handle_view_rotation(&scene.view, 0.0, 0.0, roll, false)
}
#[test]
fn matching_roll_preserves_native_empty_and_start_only_view_request_guards() {
    let fixture =
        include_str!("../../tests/fixtures/controller-camera-rotation-passive-native.txt");
    replay_rotation_chunk(fixture);
    assert_eq!(fixture.lines().count(), 48);
}
