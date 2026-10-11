//! Real native OpenGL draw matrices replay through a named CPU glyph/device fixture.
#[path = "support/decal_scene.rs"]
mod decal_scene;
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
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, graphic3d::*};
use std::sync::Arc;
fn decal_recording_driver() -> (Handle<Graphic3dRenderDriver>, RecordingDevice) {
    let device = RecordingDevice::default();
    let driver = Graphic3dRenderDriver::with_backend_and_fonts(
        device.clone(),
        256,
        Arc::new(visible_fixed_glyph_source::VisibleFixedGlyphSource::default()),
    )
    .unwrap();
    (driver, device)
}
#[test]
fn native_decal_draw_matrices_colors_depth_and_lifecycle_replay_without_occt() {
    for (family, display) in [
        (0, AspectTypeOfDisplayText::Dekale),
        (1, AspectTypeOfDisplayText::Shadow),
        (2, AspectTypeOfDisplayText::Subtitle),
    ] {
        replay_native_text_family(family, display);
    }
}
fn replay_native_text_family(family: usize, display: AspectTypeOfDisplayText) {
    for (index, mode) in decal_scene::DECAL_MODES.iter().enumerate() {
        replay_decal_mode(family, display, index, mode);
    }
}
fn replay_decal_mode(family: usize, display: AspectTypeOfDisplayText, index: usize, mode: &str) {
    let (driver, device) = decal_recording_driver();
    let fixture = decal_scene::DecalScene::new(&driver, mode);
    fixture.text.style.set_text_display_type(display);
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    fixture.text.scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    for row in native_text_rows(family, index, mode) {
        assert_native_decal_row(&frame, row, mode);
    }
    if mode == "decal-empty" || mode == "decal-whitespace" {
        assert!(frame.draws().is_empty());
    } else {
        assert_live_decal_colors(&fixture, &device);
    }
    fixture.text.scene.view.remove();
}
fn assert_native_decal_row(frame: &RenderFrame, row: &str, mode: &str) {
    let fields: Vec<f32> = row.split_whitespace().map(|v| v.parse().unwrap()).collect();
    let draw = &frame.draws()[fields[2] as usize];
    for (actual, native) in draw.clip_transform().iter().zip(&fields[4..20]) {
        assert!(
            (actual - native).abs() < 1e-6,
            "{mode}: clip {actual}; expected {native}"
        );
    }
    assert_eq!(draw.vertices()[0].color, fields[20..24], "{mode}");
    assert!(!draw.is_surface_blended());
    assert_native_text_bias(draw, &fields, mode);
}
fn assert_native_text_bias(draw: &PrimitiveDraw, fields: &[f32], mode: &str) {
    let annotation = mode == "decal-annotation" || mode == "decal-annotation-depth";
    assert_eq!(fields[24], if annotation { 0. } else { 1. }, "{mode}");
    let units = draw.polygon_offset().unwrap().parameters().2;
    assert_eq!(
        fields[26],
        units
            + if !annotation && fields[2] < if fields[0] == 0. { 4. } else { 1. } {
                1.
            } else {
                0.
            },
        "{mode}"
    );
}
fn assert_live_decal_colors(fixture: &decal_scene::DecalScene, device: &RecordingDevice) {
    fixture
        .text
        .style
        .set_edge_color_rgba(ColorRgba::new(0., 1., 0., 0.3).unwrap());
    fixture.text.scene.render_view().try_redraw().unwrap();
    let count = if fixture.text.style.text_display_type() == AspectTypeOfDisplayText::Dekale {
        4
    } else {
        1
    };
    for draw in device.last_frame().draws().iter().take(count) {
        assert_eq!(draw.vertices()[0].color, [0., 1., 0., 0.3]);
    }
}

fn native_text_row_identity(row: &str) -> (usize, usize) {
    let mut fields = row.split_whitespace();
    (
        fields.next().unwrap().parse().unwrap(),
        fields.next().unwrap().parse().unwrap(),
    )
}

fn native_text_rows(family: usize, index: usize, mode: &str) -> Vec<&'static str> {
    let fixture = include_str!("fixtures/decal_text/native-draw-trace.txt");
    let rows: Vec<&str> = fixture
        .lines()
        .filter(|row| native_text_row_identity(row) == (family, index))
        .collect();
    let expected = if mode == "decal-empty" || mode == "decal-whitespace" {
        0
    } else if family == 0 {
        5
    } else {
        2
    };
    assert_eq!(
        rows.len(),
        expected,
        "family {family} mode {mode}; expected complete native draw sequence"
    );
    rows
}
