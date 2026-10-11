//! Retained Subtitle keeps live decoration order, culling and layer policy independent of GPU I/O.
#[path = "support/fixed_glyph_source.rs"]
mod fixed_glyph_source;
#[path = "support/recording_device.rs"]
mod recording_device;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "support/text_scene.rs"]
mod text_scene;
#[path = "support/view_setup.rs"]
mod view_setup;
#[path = "support/visible_fixed_glyph_source.rs"]
mod visible_fixed_glyph_source;
use martensite_vulkan::*;
use recording_device::RecordingDevice;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use std::sync::Arc;
use visible_fixed_glyph_source::VisibleFixedGlyphSource;
fn subtitle_recording_driver() -> (Handle<Graphic3dRenderDriver>, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_fonts(
        device.clone(),
        256,
        Arc::new(VisibleFixedGlyphSource::default()),
    )
    .unwrap();
    (driver, device)
}
#[test]
fn native_subtitle_modes_keep_ordered_live_style_and_layer_depth_policy() {
    for mode in subtitle_scene::SUBTITLE_MODES {
        let (driver, device) = subtitle_recording_driver();
        check_subtitle_retained_mode(&driver, &device, mode);
    }
}
fn check_subtitle_retained_mode(
    driver: &Handle<Graphic3dRenderDriver>,
    device: &RecordingDevice,
    mode: &str,
) {
    let fixture = subtitle_scene::SubtitleScene::new(driver, mode);
    fixture.text.scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    if mode == "subtitle-empty" || mode == "subtitle-whitespace" {
        assert!(frame.draws().is_empty());
        fixture.text.scene.view.remove();
        return;
    }
    assert_subtitle_pass_shape(&frame, mode);
    assert_subtitle_live_color(&fixture, device);
    fixture.text.scene.view.remove();
}
fn assert_subtitle_live_color(fixture: &subtitle_scene::SubtitleScene, device: &RecordingDevice) {
    let color = ColorRgba::new(0., 1., 0., 0.3).unwrap();
    fixture.text.style.set_edge_color_rgba(color);
    fixture.text.scene.render_view().try_redraw().unwrap();
    assert_eq!(
        device.last_frame().draws()[0].vertices()[0].color,
        [0., 1., 0., 0.3]
    );
}
fn assert_subtitle_pass_shape(frame: &RenderFrame, mode: &str) {
    assert!(frame.draws()[0].textured_style().is_none());
    assert!(frame.draws()[1].textured_style().is_some());
    assert!(frame.draws().iter().all(|draw| !draw.is_surface_blended()));
    if mode.ends_with("-depth") || mode == "subtitle-zero-alpha-no-depth-write" {
        assert_eq!(frame.draws().len(), 3);
    }
    assert_eq!(
        frame
            .layers()
            .iter()
            .find(|layer| layer.draw_range().contains(&0))
            .unwrap()
            .depth_policy()
            .1,
        mode != "subtitle-zero-alpha-no-depth-write",
        "{mode}"
    );
}
const SUBTITLE_CULLING_POLICIES: [(Graphic3dTypeOfBackfacingModel, RenderFaceCulling); 3] = [
    (
        Graphic3dTypeOfBackfacingModel::BackCulled,
        RenderFaceCulling::Back,
    ),
    (
        Graphic3dTypeOfBackfacingModel::FrontCulled,
        RenderFaceCulling::Front,
    ),
    (
        Graphic3dTypeOfBackfacingModel::DoubleSided,
        RenderFaceCulling::None,
    ),
];
#[test]
fn subtitle_rectangle_and_glyphs_share_live_native_face_rejection() {
    let (driver, device) = subtitle_recording_driver();
    let fixture = subtitle_scene::SubtitleScene::new(&driver, "subtitle-basic");
    for (policy, expected) in SUBTITLE_CULLING_POLICIES {
        fixture.text.style.set_face_culling(policy);
        fixture.text.scene.render_view().try_redraw().unwrap();
        assert!(
            device
                .last_frame()
                .draws()
                .iter()
                .all(|draw| draw.face_culling() == expected)
        );
    }
    fixture.text.scene.view.remove();
}
