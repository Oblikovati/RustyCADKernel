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
use martensite_vulkan::Graphic3dRenderDriver;
use recording_device::RecordingDevice;
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
#[test]
fn retained_capture_cases_submit_live_clipping_and_clear_it_when_requested() {
    for mode in clipping_scene::MODES {
        let device = RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
        let scene = scene::ViewerScene::new(&driver);
        // Both native and Rust capture executables set the structure offset first.
        // Even identity transformation invalidates layer membership at this point.
        scene
            .structure
            .set_transformation(&rustycad_tkernel::standard::Handle::new(
                rustycad_tkmath::toploc::Datum3::new(rustycad_tkmath::gp::Transform3::default()),
            ));
        clipping_scene::configure_clipping(&scene, mode);
        scene.render_view().try_redraw().unwrap();
        let frame = device.last_frame();
        if matches!(
            mode,
            "clip-outside" | "clip-pers-out" | "clip-text-outside" | "clip-group-pers"
        ) {
            assert!(frame.draws().is_empty(), "{mode}");
            continue;
        }
        assert!(!frame.draws().is_empty(), "{mode}");
        let clipped = !matches!(
            mode,
            "clip-text-baseline"
                | "clip-empty-override"
                | "clip-head-off"
                | "clip-cleared"
                | "clip-inside"
                | "clip-pers-in"
                | "clip-pers-2d"
        );
        assert_eq!(frame.draws()[0].clipping().is_some(), clipped, "{mode}");
    }
}
