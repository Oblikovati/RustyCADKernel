//! Real concrete ColoredShape output replays pinned native buffers without OCCT.
#[path = "support/ais_colored_shape_scene.rs"]
mod colored_scene;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/ais_colored_shape_native_assets.rs"]
mod native_assets;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

#[test]
fn concrete_colored_shape_replays_all_native_appearance_and_update_workloads() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for row in native_assets::COLORED_NATIVE_BUFFERS {
        let capture = capture_colored_fixture(&driver, row.mode);
        let (rgba, depth) = colored_native_profile(&driver, row);
        native_golden::assert_capture(&capture, rgba, depth, row.mode);
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn capture_colored_fixture(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = colored_scene::AisColoredShapeScene::with_mode(driver, mode).unwrap();
    fixture.scene.window.set_size(128, 128);
    fixture.scene.view.must_be_resized();
    let view = fixture.scene.cview();
    view.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.set_z_layer_redraw_mode(true);
    let render = fixture.scene.render_view();
    render.try_redraw().unwrap();
    fixture.mutate(mode).unwrap();
    render.try_redraw().unwrap();
    let capture = render.capture().unwrap();
    fixture.scene.view.remove();
    capture
}
fn colored_native_profile<'a>(
    driver: &Graphic3dRenderDriver,
    row: &'a native_assets::ColoredNativeBuffers,
) -> (&'a [u8], &'a [u8]) {
    if driver.device_name().contains("llvmpipe") {
        return row.software;
    }
    assert!(
        driver.device_name().contains("AMD"),
        "received device={}; expected proved AMD or llvmpipe profile",
        driver.device_name()
    );
    row.host
}
