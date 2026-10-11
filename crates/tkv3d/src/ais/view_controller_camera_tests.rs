//! Pinned native replay verifies actual camera/animation/XR state and every UI/GL buffer field.
use super::view_controller_camera_fixture::*;
const PRIMITIVE_NATIVE_CHUNKS: [&str; 9] = [
    include_str!("../../tests/fixtures/controller-camera-primitives-0-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-1-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-2-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-3-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-4-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-5-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-6-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-7-native.txt"),
    include_str!("../../tests/fixtures/controller-camera-primitives-8-native.txt"),
];
#[test]
fn actual_camera_primitives_match_native_pan_rotation_zoom_focus_and_xr() {
    for chunk in PRIMITIVE_NATIVE_CHUNKS {
        replay_primitive_chunk(chunk);
    }
}
fn replay_primitive_chunk(chunk: &str) {
    let mut scene = primitive_scene(0);
    for line in chunk.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            scene = primitive_scene_from_row(&row);
        }
        scene.view.get().unwrap().view().redraw();
        if row[1] != 0.0 {
            apply_primitive((row[0] as usize) % 10000, &mut scene);
        }
        assert_primitive_row(&scene, &row);
    }
}
fn assert_primitive_row(scene: &PrimitiveScene, row: &[f64]) {
    let actual = primitive_snapshot(scene);
    assert_eq!(row.len(), 166);
    assert_eq!(actual.len(), row.len() - 2);
    for (column, (actual, expected)) in actual.iter().zip(&row[2..]).enumerate() {
        assert_primitive_column(row, column, *actual, *expected);
    }
}

fn assert_primitive_column(row: &[f64], column: usize, actual: f64, expected: f64) {
    if column >= 50 {
        assert_eq!(
            actual, expected,
            "case {} frame {} queue column {column}",
            row[0], row[1]
        );
        return;
    }
    let floor = if matches!(column, 19 | 20 | 41 | 42) {
        1e-12
    } else {
        1.0
    };
    let tolerance = 2e-9 * expected.abs().max(floor);
    assert!(
        (actual - expected).abs() <= tolerance,
        "case {} frame {} col {column}: {actual} != {expected}",
        row[0],
        row[1]
    );
}

fn primitive_scene_from_row(row: &[f64]) -> PrimitiveScene {
    let index = row[0] as usize;
    let scene = primitive_scene(index % 10000);
    if index >= 10000 {
        scene
            .view
            .camera()
            .set_scale(if index < 20000 { 1e-9 } else { 1e14 });
    }
    scene
}
#[test]
fn cursor_zoom_clamps_real_camera_dimensions_at_native_scale_limits() {
    let fixture =
        include_str!("../../tests/fixtures/controller-camera-primitives-limits-native.txt");
    replay_primitive_chunk(fixture);
    assert_eq!(fixture.lines().count(), 150);
}
#[test]
fn native_animation_stop_rejection_precedes_pan_rotation_zoom_but_not_focus() {
    let fixture =
        include_str!("../../tests/fixtures/controller-camera-primitives-failure-native.txt");
    let mut scene = primitive_scene(0);
    for line in fixture.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        replay_primitive_failure_frame(&mut scene, &row);
        assert_primitive_row(&scene, &row);
    }
}
fn replay_primitive_failure_frame(scene: &mut PrimitiveScene, row: &[f64]) {
    let index = row[0] as usize;
    if row[1] == 0.0 {
        *scene = primitive_scene(index);
        super::view_controller_camera_failure::attach_primitive_stop_failure(scene);
        scene.view.get().unwrap().view().redraw();
        return;
    }
    let result = apply_primitive_result(index, scene);
    assert_eq!(result.is_err(), row[1] == 1.0 && index / 2400 < 3);
}
#[test]
fn panning_anchor_matches_native_infinite_x_sentinel_and_retains_other_coordinates() {
    let mut controller = super::AisViewController::new().unwrap();
    assert!(!controller.has_panning_anchor_point());
    for (x, defined) in [
        (0.0, true),
        (1e100, false),
        (-1e100, false),
        (f64::NAN, true),
    ] {
        let point = rustycad_tkmath::gp::Point3::new(x, f64::INFINITY, -17.0);
        controller.set_panning_anchor_point(&point);
        assert_eq!(controller.has_panning_anchor_point(), defined);
        assert_eq!(controller.panning_anchor_point().y(), f64::INFINITY);
        assert_eq!(controller.panning_anchor_point().z(), -17.0);
    }
}
