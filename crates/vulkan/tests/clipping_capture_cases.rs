//! Every native capture fixture reaches the immutable retained draw stream.
#[path = "support/clipping_scene.rs"]
mod clipping_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderFrame};
use recording_device::RecordingDevice;
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
#[test]
fn retained_capture_cases_submit_live_clipping_and_clear_it_when_requested() {
    for mode in clipping_scene::MODES {
        verify_clipping_frame(mode, &record_clipping_frame(mode));
    }
}
fn record_clipping_frame(mode: &str) -> RenderFrame {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    // Both native and Rust executables set the structure offset first. Even identity
    // transformation invalidates layer membership here and preserves the native capture lifecycle.
    scene
        .structure
        .set_transformation(&rustycad_tkernel::standard::Handle::new(
            rustycad_tkmath::toploc::Datum3::new(rustycad_tkmath::gp::Transform3::default()),
        ));
    clipping_scene::configure_clipping(&scene, mode);
    scene.render_view().try_redraw().unwrap();
    device.last_frame()
}
fn verify_clipping_frame(mode: &str, frame: &RenderFrame) {
    if matches!(
        mode,
        "clip-outside" | "clip-pers-out" | "clip-text-outside" | "clip-group-pers"
    ) {
        assert!(frame.draws().is_empty(), "{mode}");
        return;
    }
    assert!(!frame.draws().is_empty(), "{mode}");
    assert_eq!(
        frame.draws()[0].clipping().is_some(),
        clipping_expected(mode),
        "{mode}"
    );
}
fn clipping_expected(mode: &str) -> bool {
    !matches!(
        mode,
        "clip-text-baseline"
            | "clip-empty-override"
            | "clip-head-off"
            | "clip-cleared"
            | "clip-derived-alias-clear"
            | "clip-inside"
            | "clip-pers-in"
            | "clip-pers-2d"
    )
}
