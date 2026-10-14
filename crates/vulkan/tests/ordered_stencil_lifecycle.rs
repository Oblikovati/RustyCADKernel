//! Actual retained views and real submissions preserve ordered stencil through lifecycle/failure.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/stencil_lifecycle_device.rs"]
mod lifecycle_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
fn stencil_lifecycle_driver(failure: &Arc<AtomicBool>) -> Handle<Graphic3dRenderDriver> {
    let backend = lifecycle_device::StencilLifecycleDevice::new(
        device_options::validation_options(),
        Arc::clone(failure),
    );
    Graphic3dRenderDriver::with_backend(backend, 128).unwrap()
}
fn assert_lifecycle_state(scene: &scene::ViewerScene, enabled: bool, comparison: u32) {
    let state = scene.render_view().stencil_context_state();
    assert_eq!(state.is_enabled(), enabled);
    let expected = expected_lifecycle_fields(enabled, comparison);
    assert_eq!(state.native_state_fields(), expected);
}
#[test]
fn actual_clear_remove_and_group_destruction_do_not_reset_view_stencil() {
    let driver = stencil_lifecycle_driver(&Arc::new(AtomicBool::new(false)));
    let scene = scene::ViewerScene::new(&driver);
    scene.group.set_stencil_test_options(true);
    scene.render_view().try_redraw().unwrap();
    assert_lifecycle_state(&scene, true, 0x205);
    scene.group.clear(false);
    assert_lifecycle_state(&scene, true, 0x205);
    scene.render_view().try_redraw().unwrap();
    scene.group.remove();
    scene.render_view().try_redraw().unwrap();
    scene.structure.erase();
    scene.structure.remove();
    assert_lifecycle_state(&scene, true, 0x205);
    scene.view.remove();
}
#[test]
fn original_view_failure_repair_and_adjacent_replacement_views_are_independent() {
    let failure = Arc::new(AtomicBool::new(false));
    let driver = stencil_lifecycle_driver(&failure);
    let first = scene::ViewerScene::new(&driver);
    first.group.set_stencil_test_options(true);
    first.render_view().try_redraw().unwrap();
    assert_lifecycle_center(&first, true);
    first.render_view().try_redraw().unwrap();
    assert_lifecycle_center(&first, false);
    assert_submission_failure_repair(&first, &failure);
    assert_adjacent_stencil_lifetimes(&first, &driver);
}
fn assert_submission_failure_repair(first: &scene::ViewerScene, failure: &AtomicBool) {
    first.group.set_stencil_test_options(false);
    failure.store(true, Ordering::Relaxed);
    assert!(first.render_view().try_redraw().is_err());
    assert!(first.render_view().capture().is_err());
    assert_lifecycle_state(first, true, 0x205);
    failure.store(false, Ordering::Relaxed);
    first.render_view().try_redraw().unwrap();
    assert_lifecycle_state(first, false, 0x205);
    assert_lifecycle_center(first, false);
    first.render_view().try_redraw().unwrap();
    assert_lifecycle_center(first, true);
}
fn assert_adjacent_stencil_lifetimes(
    first: &scene::ViewerScene,
    driver: &Handle<Graphic3dRenderDriver>,
) {
    let adjacent = scene::ViewerScene::new(driver);
    adjacent.render_view().try_redraw().unwrap();
    assert_lifecycle_state(&adjacent, false, 0x207);
    assert_lifecycle_center(&adjacent, true);
    first.group.set_stencil_test_options(true);
    first.render_view().try_redraw().unwrap();
    adjacent.render_view().try_redraw().unwrap();
    assert_lifecycle_state(first, true, 0x205);
    assert_lifecycle_state(&adjacent, false, 0x207);
    assert_destroyed_stencil_view_replacement(first, &adjacent, driver);
}

fn assert_destroyed_stencil_view_replacement(
    first: &scene::ViewerScene,
    adjacent: &scene::ViewerScene,
    driver: &Handle<Graphic3dRenderDriver>,
) {
    first.view.remove();
    adjacent.render_view().try_redraw().unwrap();
    assert_lifecycle_state(adjacent, false, 0x207);
    let replacement = scene::ViewerScene::new(driver);
    replacement.render_view().try_redraw().unwrap();
    assert_lifecycle_state(&replacement, false, 0x207);
    replacement.view.remove();
    adjacent.view.remove();
}

fn assert_lifecycle_center(scene: &scene::ViewerScene, visible: bool) {
    let image = scene.render_view().capture().unwrap();
    let color = if visible {
        [255, 0, 0, 255]
    } else {
        [0, 0, 255, 255]
    };
    assert_eq!(image.pixel(32, 32), Some(color));
}
#[test]
fn attributed_native_lifecycle_rows_preserve_every_context_field() {
    let fixture = include_str!("fixtures/ordered_stencil/native-lifecycle.txt");
    for line in fixture.lines() {
        let mut fields = line.split_whitespace();
        let stage = fields.next().unwrap();
        let values: Vec<u32> = fields.map(|value| value.parse().unwrap()).collect();
        let enabled = !stage.starts_with("adjacent") && stage != "replacement-first";
        let expected = expected_lifecycle_fields(enabled, if enabled { 0x205 } else { 0x207 });
        assert_eq!(values, expected, "{stage}");
    }
}

fn expected_lifecycle_fields(enabled: bool, comparison: u32) -> [u32; 8] {
    [
        u32::from(enabled),
        comparison,
        u32::from(comparison == 0x205),
        255,
        255,
        0x1E00,
        0x1E00,
        0x1E00,
    ]
}
