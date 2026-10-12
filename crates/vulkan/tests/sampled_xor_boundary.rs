//! Real retained images must survive ordered UNORM XOR to SRGB surface rendering.
#[allow(dead_code)]
#[path = "../examples/support/blend_boundary_runtime.rs"]
mod boundary;
#[allow(dead_code)]
#[path = "../examples/support/capping_runtime.rs"]
mod capping_runtime;
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod texture_scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;

#[test]
fn actual_retained_sampled_surfaces_preserve_xor_attachment_color_and_depth() {
    let mut renderer = VulkanRenderer::new(device_options::validation_options()).unwrap();
    for mode in ["nearest", "phong", "mask-blend", "blend"] {
        assert_retained_texture_boundary(&mut renderer, mode);
    }
    assert!(renderer.validation_errors().is_empty());
}
fn assert_retained_texture_boundary(renderer: &mut VulkanRenderer, mode: &str) {
    let recorded = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(recorded.clone(), 128).unwrap();
    let fixture = texture_scene::SurfaceTextureScene::with_mode(&driver, mode);
    fixture.scene.render_view().try_redraw().unwrap();
    let sampled = recorded.last_frame().draws().to_vec();
    assert!(
        !sampled.is_empty(),
        "mode={mode}; expected retained surface draws"
    );
    let mut draws = vec![boundary::fullscreen_xor()];
    draws.extend(sampled.clone());
    let actual = RenderFrame::new(64, 64, [0., 0., 1., 1.], draws).unwrap();
    let control = RenderFrame::new(64, 64, [1., 1., 0., 1.], sampled).unwrap();
    boundary::assert_capture_identity(renderer, &actual, &control, mode);
    fixture.scene.view.remove();
}
