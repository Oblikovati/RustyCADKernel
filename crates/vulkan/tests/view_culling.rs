//! View culling follows live bounds, camera state, and native On/Off/NoUpdate policy.
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Point3, Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
fn setup() -> (scene::ViewerScene, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    (scene::ViewerScene::new(&driver), device)
}
fn translate(scene: &scene::ViewerScene, x: f64) {
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(Transform3::translation_vector(
            Vector3::new(x, 0., 0.),
        ))));
}
fn assert_draws(scene: &scene::ViewerScene, device: &RecordingDevice, count: usize) {
    scene.render_view().try_redraw().unwrap();
    assert_eq!(device.last_frame().draws().len(), count);
}
#[test]
fn live_structure_bounds_and_camera_changes_reach_the_frustum_cache() {
    let (scene, device) = setup();
    assert_draws(&scene, &device, 1);
    translate(&scene, 40.);
    assert_draws(&scene, &device, 0);
    scene
        .view
        .camera()
        .set_eye(&Point3::new(40., 0., 20.))
        .unwrap();
    scene
        .view
        .camera()
        .set_center(&Point3::new(40., 0., 0.))
        .unwrap();
    assert_draws(&scene, &device, 1);
    translate(&scene, 0.);
    assert_draws(&scene, &device, 0);
}
#[test]
fn no_update_preserves_previous_cull_state_and_off_resets_it() {
    use Graphic3dRenderingParamsFrustumCulling::{NoUpdate, Off, On};
    let (scene, device) = setup();
    translate(&scene, 40.);
    assert_draws(&scene, &device, 0);
    for (state, offset, count) in [
        (NoUpdate, 0., 0),
        (Off, 40., 1),
        (NoUpdate, 40., 1),
        (On, 0., 1),
    ] {
        scene.view.change_rendering_params(&mut |params| {
            params.ray_tracing.frustum_culling_state = state;
        });
        translate(&scene, offset);
        assert_draws(&scene, &device, count);
    }
}
