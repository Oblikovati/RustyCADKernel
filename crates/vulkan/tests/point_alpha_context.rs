//! The real view context retains native marker alpha state across opaque frames.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[path = "support/point_alpha_arrays.rs"]
mod point_alpha_arrays;
#[path = "../examples/support/point_alpha_goldens.rs"]
mod point_alpha_goldens;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
mod viewer_runtime {
    pub(super) use super::scene;
}
#[test]
fn retained_mask_then_opaque_sprite_reset_matches_native_camera_persistence() {
    let driver =
        martensite_vulkan::Graphic3dRenderDriver::new(device_options::validation_options())
            .unwrap();
    point_alpha_goldens::check_point_goldens(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
