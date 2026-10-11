//! Native-style AIS mutations run against the selected Vulkan device, with validation enabled.
use crate::view_setup;
#[path = "ais_lifecycle.rs"]
pub(super) mod ais_lifecycle;
#[path = "../../tests/support/ais_scene.rs"]
pub(super) mod scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkv3d::prs_mgr::PrsMgrDisplayStatus;
use std::sync::atomic::Ordering;
pub fn check_ais_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = scene::AisViewerScene::new(&driver);
    assert_ais_center(&scene, [255, 0, 0, 255]);
    check_lifecycle_transitions(&scene);
    scene.window.set_size(96, 48);
    scene.view.must_be_resized();
    assert_ais_center(&scene, [255, 0, 0, 255]);
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert_eq!(scene.render_view().capture().unwrap().width, 96);
    assert_eq!(scene.object.computations.load(Ordering::Relaxed), 2);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
}
fn check_lifecycle_transitions(scene: &scene::AisViewerScene) {
    for (mode, color) in [
        ("ais-displayed", [255, 0, 0, 255]),
        ("ais-erased", [0, 0, 255, 255]),
        ("ais-redisplayed", [255, 0, 0, 255]),
        ("ais-mode1", [0, 255, 0, 255]),
        ("ais-affinity-off", [0, 0, 255, 255]),
    ] {
        ais_lifecycle::apply_capture_transition(scene, mode).unwrap();
        assert_ais_center(scene, color);
    }
    assert!(ais_lifecycle::apply_capture_transition(scene, "invalid").is_err());
    scene
        .context
        .set_view_affinity(&scene.object.cast(), &scene.view, true)
        .unwrap();
    scene
        .context
        .display_with_modes(&scene.object.cast(), 0, 0, true, PrsMgrDisplayStatus::None)
        .unwrap();
    assert_ais_center(scene, [255, 0, 0, 255]);
}
fn assert_ais_center(scene: &scene::AisViewerScene, color: [u8; 4]) {
    scene.view.redraw();
    let capture = scene.render_view().capture().unwrap();
    assert_eq!(
        capture.pixel(capture.width / 2, capture.height / 2),
        Some(color)
    );
}
