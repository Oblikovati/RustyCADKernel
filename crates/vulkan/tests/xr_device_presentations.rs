//! Actual retained AIS XR presentations replay native assets without an OCCT dependency.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
#[allow(dead_code)]
#[path = "support/xr_device_scene.rs"]
mod xr_scene;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers;
macro_rules! xr_asset {
    ($mode:literal) => {
        (
            $mode,
            include_bytes!(concat!("fixtures/xr_device/host/", $mode, ".rgba")).as_slice(),
            include_bytes!(concat!("fixtures/xr_device/host/", $mode, ".depth")).as_slice(),
            include_bytes!(concat!("fixtures/xr_device/lavapipe/", $mode, ".rgba")).as_slice(),
            include_bytes!(concat!("fixtures/xr_device/lavapipe/", $mode, ".depth")).as_slice(),
        )
    };
}
type XrNativeAsset = (
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);
const XR_NATIVE_ASSETS: &[XrNativeAsset] = &[
    xr_asset!("axes"),
    xr_asset!("mesh"),
    xr_asset!("texture"),
    xr_asset!("finite-ray"),
    xr_asset!("infinite-ray"),
    xr_asset!("finite-zero"),
    xr_asset!("infinite-zero"),
    xr_asset!("color-update"),
];
#[test]
fn xr_real_presentations_replay_native_captures_on_each_device() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    let software = driver.device_name().contains("llvmpipe");
    for (mode, host_rgba, host_depth, software_rgba, software_depth) in XR_NATIVE_ASSETS {
        let (rgba, depth) = if software {
            (*software_rgba, *software_depth)
        } else {
            (*host_rgba, *host_depth)
        };
        native_golden::assert_capture(&capture_xr_scene(&driver, mode), rgba, depth, mode);
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn capture_xr_scene(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let fixture = xr_scene::XrDeviceScene::new(driver, mode).unwrap();
    let cview = fixture.view.get().unwrap().view();
    cview.set_z_layer_target(rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_DEFAULT);
    cview.set_z_layer_redraw_mode(true);
    let render = fixture.render_view();
    render.try_redraw().unwrap();
    fixture.mutate(mode).unwrap();
    render.try_redraw().unwrap();
    let capture = render.capture().unwrap();
    fixture.view.remove();
    capture
}
#[test]
fn xr_actual_context_submits_retained_texture_and_native_ray_clear_policy() {
    let recorder = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(recorder.clone(), 512).unwrap();
    let mesh = recorded_xr_scene(&driver, &recorder, "mesh");
    for mode in xr_scene::XR_CAPTURE_MODES {
        assert_xr_submission(
            &recorded_xr_scene(&driver, &recorder, mode),
            mode,
            &mesh.draws()[0],
        );
    }
}
fn recorded_xr_scene(
    driver: &Handle<Graphic3dRenderDriver>,
    recorder: &recording_device::RecordingDevice,
    mode: &str,
) -> RenderFrame {
    let fixture = xr_scene::XrDeviceScene::new(driver, mode).unwrap();
    fixture.render_view().try_redraw().unwrap();
    fixture.mutate(mode).unwrap();
    fixture.render_view().try_redraw().unwrap();
    assert!(fixture.context.is_displayed(&fixture.device.cast()));
    let frame = recorder.last_frame();
    fixture.view.remove();
    frame
}

fn assert_xr_submission(frame: &RenderFrame, mode: &str, mesh: &PrimitiveDraw) {
    assert!(
        !frame.draws().is_empty(),
        "received empty mode={mode}; expected actual XR draws"
    );
    if mode == "texture" {
        assert_ne!(
            &frame.draws()[0],
            mesh,
            "expected delegated image in retained snapshot"
        );
    }
    let expected = xr_expected_draw_count(mode);
    assert_eq!(
        frame.draws().len(),
        expected,
        "received mode={mode}; expected native retained arrays"
    );
}

fn xr_expected_draw_count(mode: &str) -> usize {
    if mode == "color-update" {
        return 3;
    }
    if mode.contains("ray") || mode == "finite-zero" {
        return 2;
    }
    1
}
