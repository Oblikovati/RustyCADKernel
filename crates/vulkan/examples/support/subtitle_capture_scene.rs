//! Capture retained Subtitle via the same ordinary V3d path as native reference assets.
use super::{device_options, subtitle_scene};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkservice::graphic3d::*;
/// Capture one native-sized default-layer scene; `subtitle_replay_capture("subtitle-basic")` includes validation.
pub fn subtitle_replay_capture(mode: &str) -> Result<FrameCapture, Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = subtitle_scene::SubtitleScene::new(&driver, mode);
    prepare_subtitle_replay_target(&fixture);
    fixture.text.scene.render_view().try_redraw()?;
    let capture = fixture.text.scene.render_view().capture().unwrap();
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.text.scene.view.remove();
    Ok(capture)
}

fn prepare_subtitle_replay_target(fixture: &subtitle_scene::SubtitleScene) {
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    fixture
        .text
        .scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    fixture.text.scene.cview().set_z_layer_redraw_mode(true);
}
