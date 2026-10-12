//! Standard2D retained interfaces and pinned native buffers replay without any OCCT dependency.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;
macro_rules! texture_asset {
    ($mode:literal) => {
        (
            $mode,
            include_bytes!(concat!(
                "fixtures/surface_textures/native-amd/",
                $mode,
                ".rgba"
            ))
            .as_slice(),
            include_bytes!(concat!(
                "fixtures/surface_textures/native-amd/",
                $mode,
                ".depth"
            ))
            .as_slice(),
        )
    };
}
const NATIVE_TEXTURE_ASSETS: &[(&str, &[u8], &[u8])] = &[
    texture_asset!("nearest"),
    texture_asset!("mask-exact"),
    texture_asset!("bc1-rgb"),
    texture_asset!("bc1-rgba"),
    texture_asset!("bc2"),
    texture_asset!("bc3"),
    texture_asset!("bc1-minified"),
    texture_asset!("bc1-base-level"),
];
#[test]
fn sampled_image_native_assets_replay_on_both_devices_without_occt() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for (mode, rgba, depth) in NATIVE_TEXTURE_ASSETS {
        native_golden::assert_capture(&capture_texture_fixture(&driver, mode), rgba, depth, mode);
    }
    assert_texture_validation(&driver);
}
fn capture_texture_fixture(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = texture_scene::SurfaceTextureScene::with_mode(driver, mode);
    let view = fixture.scene.cview();
    view.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.set_z_layer_redraw_mode(true);
    let render = fixture.scene.render_view();
    render.try_redraw().unwrap();
    fixture.mutate(mode);
    render.try_redraw().unwrap();
    let capture = render.capture().unwrap();
    fixture.scene.view.remove();
    capture
}
#[test]
fn content_revision_and_live_sampler_commands_keep_prior_frames_immutable() {
    for (mode, changes) in [
        ("image-stale", false),
        ("image-updated", true),
        ("sampler-updated", true),
        ("bc1-levels-updated", false),
        ("bc1-filter-levels-updated", true),
    ] {
        assert_texture_snapshot_change(mode, changes);
    }
}
fn assert_texture_snapshot_change(mode: &str, changes: bool) {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let fixture = texture_scene::SurfaceTextureScene::with_mode(&driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    let before = device.last_frame();
    fixture.mutate(mode);
    fixture.scene.render_view().try_redraw().unwrap();
    let after = device.last_frame();
    assert_eq!(before.draws() != after.draws(), changes, "mode={mode}");
    assert_eq!(device.frames.read()[0].draws(), before.draws());
    fixture.scene.view.remove();
}
#[test]
fn repeated_content_uploads_and_sampler_switches_keep_each_prior_submission() {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let fixture = texture_scene::SurfaceTextureScene::with_mode(&driver, "image-updated");
    for filter in [
        Graphic3dTypeOfTextureFilter::Nearest,
        Graphic3dTypeOfTextureFilter::Bilinear,
    ] {
        fixture.mutate("image-updated");
        fixture.texture.get_params().set_filter(filter);
        fixture.scene.render_view().try_redraw().unwrap();
    }
    let frames = device.frames.read();
    assert_ne!(frames[0].draws(), frames[1].draws());
    assert_eq!(frames.len(), 2);
    fixture.scene.view.remove();
}
#[test]
fn every_surface_policy_renders_valid_depth_and_source_format_without_validation_errors() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for mode in texture_scene::modes::SURFACE_TEXTURE_MODES {
        assert_valid_texture_capture(&capture_texture_fixture(&driver, mode), mode);
    }
    assert_texture_validation(&driver);
}
fn assert_valid_texture_capture(capture: &FrameCapture, mode: &str) {
    assert_eq!(capture.rgba.len(), 64 * 64 * 4, "mode={mode}");
    assert!(
        capture
            .depth
            .iter()
            .all(|z| z.is_finite() && (0. ..=1.).contains(z)),
        "mode={mode}"
    );
    let colored = capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .any(|pixel| *pixel != [0, 0, 255, 255]);
    assert!(
        colored || capture.depth.iter().any(|depth| *depth < 1.),
        "mode={mode}"
    );
}
fn assert_texture_validation(driver: &Graphic3dRenderDriver) {
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
