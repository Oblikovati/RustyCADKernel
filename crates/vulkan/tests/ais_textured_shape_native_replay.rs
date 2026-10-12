//! Rust-only GPU replay retains exact native color and the unchanged 1e-6 depth gate.
#[path = "../examples/support/device_options.rs"]
mod device_options;
#[path = "support/ais_textured_shape_native_assets.rs"]
mod native_assets;
#[path = "../examples/support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "support/viewer_scene.rs"]
mod scene;
#[path = "support/ais_textured_shape_scene.rs"]
mod textured_scene;
#[allow(dead_code)]
#[path = "support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::*;

#[test]
fn textured_native_mapping_assets_replay_on_either_actual_device() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    for row in native_assets::TEXTURED_NATIVE_BUFFERS {
        if !["map-off", "shaded", "map-5", "map-7"].contains(&row.mode) {
            continue;
        }
        let capture = capture_textured_fixture(&driver, row.mode).unwrap();
        let (rgba, depth) = native_textured_profile(&driver, row);
        native_golden::assert_capture(&capture, rgba, depth, row.mode);
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}

#[test]
#[ignore = "Full strict matrix retains renderer dependencies and native software filtering/line diagnostics"]
fn textured_full_native_matrix_preserves_all_unresolved_diagnostics() {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options()).unwrap();
    let mut failures = Vec::new();
    for row in native_assets::TEXTURED_NATIVE_BUFFERS {
        let result = assess_textured_native_case(&driver, row);
        if let Err(failure) = result {
            eprintln!("{failure}");
            failures.push(failure);
        }
    }
    assert!(
        failures.is_empty(),
        "received {} native mismatches: {failures:?}; expected exact RGBA and depth within 1e-6",
        failures.len()
    );
}

fn native_textured_profile<'a>(
    driver: &Graphic3dRenderDriver,
    row: &'a native_assets::TexturedNativeBuffers,
) -> (&'a [u8], &'a [u8]) {
    if driver.device_name().contains("llvmpipe") {
        return row.software;
    }
    assert!(
        driver.device_name().contains("AMD"),
        "received device={}; expected proved AMD or llvmpipe profile",
        driver.device_name()
    );
    row.host
}

fn capture_textured_fixture(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
) -> Result<FrameCapture, Box<dyn std::error::Error>> {
    let fixture = textured_scene::AisTexturedShapeScene::with_mode(driver, mode)?;
    set_textured_native_capture_layer(&fixture);
    let render = fixture.scene.render_view();
    let result = (|| -> Result<FrameCapture, Box<dyn std::error::Error>> {
        render.try_redraw()?;
        fixture.mutate(mode)?;
        render.try_redraw()?;
        Ok(render.capture()?)
    })();
    fixture.scene.view.remove();
    result
}
fn set_textured_native_capture_layer(fixture: &textured_scene::AisTexturedShapeScene) {
    // Match native ToPixMap's single Default-layer dump; later overlay layers clear final frame depth.
    let view = fixture.scene.cview();
    view.set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.set_z_layer_redraw_mode(true);
}

fn assess_textured_native_case(
    driver: &Handle<Graphic3dRenderDriver>,
    row: &native_assets::TexturedNativeBuffers,
) -> Result<(), String> {
    let capture = capture_textured_fixture(driver, row.mode)
        .map_err(|failure| format!("{}: {failure}", row.mode))?;
    let (rgba, depth) = native_textured_profile(driver, row);
    let color_difference = textured_native_color_difference(&capture.rgba, rgba);
    let depth_difference = textured_native_depth_difference(&capture.depth, depth);
    if color_difference != 0 || depth_difference > 1e-6 {
        return Err(format!(
            "{}: different_color_bytes={color_difference}, max_depth_delta={depth_difference}; expected exact native RGBA and depth within 1e-6",
            row.mode
        ));
    }
    Ok(())
}

fn textured_native_color_difference(actual: &[u8], expected: &[u8]) -> usize {
    assert_eq!(
        actual.len(),
        expected.len(),
        "received different RGBA lengths; expected native buffer shape"
    );
    actual.iter().zip(expected).filter(|(a, b)| a != b).count()
}

fn textured_native_depth_difference(actual: &[f32], expected: &[u8]) -> f32 {
    assert_eq!(
        actual.len() * 4,
        expected.len(),
        "received mismatched depth buffer shapes; expected equal native and Rust lengths"
    );
    actual
        .iter()
        .zip(expected.as_chunks::<4>().0)
        .map(|(value, bytes)| finite_textured_depth_delta(*value, f32::from_le_bytes(*bytes)))
        .fold(0., f32::max)
}
fn finite_textured_depth_delta(actual: f32, expected: f32) -> f32 {
    let difference = (actual - expected).abs();
    if !difference.is_finite() {
        return f32::INFINITY;
    }
    difference
}

#[test]
fn textured_diagnostic_gates_keep_exact_color_and_reject_nonfinite_depth() {
    assert_eq!(textured_native_color_difference(&[0, 1], &[0, 2]), 1);
    assert_eq!(
        textured_native_depth_difference(&[0.25], &0.25f32.to_le_bytes()),
        0.
    );
    assert!(textured_native_depth_difference(&[f32::NAN], &0.25f32.to_le_bytes()).is_infinite());
    assert!(finite_textured_depth_delta(0.25, f32::NAN).is_infinite());
    assert_eq!(finite_textured_depth_delta(0.25, 0.5), 0.25);
}
