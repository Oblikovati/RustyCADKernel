//! Native non-fill commands remain ordered even without retained geometry.
#[path = "support/capping_scene.rs"]
mod capping_scene;
#[allow(dead_code)]
#[path = "support/oit_scene.rs"]
mod oit_scene;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/ordered_stencil_scene.rs"]
mod stencil_scene;
#[allow(dead_code)]
#[path = "support/surface_texture_scene.rs"]
mod surface_texture_scene;
#[path = "support/view_setup.rs"]
mod view_setup;
use surface_texture_scene as texture_scene;
#[allow(dead_code)]
#[path = "support/capping_texture_scene.rs"]
mod capping_texture_scene;
use martensite_vulkan::*;
#[test]
fn retained_enable_disable_and_trailing_empty_commands_do_not_add_geometry() {
    for mode in [
        "enable",
        "disable",
        "toggle",
        "trailing-empty",
        "leading-empty",
    ] {
        assert_retained_mode(mode);
    }
}
fn assert_retained_mode(mode: &str) {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let fixture = stencil_scene::OrderedStencilScene::new(&driver, mode, "unordered");
    fixture.initial_redraw(mode).unwrap();
    let frame = backend.last_frame();
    let states = frame.stencil_draw_states();
    assert_eq!(
        states.len(),
        if mode == "leading-empty" { 1 } else { 2 },
        "{mode}"
    );
    assert_eq!(
        states[0].is_enabled(),
        mode != "disable" && mode != "toggle",
        "{mode}"
    );
    fixture.finish();
}
#[test]
fn cap_fill_only_filter_excludes_group_stencil_options_and_restores_context() {
    let backend = recording_device::RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend(backend.clone(), 128).unwrap();
    let fixture = stencil_scene::OrderedStencilScene::new(&driver, "cap-enable", "peeling");
    fixture.fixture.scene.render_view().try_redraw().unwrap();
    let frame = backend.last_frame();
    for (draw, state) in frame.draws().iter().zip(frame.stencil_draw_states()) {
        if matches!(
            draw.capping_pass(),
            RenderCappingPass::MaskStart | RenderCappingPass::MaskContinue
        ) {
            assert_eq!(state.native_comparison(), (0x207, 1, 1));
        }
        if draw.capping_pass() == RenderCappingPass::Surface {
            assert_eq!(state.native_comparison(), (0x202, 1, 1));
        }
    }
    fixture.finish();
}
