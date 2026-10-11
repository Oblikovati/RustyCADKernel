//! Object timeline modes reach retained geometry and preserve native invalidation policy.
#[path = "support/ais_scene.rs"]
mod ais_scene;
#[path = "support/object_animation_scene.rs"]
mod object_animation_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use recording_device::RecordingDevice;
use rustycad_tkservice::aspect::AspectWindow;
use rustycad_tkv3d::{ais::*, prs_mgr::PrsMgrPresentableObjectMembers};
#[test]
fn object_updates_defer_redraw_and_keep_active_selection() {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let scene = ais_scene::AisViewerScene::new(&driver);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert_eq!(scene.window.size(), (64, 64));
    let count = device.frames.read().len();
    object_animation_scene::apply(&scene, "animobj-mid", 0.);
    assert_eq!(device.frames.read().len(), count);
    assert!(scene.view.is_invalidated());
    assert_eq!(
        scene.object.local_transformation().translation_part().x(),
        2.
    );
    assert_eq!(scene.object.local_transformation().scale_factor(), 0.75);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    scene.view.remove();
}
#[test]
fn all_capture_modes_keep_exact_local_endpoints() {
    for mode in object_animation_scene::MODES {
        let device = RecordingDevice::default();
        let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
        let scene = ais_scene::AisViewerScene::new(&driver);
        object_animation_scene::apply(&scene, mode, 0.);
        let expected = match mode {
            "animobj-start" | "animobj-instant" | "animobj-paused" => 0.,
            "animobj-end" => 4.,
            _ => 2.,
        };
        assert_eq!(
            scene.object.local_transformation().translation_part().x(),
            expected,
            "{mode}"
        );
        scene.render_view().try_redraw().unwrap();
        assert!(device.last_frame().draws().len() <= 1);
        scene.view.remove();
    }
}
#[test]
fn immediate_layer_invalidates_only_for_escaped_z_range() {
    for (mode, invalidated) in [
        ("animobj-immediate", false),
        ("animobj-escaped", true),
        ("animobj-detached", false),
    ] {
        let driver = Graphic3dRenderDriver::with_backend(RecordingDevice::default(), 128).unwrap();
        let scene = ais_scene::AisViewerScene::new(&driver);
        scene.render_view().try_redraw().unwrap();
        object_animation_scene::apply(&scene, mode, 0.);
        assert_eq!(scene.view.is_invalidated(), invalidated, "{mode}");
        assert_eq!(
            scene.view.camera().z_far(),
            if invalidated { 5. } else { 100. }
        );
        scene.view.remove();
    }
}
#[test]
fn performance_animation_reuses_bounded_progress_and_presentation() {
    let driver = Graphic3dRenderDriver::with_backend(RecordingDevice::default(), 128).unwrap();
    let scene = ais_scene::AisViewerScene::new(&driver);
    let animation = object_animation_scene::animation(&scene, 0.);
    animation.start(false).unwrap();
    for pts in [0.5, 1.5, 0.5] {
        assert!(animation.update(pts).unwrap());
    }
    assert_eq!(
        scene.object.local_transformation().translation_part().x(),
        1.
    );
    assert_eq!(scene.object.local_transformation().scale_factor(), 0.875);
    scene.view.remove();
}
