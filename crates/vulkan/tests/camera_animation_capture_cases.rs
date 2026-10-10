//! Each camera timeline fixture reaches retained draw snapshots without an immediate redraw.
#[path = "support/camera_animation_scene.rs"]
mod camera_animation_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use recording_device::RecordingDevice;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dStructureSource;
use rustycad_tkv3d::ais::*;
#[test]
fn animation_frames_invalidate_view_and_defer_submission_until_requested_redraw() {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.structure.is_displayed());
    assert!(!scene.group.is_null());
    assert_eq!(scene.vertices.vertex_number(), 3);
    scene.window.set_size(64, 64);
    scene.render_view().try_redraw().unwrap();
    let before = device.frames.read().len();
    scene.view.set_immediate_update(true);
    camera_animation_scene::apply(&scene.view, "animcam-mid");
    assert_eq!(device.frames.read().len(), before);
    assert!(scene.view.is_invalidated());
    assert!(scene.view.set_immediate_update(false));
    assert_eq!(scene.view.camera().eye(), Point3::new(2., 0., 20.));
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.frames.read().len(), before + 1);
    assert_eq!(device.last_frame().draws().len(), 1);
    scene.view.remove();
}
#[test]
fn native_fixture_modes_keep_complete_camera_state_and_retained_triangle() {
    for mode in camera_animation_scene::MODES {
        let device = RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
        let scene = scene::ViewerScene::new(&driver);
        camera_animation_scene::apply(&scene.view, mode);
        let expected = match mode {
            "animcam-start" | "animcam-paused" | "animcam-detached" => 20.,
            "animcam-mid" | "animcam-nested" => 30.,
            _ => 40.,
        };
        assert_eq!(scene.view.camera().scale(), expected, "{mode}");
        scene.render_view().try_redraw().unwrap();
        assert_eq!(device.last_frame().draws().len(), 1, "{mode}");
        scene.view.remove();
    }
}
#[test]
fn animation_constructor_supports_repeated_bounded_progress_for_performance() {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device, 128).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    let animation = camera_animation_scene::animation(&scene.view);
    animation.start(false).unwrap();
    for pts in [0.5, 1.5, 0.5] {
        assert!(animation.update(pts).unwrap());
    }
    assert_eq!(scene.view.camera().scale(), 25.);
    scene.view.remove();
}
