//! Dedicated cap resources and live sampler commands preserve submitted frame snapshots.
#[allow(dead_code)]
#[path = "support/capping_texture_scene.rs"]
mod capping_scene;
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
#[test]
fn dedicated_cap_revisions_sampler_and_alpha_keep_native_update_boundaries() {
    for (mode, changes) in [
        ("oblique-image-updated", false),
        ("oblique-image-refreshed", true),
        ("oblique-sampler-updated", true),
        ("oblique-plane-updated", true),
        ("oblique-mask-blend-alpha-updated", true),
    ] {
        assert_cap_update_snapshot(mode, changes);
    }
}
fn assert_cap_update_snapshot(mode: &str, changes: bool) {
    let recorder = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(recorder.clone(), 128).unwrap();
    let fixture = capping_scene::CappingTextureScene::new(&driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    let before = recorder.last_frame();
    fixture.mutate(mode);
    fixture.scene.render_view().try_redraw().unwrap();
    assert_eq!(
        before.draws() != recorder.last_frame().draws(),
        changes,
        "{mode}"
    );
    assert_eq!(recorder.frames.read()[0].draws(), before.draws());
    fixture.scene.view.remove();
}
