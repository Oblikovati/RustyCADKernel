//! Native callbacks exercise current-main actual camera fields; no Frame ancestry is required.
use super::super::{
    AisViewController,
    view_controller_fit_fixture::{NativeFitScene, fit_scene, fit_snapshot},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Direction3;
use rustycad_tkservice::graphic3d::Graphic3dCamera;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::atomic::Ordering,
};
#[test]
fn native_fit_all_camera_alias_survives_bounds_callback_and_failure_retry() {
    let expected: Vec<Vec<f64>> =
        include_str!("../../tests/fixtures/controller-fit-camera-slot-native.txt")
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .map(|column| column.parse().unwrap())
                    .collect()
            })
            .collect();
    for case in 0..6 {
        replay_fit_camera_slot(case, &expected);
    }
}
fn replay_fit_camera_slot(case: i32, expected: &[Vec<f64>]) {
    let scene = fit_scene((case / 3 * 256 + 1) as usize);
    let original = scene.view.camera();
    let replacement = selected_camera_slot_replacement(&scene);
    let control = fit_camera_slot_control(case % 3, &scene.view, &replacement);
    let _window = attach_fit_camera_slot_window(&scene.view, &control);
    control.armed.store(true, Ordering::Relaxed);
    let mut controller = AisViewController::new().unwrap();
    for frame in 0..2 {
        let failure = apply_fit_camera_slot(&mut controller, &scene);
        let row = fit_camera_slot_snapshot(
            [case, frame, failure],
            &scene,
            &control.records.read(),
            (&original, &replacement),
        );
        assert_fit_camera_slot_row(&row, &expected[(case * 2 + frame) as usize]);
    }
}
fn apply_fit_camera_slot(controller: &mut AisViewController, scene: &NativeFitScene) -> i32 {
    let result = catch_unwind(AssertUnwindSafe(|| controller.fit_all_auto(&scene.context, &scene.view)));
    fit_camera_slot_callback_failure(result)
}
fn selected_camera_slot_replacement(scene: &NativeFitScene) -> Handle<Graphic3dCamera> {
    let camera = Handle::new(scene.view.camera().copy_members());
    let selected = scene
        .context
        .bounding_box_of_selection(Some(&scene.view))
        .unwrap();
    scene
        .view
        .fit_min_max(&camera, &selected, 0.01, 0., true)
        .unwrap();
    camera.set_up(&Direction3::Y);
    camera
}
fn fit_camera_slot_snapshot(
    prefix: [i32; 3],
    scene: &NativeFitScene,
    records: &[i32; 3],
    cameras: (&Handle<Graphic3dCamera>, &Handle<Graphic3dCamera>),
) -> Vec<f64> {
    let mut row = prefix.map(f64::from).to_vec();
    row.extend(records.map(f64::from));
    row.extend([
        cameras.0.ref_count() as f64,
        cameras.1.ref_count() as f64,
        prefix[0] as f64,
        prefix[1] as f64,
    ]);
    row.extend(fit_snapshot(scene));
    row
}
fn assert_fit_camera_slot_row(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (column, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        if column <= 11 || matches!(column, 18 | 25 | 47) {
            assert_eq!(actual, expected, "column {column}");
            continue;
        }
        assert!(
            (actual - expected).abs() < 1e-9 * expected.abs().max(1.),
            "column {column}: actual {actual} expected {expected}"
        );
    }
}

#[path = "view_controller_fit_camera_slot_fixture.rs"]
mod fixture;
use fixture::*;
