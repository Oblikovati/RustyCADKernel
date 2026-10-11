//! Replay actual selected geometry fitting and camera toggles without OCCT at test time.
use super::AisViewController;
use super::view_controller_fit_fixture::*;
#[test]
fn selection_bounds_and_camera_fit_match_native_global_and_part_owner_scenes() {
    for fixture in fit_native_chunks() {
        replay_fit_chunk(fixture);
    }
}
fn replay_fit_chunk(fixture: &str) {
    let mut scene = fit_scene(0);
    let mut controller = AisViewController::new();
    for line in fixture.lines() {
        let row: Vec<f64> = line
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        if row[1] == 0.0 {
            scene = fit_scene(row[0] as usize);
            controller = AisViewController::new();
        }
        apply_fit_frame(&mut controller, &scene, row[1] as usize);
        assert_fit_row(&scene, &row);
    }
}
fn apply_fit_frame(c: &mut AisViewController, s: &NativeFitScene, step: usize) {
    match step {
        0 => {}
        1..=3 => c.fit_all_auto(&s.context, &s.view).unwrap(),
        4 => s.context.fit_selected_default(&s.view).unwrap(),
        5 => s.context.fit_selected(&s.view, 0.05, false).unwrap(),
        other => panic!("fit frame {other}; expected 0..=5"),
    }
}
fn assert_fit_row(s: &NativeFitScene, row: &[f64]) {
    let actual = fit_snapshot(s);
    assert_eq!(actual.len(), row.len() - 2);
    for (column, (actual, expected)) in actual.iter().zip(&row[2..]).enumerate() {
        let tolerance = 1e-9 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() < tolerance,
            "case {} frame {} col {column}: {actual} != {expected}",
            row[0],
            row[1]
        );
    }
}
fn fit_native_chunks() -> [&'static str; 8] {
    [
        include_str!("../../tests/fixtures/controller-fit-all-0-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-1-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-2-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-3-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-4-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-5-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-6-native.txt"),
        include_str!("../../tests/fixtures/controller-fit-all-7-native.txt"),
    ]
}
#[test]
fn selection_bounds_preserve_the_public_selected_cursor() {
    let scene = fit_scene(5);
    scene.context.init_selected();
    scene.context.next_selected();
    let before = scene.context.selected_owner().unwrap();
    scene
        .context
        .bounding_box_of_selection(Some(&scene.view))
        .unwrap();
    scene.context.bounding_box_of_selection(None).unwrap();
    assert_eq!(scene.context.selected_owner().unwrap(), before);
    assert!(scene.context.more_selected());
}
#[test]
fn void_selection_bypasses_margin_validation_and_preserves_camera() {
    let scene = fit_scene(0);
    let before = fit_snapshot(&scene);
    scene.context.fit_selected(&scene.view, -1.0, true).unwrap();
    scene.context.fit_selected(&scene.view, 1.0, false).unwrap();
    assert_eq!(fit_snapshot(&scene), before);
}
#[test]
fn selected_bounds_validate_margin_before_changing_camera() {
    let scene = fit_scene(1);
    let before = fit_snapshot(&scene);
    for margin in [-1.0, 1.0] {
        assert!(
            scene
                .context
                .fit_selected(&scene.view, margin, true)
                .is_err()
        );
        assert_eq!(fit_snapshot(&scene), before);
    }
}
#[test]
fn fitting_toggles_selection_and_scene_without_replacing_camera() {
    let scene = fit_scene(1);
    let camera = scene.view.camera();
    let mut controller = AisViewController::new();
    controller
        .fit_all_auto(&scene.context, &scene.view)
        .unwrap();
    let selected_center = camera.center();
    controller
        .fit_all_auto(&scene.context, &scene.view)
        .unwrap();
    assert_ne!(camera.center(), selected_center);
    controller
        .fit_all_auto(&scene.context, &scene.view)
        .unwrap();
    assert_eq!(camera.center(), selected_center);
    assert_eq!(camera, scene.view.camera());
}
