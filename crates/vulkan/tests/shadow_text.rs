//! Native Shadow glyph assets replay through Rust alone; broader strict failures stay in receipts.
#[path = "../examples/support/capture_request.rs"]
mod capture_request;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/text_scene.rs"]
mod text_scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
macro_rules! shadow_asset {
    ($profile:literal, $mode:literal) => {
        (
            concat!("text-shadow-", $mode),
            include_bytes!(concat!(
                "fixtures/shadow-text/",
                $profile,
                "/",
                $mode,
                ".rgba"
            )) as &[u8],
            include_bytes!(concat!(
                "fixtures/shadow-text/",
                $profile,
                "/",
                $mode,
                ".depth"
            )) as &[u8],
        )
    };
}
macro_rules! shadow_assets {
    ($profile:literal) => {
        [
            shadow_asset!($profile, "plane"),
            shadow_asset!($profile, "plane-local"),
            shadow_asset!($profile, "opaque"),
        ]
    };
}
#[test]
fn shadow_native_buffers_replay_without_occt_and_all_modes_render() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for mode in text_scene::SHADOW_MODES {
        assert!(!capture_shadow(&driver, mode).rgba.is_empty());
    }
    replay_shadow_assets(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn replay_shadow_assets(driver: &Handle<Graphic3dRenderDriver>) {
    let assets = if driver.device_name().contains("llvmpipe") {
        shadow_assets!("lavapipe")
    } else {
        shadow_assets!("host")
    };
    for (mode, rgba, depth) in assets {
        native_golden::assert_capture(&capture_shadow(driver, mode), rgba, depth, mode);
    }
}
fn capture_shadow(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let text = text_scene::TextScene::new(driver);
    configure_shadow_capture(&text, mode);
    text.apply(mode);
    text.scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    text.scene.cview().set_z_layer_redraw_mode(true);
    text.scene.render_view().try_redraw().unwrap();
    let capture = text.scene.render_view().capture().unwrap();
    text.scene.view.remove();
    capture
}

fn configure_shadow_capture(text: &text_scene::TextScene, mode: &str) {
    let args: Vec<String> = ["capture", "96", "48", "0", "2", "unused", mode]
        .into_iter()
        .map(String::from)
        .collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, mode).unwrap();
    request.configure(&text.scene.window, &text.scene.view);
    text.scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(Transform3::translation_vector(
            Vector3::new(2., 0., 0.),
        ))));
}
