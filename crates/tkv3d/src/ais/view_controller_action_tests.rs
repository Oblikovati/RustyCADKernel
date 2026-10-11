//! Actual scene, presentation and request snapshots replay without OCCT.
use super::view_controller_action_scene_fixture::*;
use super::view_controller_action_seed_fixture::*;
use super::view_controller_action_snapshot_fixture::action_snapshot;
use super::{AisAnimationMembers, AisAnimationSource};
const ACTION_NATIVE_CHUNKS: [&str; 24] = [
    include_str!("../../tests/fixtures/controller-camera-actions-0-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-1-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-2-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-3-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-4-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-5-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-6-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-7-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-8-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-9-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-10-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-11-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-12-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-13-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-14-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-15-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-16-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-17-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-18-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-19-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-20-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-21-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-22-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-actions-23-native.txt"),
];
#[test]
fn complete_camera_actions_match_native_scene_requests_and_xr() {
    for chunk in ACTION_NATIVE_CHUNKS {
        replay_action_chunk(chunk);
    }
}
fn replay_action_chunk(chunk: &str) {
    let mut scene = action_scene(0);
    for line in chunk.lines() {
        replay_action_line(&mut scene, line);
    }
}
fn replay_action_line(scene: &mut ActionScene, line: &str) {
    let row: Vec<f64> = line
        .split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect();
    let actual = replay_action_frame(scene, &row);
    assert_eq!(actual.len(), row.len() - 2);
    for (column, (actual, expected)) in actual.iter().zip(&row[2..]).enumerate() {
        assert_action_column(&row, column, *actual, *expected);
    }
}
fn replay_action_frame(scene: &mut ActionScene, row: &[f64]) -> Vec<f64> {
    let (index, frame, group) = (
        row[0] as usize % 10000,
        row[1] as usize,
        row[0] as usize / 10000,
    );
    if frame == 0 {
        *scene = action_scene(index);
        seed_action_scene(scene, index);
        seed_action_failure(scene, group);
    }
    reset_action_redraw(scene);
    let result = if frame == 0 {
        0
    } else {
        dispatch_action_fixture(scene, index, frame, group)
    };
    action_snapshot(scene, result)
}

fn assert_action_column(row: &[f64], column: usize, actual: f64, expected: f64) {
    // Native DumpJson emits grid-echo float coordinates with six significant digits.
    let tolerance = if (85..88).contains(&column) {
        2e-6
    } else {
        2e-9
    };
    assert!(
        (actual - expected).abs() <= tolerance * expected.abs().max(1.0),
        "case {} frame {} col {column}: {actual} != {expected}",
        row[0],
        row[1]
    );
}

fn dispatch_action_fixture(
    scene: &mut ActionScene,
    index: usize,
    frame: usize,
    group: usize,
) -> i32 {
    if matches!(group, 3 | 5) {
        return dispatch_action_lazy(scene, frame, group);
    }
    if group == 4 && frame == 2 {
        scene.controller.view_animation().clear();
    }
    configure_action_callback_failure(scene, frame, group);
    capture_action_callback_result(scene, index)
}
fn configure_action_callback_failure(scene: &ActionScene, frame: usize, group: usize) {
    use std::sync::atomic::Ordering;
    scene.callback.fail_kind.store(
        if frame == 1 && group < 3 {
            group as i32
        } else {
            0
        },
        Ordering::Relaxed,
    );
}
fn dispatch_action_lazy(scene: &mut ActionScene, frame: usize, group: usize) -> i32 {
    use rustycad_tkernel::ncollection::Vec2;
    if group == 3 {
        scene.controller.camera_action_state.had_computed_hlr = frame == 2;
    }
    let pixel = if frame <= 2 {
        Vec2::new(320, 240)
    } else {
        Vec2::new(137, 91)
    };
    if group == 5 && frame == 1 {
        return dispatch_action_wrong_context(scene, &pixel);
    }
    i32::from(
        scene
            .controller
            .context_lazy_move_to(&scene.context, &scene.view, &pixel)
            .is_err(),
    )
}
fn capture_action_callback_result(scene: &mut ActionScene, index: usize) -> i32 {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scene.controller.handle_camera_actions(
            &scene.context,
            &scene.view,
            &action_walk(index % 24),
        )
    }));
    match result {
        Ok(result) => i32::from(result.is_err()),
        Err(payload) => {
            assert_eq!(*payload.downcast::<u32>().unwrap(), 73);
            2
        }
    }
}

#[test]
fn typed_callback_failures_preserve_full_scene_before_retry() {
    replay_action_chunk(include_str!(
        "../../tests/fixtures/controller-camera-actions-typed-failure-native.txt"
    ));
}
#[test]
fn unrelated_callback_panics_escape_the_fallible_boundary() {
    replay_action_chunk(include_str!(
        "../../tests/fixtures/controller-camera-actions-unrelated-panic-native.txt"
    ));
}
#[test]
fn lazy_detection_matches_real_grid_echo_cached_cursor_hlr_and_active_views() {
    replay_action_chunk(include_str!(
        "../../tests/fixtures/controller-camera-actions-lazy-native.txt"
    ));
}

fn seed_action_failure(scene: &ActionScene, group: usize) {
    use super::view_controller_camera_failure::PrimitiveStopFailure;
    use rustycad_tkernel::standard::Handle;
    if group != 4 {
        return;
    }
    scene
        .controller
        .view_animation()
        .add(&Handle::new(PrimitiveStopFailure::new()).cast())
        .unwrap();
    scene.controller.view_animation().start(false).unwrap();
}
#[test]
fn actual_child_stop_failure_retains_full_zoom_queue_and_partial_scene_before_retry() {
    replay_action_chunk(include_str!(
        "../../tests/fixtures/controller-camera-actions-animation-failure-native.txt"
    ));
}

fn dispatch_action_wrong_context(
    scene: &mut ActionScene,
    pixel: &rustycad_tkernel::ncollection::Vec2<i32>,
) -> i32 {
    let other = action_scene(0);
    i32::from(
        scene
            .controller
            .context_lazy_move_to(&other.context, &scene.view, pixel)
            .is_err(),
    )
}
#[test]
fn lazy_wrong_viewer_failure_retains_temporary_depth_and_cursor_before_retry_guard() {
    replay_action_chunk(include_str!(
        "../../tests/fixtures/controller-camera-actions-lazy-failure-native.txt"
    ));
}
