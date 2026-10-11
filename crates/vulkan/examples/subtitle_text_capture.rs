//! Capture actual Subtitle formatter backgrounds and foreground glyphs against native OpenGL.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "support/text_capture_request.rs"]
mod text_capture_request;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let (mode, prefix) =
        text_capture_request::parse_text_capture(&args[1..], &subtitle_scene::SUBTITLE_MODES)?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = subtitle_scene::SubtitleScene::new(&driver, mode);
    scene.text.scene.window.set_size(128, 128);
    scene.text.scene.view.must_be_resized();
    view_capture::capture_view(&scene.text.scene.view, 128, 128, prefix, false)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.text.scene.view.remove();
    Ok(())
}
