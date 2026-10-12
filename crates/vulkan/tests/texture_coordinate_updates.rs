//! Retained coordinate policies snapshot live parameters and preserve earlier frame submissions.
#[allow(dead_code)]
#[path = "support/texture_generation_scene.rs"]
mod generation_scene;
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
use std::sync::Arc;
#[test]
fn generated_live_eye_plane_update_changes_draws_and_keeps_the_prior_snapshot() {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_texture_coordinates(
        device.clone(),
        128,
        Arc::new(FreeTypeTextSource::default()),
        RenderTextureCoordinatePipeline::GeneratedCompatibility,
    )
    .unwrap();
    let fixture = generation_scene::TextureGenerationScene::new(&driver, "eye-params-updated");
    fixture.scene.render_view().try_redraw().unwrap();
    let before = device.last_frame();
    fixture.mutate("eye-params-updated");
    fixture.scene.render_view().try_redraw().unwrap();
    assert_ne!(before.draws(), device.last_frame().draws());
    assert_eq!(device.frames.read()[0].draws(), before.draws());
    fixture.scene.view.remove();
}
#[test]
fn programmable_live_eye_planes_preserve_explicit_uv_behavior() {
    let device = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(device.clone(), 128).unwrap();
    let fixture = generation_scene::TextureGenerationScene::new(&driver, "eye-params-updated");
    fixture.scene.render_view().try_redraw().unwrap();
    let before = device.last_frame();
    fixture.mutate("eye-params-updated");
    fixture.scene.render_view().try_redraw().unwrap();
    assert_eq!(before.draws(), device.last_frame().draws());
    fixture.scene.view.remove();
}
