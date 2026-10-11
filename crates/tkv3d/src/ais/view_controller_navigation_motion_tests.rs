//! Frozen native snapshots prove complete walking motion and retained request state.
use super::view_controller_navigation_motion_fixture::*;
const NAVIGATION_NATIVE_CHUNKS: [&str; 18] = [
    include_str!("../../tests/fixtures/controller-navigation-motion-0-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-1-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-2-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-3-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-4-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-5-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-6-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-7-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-8-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-9-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-10-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-11-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-12-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-13-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-14-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-15-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-16-native.txt"),
    include_str!("../../tests/fixtures/controller-navigation-motion-17-native.txt"),
];

#[test]
fn camera_navigation_matches_native_modes_policies_keys_and_active_xr() {
    for chunk in NAVIGATION_NATIVE_CHUNKS {
        replay_navigation_chunk(chunk);
    }
}
fn replay_navigation_chunk(chunk: &str) {
    let mut scene = navigation_scene(0);
    for line in chunk.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        let (index, step) = (row[0] as usize, row[1] as usize);
        if step == 0 {
            scene = navigation_scene(index);
        }
        let walk = apply_navigation_frame(&mut scene, index, step);
        assert_navigation_snapshot(&navigation_snapshot(&scene, &walk), &row);
    }
}
fn assert_navigation_snapshot(actual: &[f64], row: &[f64]) {
    assert_eq!(row.len(), 244);
    assert_eq!(actual.len(), 242);
    for (column, (actual, expected)) in actual.iter().zip(&row[2..]).enumerate() {
        assert_navigation_column(row, column, *actual, *expected);
    }
}
fn assert_navigation_column(row: &[f64], column: usize, actual: f64, expected: f64) {
    if !(28..93).contains(&column) {
        assert_eq!(
            actual, expected,
            "case {} step {} col {column}",
            row[0], row[1]
        );
        return;
    }
    let tolerance = 2e-9 * expected.abs().max(1e-12);
    assert!(
        (actual - expected).abs() <= tolerance,
        "case {} step {} col {column}: {actual} != {expected}",
        row[0],
        row[1]
    );
}
#[test]
fn empty_navigation_defers_view_validation_and_jump_preserves_failure_state() {
    use super::view_controller_keyboard_fixture::prepare;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic};
    let (mut c, _) = prepare(1.0, 2.0);
    assert!(
        c.handle_navigation_keys(&Handle::default(), &Handle::default())
            .unwrap()
            .is_empty()
    );
    c.key_down(AspectVKeyBasic::NavJump as AspectVKey, 1.25, 1.0)
        .unwrap();
    assert!(
        c.handle_navigation_keys(&Handle::default(), &Handle::default())
            .is_err()
    );
    assert!(c.to_ask_next_frame());
    assert_navigation_jump_bookkeeping(&c);
}
#[test]
fn navigation_zoom_propagates_stop_failure_after_fetch_and_retries_actual_camera() {
    let fixture =
        include_str!("../../tests/fixtures/controller-navigation-motion-failure-native.txt");
    let mut scene = navigation_scene(0);
    for line in fixture.lines() {
        replay_navigation_failure_line(&mut scene, line);
    }
    assert_eq!(fixture.lines().count(), 21);
}
fn replay_navigation_failure_line(scene: &mut NavigationScene, line: &str) {
    use super::AisWalkDelta;
    use rustycad_tkernel::standard::Handle;
    let row: Vec<f64> = line
        .split_whitespace()
        .map(|v| v.parse().unwrap())
        .collect();
    let (index, step) = (row[0] as usize - 10000, row[1] as usize);
    reset_navigation_failure_scene(scene, index, step);
    prepare_navigation_frame(scene, step);
    let result = scene
        .controller
        .handle_navigation_keys(&Handle::default(), &scene.view);
    assert_eq!(result.is_err(), step == 0 && matches!(index, 0 | 1440));
    assert_navigation_snapshot(
        &navigation_snapshot(scene, &result.unwrap_or_else(|_| AisWalkDelta::new())),
        &row,
    );
}

fn reset_navigation_failure_scene(scene: &mut NavigationScene, index: usize, step: usize) {
    use super::view_controller_camera_failure::PrimitiveStopFailure;
    use super::{AisAnimationMembers as _, AisAnimationSource};
    use rustycad_tkernel::standard::Handle;
    if step != 0 {
        return;
    }
    *scene = navigation_scene(index);
    let child: Handle<dyn AisAnimationSource> = Handle::new(PrimitiveStopFailure::new()).cast();
    scene.controller.view_animation().add(&child).unwrap();
    scene.controller.view_animation().start(false).unwrap();
}

fn assert_navigation_jump_bookkeeping(c: &super::AisViewController) {
    use rustycad_tkservice::aspect::{AspectVKey, AspectVKeyBasic};
    assert_eq!(c.navigation_state.last_events_time, 2.0);
    assert!(
        !c.keys()
            .is_key_down(AspectVKeyBasic::NavJump as AspectVKey)
            .unwrap()
    );
}
