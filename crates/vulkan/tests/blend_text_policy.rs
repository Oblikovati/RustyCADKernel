//! Actual native GL_XOR matrices/colors replay through the named CPU renderer fake.
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
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, graphic3d::*};
use std::sync::Arc;
fn blend_recording_driver() -> (Handle<Graphic3dRenderDriver>, RecordingDevice) {
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
fn actual_native_xor_matrices_colors_and_depth_policy_replay_without_occt() {
    for (index, mode) in decal_scene::DECAL_MODES.iter().enumerate() {
        replay_blend_trace(index, mode);
    }
}
fn replay_blend_trace(index: usize, mode: &str) {
    let (driver, device) = blend_recording_driver();
    let fixture = decal_scene::DecalScene::new(&driver, mode);
    fixture
        .text
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    fixture.text.scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    for row in include_str!("fixtures/blend_text/native-draw-trace.txt").lines() {
        let fields: Vec<f32> = row.split_whitespace().map(|v| v.parse().unwrap()).collect();
        if fields[0] as usize != index {
            continue;
        }
        assert_blend_native_draw(&frame, &fields, mode);
    }
    fixture.text.scene.view.remove();
}
fn assert_blend_native_draw(frame: &RenderFrame, fields: &[f32], mode: &str) {
    let draw = &frame.draws()[0];
    for (actual, native) in draw.clip_transform().iter().zip(&fields[3..19]) {
        assert!(
            (actual - native).abs() < 1e-6,
            "{mode}: {actual}; expected native clip {native}"
        );
    }
    let native_color: [f32; 4] = fields[19..23].try_into().unwrap();
    assert_eq!(draw.vertices()[0].color, native_color);
    assert_eq!(fields[26], 1., "native COLOR_LOGIC_OP enabled for {mode}");
    assert_eq!(fields[27], 5382., "native GL_XOR for {mode}");
    assert!(!draw.is_surface_blended());
}

#[allow(dead_code)]
#[path = "support/pixel_scene.rs"]
mod pixel_scene;
#[test]
fn true_native_set2d_matrices_and_reset_reuse_replay_with_named_font_fake() {
    for mode in decal_scene::DECAL_MODES
        .into_iter()
        .chain(["pixel-reuse", "pixel-reuse-cached"])
    {
        replay_pixel_trace(mode);
    }
}
fn replay_pixel_trace(mode: &str) {
    let (driver, _) = blend_recording_driver();
    let reuse = mode.starts_with("pixel-reuse");
    let fixture = decal_scene::DecalScene::new(&driver, if reuse { "decal-basic" } else { mode });
    let source = visible_fixed_glyph_source::VisibleFixedGlyphSource::default();
    let frame = if mode == "pixel-reuse-cached" {
        pixel_scene::pixel_cached_fixture(&fixture, &source).unwrap()
    } else {
        pixel_scene::pixel_fixture_with_source(&fixture, reuse, &source).unwrap()
    };
    let rows = native_pixel_rows(mode);
    assert_eq!(
        rows.len(),
        usize::from(!["decal-empty", "decal-whitespace"].contains(&mode))
    );
    for row in rows {
        assert_pixel_native_draw(&frame, row, mode);
    }
    fixture.text.scene.view.remove();
}
fn assert_pixel_native_draw(frame: &RenderFrame, row: &str, mode: &str) {
    let fields: Vec<f32> = row
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap())
        .collect();
    let draw = &frame.draws()[0];
    for (actual, native) in draw.clip_transform().iter().zip(&fields[2..18]) {
        assert!(
            (actual - native).abs() < 1e-6,
            "{mode}: {actual}; expected native pixel clip {native}"
        );
    }
    assert_eq!(fields[22], 0., "native true2D depth test disabled");
    let color: [f32; 4] = fields[18..22].try_into().unwrap();
    assert_eq!(draw.vertices()[0].color, color);
}

#[path = "support/blend_scene.rs"]
mod blend_scene;
#[test]
fn native_colored_xor_overlap_cancellation_and_live_highlight_replay_with_named_fake() {
    for mode in blend_scene::BLEND_EXTRA_MODES {
        replay_blend_extra(mode);
    }
}
fn replay_blend_extra(mode: &str) {
    let (driver, device) = blend_recording_driver();
    let fixture = blend_scene::blend_extra_fixture(&driver, mode);
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    fixture.text.scene.render_view().try_redraw().unwrap();
    let frame = device.last_frame();
    let rows = native_extra_rows(mode);
    assert_eq!(rows.len(), frame.draws().len());
    for row in rows {
        assert_extra_native_draw(&frame, row, mode);
    }
    fixture.text.scene.view.remove();
}
fn native_extra_rows(mode: &str) -> Vec<&'static str> {
    include_str!("fixtures/blend_text/native-extras-trace.txt")
        .lines()
        .filter(|row| row.split_whitespace().next() == Some(mode))
        .collect()
}
fn assert_extra_native_draw(frame: &RenderFrame, row: &str, mode: &str) {
    let fields: Vec<f32> = row
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap())
        .collect();
    let draw = &frame.draws()[fields[0] as usize];
    for (actual, native) in draw.clip_transform().iter().zip(&fields[2..18]) {
        assert!(
            (actual - native).abs() < 1e-6,
            "{mode}: {actual}; expected native {native}"
        );
    }
    let rgba: [f32; 4] = fields[18..22].try_into().unwrap();
    let expected = rgba;
    assert_eq!(draw.vertices()[0].color, expected, "{mode}");
    assert_eq!(draw.polygon_offset().unwrap().parameters().2, fields[24]);
    assert_native_xor_attachment(&fields);
}

#[test]
fn owned_pixel_layout_dependency_matches_native_explicit_reset_requirement() {
    let (driver, _) = blend_recording_driver();
    let fixture = decal_scene::DecalScene::new(&driver, "decal-basic");
    let fonts = fixed_glyph_source::FixedGlyphSource::default();
    let cached = pixel_scene::pixel_cached_fixture(&fixture, &fonts).unwrap();
    assert!(fonts.requests.read()[0].0.starts_with(b"AV"));
    let reset = pixel_scene::pixel_fixture_with_source(&fixture, true, &fonts).unwrap();
    assert!(fonts.requests.read()[1].0.starts_with(b"V"));
    assert_eq!(
        cached.draws()[0].clip_transform(),
        reset.draws()[0].clip_transform()
    );
    fixture.text.scene.view.remove();
}

#[test]
fn unknown_pixel_reuse_mode_reports_the_offending_value_and_expected_shape() {
    let (driver, _) = blend_recording_driver();
    let fixture = decal_scene::DecalScene::new(&driver, "decal-basic");
    let error = pixel_scene::pixel_case_frame(&fixture, "pixel-reuse-unknown")
        .unwrap_err()
        .to_string();
    assert!(error.contains("pixel-reuse-unknown") && error.contains("DECAL_MODES"));
    fixture.text.scene.view.remove();
}

fn assert_native_xor_attachment(fields: &[f32]) {
    assert_eq!(fields.len(), 39);
    assert_eq!(fields[28..31], [1., 35904., 35863.]);
    assert_eq!(fields[31..35], [8.; 4]);
    assert_eq!(fields[35..39], [1., 1., 1., 0.]);
}

fn native_pixel_rows(mode: &str) -> Vec<&'static str> {
    include_str!("fixtures/blend_text/native-pixel-trace.txt")
        .lines()
        .filter(|row| row.split_whitespace().next() == Some(mode))
        .collect()
}
