//! AIS lifecycle submits real presentation groups to the renderer through an injected device.
#[path = "support/recording_device.rs"]
mod recording_device;
#[path = "support/ais_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use recording_device::RecordingDevice;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkv3d::{prs_mgr::*, select_mgr::*};
use std::sync::atomic::Ordering;
fn ais_setup() -> (scene::AisViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    (scene::AisViewerScene::new(&driver), device)
}
#[test]
fn context_display_and_erase_update_the_real_view_and_reuse_presentations() {
    let (scene, device) = ais_setup();
    let frames = device.frames.read().len();
    scene.context.erase(&scene.object.cast(), true).unwrap();
    assert_eq!(device.frames.read().len(), frames + 1);
    assert!(device.last_frame().draws().is_empty());
    scene.context.display(&scene.object.cast(), true).unwrap();
    assert_eq!(device.frames.read().len(), frames + 2);
    assert_eq!(device.last_frame().draws().len(), 1);
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].color,
        [1., 0., 0., 1.]
    );
    assert_eq!(scene.object.computations.load(Ordering::Relaxed), 1);
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.render_view().capture().is_ok());
}
#[test]
fn display_mode_switches_color_and_keeps_only_one_visible_presentation() {
    let (scene, device) = ais_setup();
    scene
        .context
        .display_with_modes(&scene.object.cast(), 1, -1, true, PrsMgrDisplayStatus::None)
        .unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].color,
        [0., 1., 0., 1.]
    );
    assert_eq!(scene.object.presentations().len(), 2);
    scene.context.display(&scene.object.cast(), true).unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].color,
        [1., 0., 0., 1.]
    );
    assert_eq!(scene.object.computations.load(Ordering::Relaxed), 2);
}
#[test]
fn object_view_affinity_survives_erase_and_redisplay() {
    let (scene, device) = ais_setup();
    scene
        .context
        .set_view_affinity(&scene.object.cast(), &scene.view, false)
        .unwrap();
    scene.context.erase(&scene.object.cast(), false).unwrap();
    scene.context.display(&scene.object.cast(), true).unwrap();
    assert!(device.last_frame().draws().is_empty());
    scene
        .context
        .set_view_affinity(&scene.object.cast(), &scene.view, true)
        .unwrap();
    scene.context.update_current_viewer();
    assert_eq!(device.last_frame().draws().len(), 1);
}
#[test]
fn retained_ais_geometry_transform_and_resize_reach_the_rendered_frame() {
    let (scene, device) = ais_setup();
    scene.context.update_current_viewer();
    let matrix = *device.last_frame().draws()[0].clip_transform();
    scene
        .object
        .set_local_transformation(&Transform3::translation_vector(Vector3::new(2., 0., 0.)))
        .unwrap();
    scene.object.vertices.set_vertice(3, 0., 6., 0.).unwrap();
    scene.window.set_size(80, 40);
    scene.view.must_be_resized();
    scene.context.update_current_viewer();
    let frame = device.last_frame();
    assert_ne!(*frame.draws()[0].clip_transform(), matrix);
    assert_eq!(frame.draws()[0].vertices()[2].position, [0., 6., 0.]);
    assert_eq!(frame.extent(), [80, 40]);
}
#[test]
fn failed_presentation_mode_preserves_partial_native_status() {
    let (scene, device) = ais_setup();
    assert!(scene.object.accept_display_mode(0));
    assert!(!scene.object.accept_display_mode(2));
    assert!(
        scene
            .context
            .display_with_modes(
                &scene.object.cast(),
                2,
                -1,
                false,
                PrsMgrDisplayStatus::None
            )
            .is_err()
    );
    assert!(scene.context.is_displayed_mode(&scene.object.cast(), 2));
    scene.context.update_current_viewer();
    assert!(device.last_frame().draws().is_empty());
    scene.context.display(&scene.object.cast(), true).unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
}
#[test]
fn real_interactive_triangle_activates_and_deactivates_its_sensitive_point() {
    let (scene, _) = ais_setup();
    let manager = scene.context.selection_manager().clone();
    let owner = scene.object.global_sel_owner();
    assert!(!owner.is_null());
    assert!(owner.is_same_selectable(&scene.object.cast()));
    assert!(manager.is_activated(&scene.object.cast(), 0).unwrap());
    scene.context.erase(&scene.object.cast(), false).unwrap();
    assert!(!manager.is_activated(&scene.object.cast(), 0).unwrap());
    scene.context.display(&scene.object.cast(), false).unwrap();
    assert!(manager.is_activated(&scene.object.cast(), 0).unwrap());
    assert_eq!(owner, scene.object.global_sel_owner());
    assert!(scene.context.display(&Handle::default(), false).is_ok());
}
#[test]
fn removal_destroys_gpu_presentations_and_redisplay_recomputes_geometry() {
    let (scene, device) = ais_setup();
    scene.context.update_current_viewer();
    let previous = device.frames.read().len();
    scene.context.remove(&scene.object.cast(), true).unwrap();
    // ClearGlobal tests status after resetting it: native Remove(true) does not redraw by itself.
    assert_eq!(device.frames.read().len(), previous);
    scene.view.redraw();
    assert!(device.last_frame().draws().is_empty());
    scene.context.display(&scene.object.cast(), true).unwrap();
    assert_eq!(device.last_frame().draws().len(), 1);
    assert_eq!(scene.object.computations.load(Ordering::Relaxed), 2);
    assert!(
        scene
            .context
            .selection_manager()
            .is_activated(&scene.object.cast(), 0)
            .unwrap()
    );
}
#[test]
fn bulk_lifecycle_updates_once_and_clear_prs_retains_selection_activation() {
    let (scene, device) = ais_setup();
    let before = device.frames.read().len();
    scene.context.erase_all(true).unwrap();
    assert_eq!(device.frames.read().len(), before + 1);
    assert!(device.last_frame().draws().is_empty());
    scene.context.display_all(true).unwrap();
    assert_eq!(device.frames.read().len(), before + 2);
    assert_eq!(device.last_frame().draws().len(), 1);
    scene
        .context
        .clear_prs(&scene.object.cast(), 0, true)
        .unwrap();
    assert_eq!(device.frames.read().len(), before + 3);
    assert!(device.last_frame().draws().is_empty());
    assert!(
        scene
            .context
            .selection_manager()
            .is_activated(&scene.object.cast(), 0)
            .unwrap()
    );
    scene.context.remove_all(true).unwrap();
    assert_eq!(device.frames.read().len(), before + 4);
    assert!(scene.context.fields.read().objects.is_empty());
}
