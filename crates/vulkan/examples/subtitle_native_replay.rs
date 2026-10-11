//! Strict device-bound Subtitle replay; OCCT is absent from the Rust executable.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/subtitle_capture_scene.rs"]
mod subtitle_capture_scene;
#[path = "support/subtitle_native_assets.rs"]
mod subtitle_native_assets;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "support/text_capture_request.rs"]
mod text_capture_request;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use subtitle_capture_scene::subtitle_replay_capture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let (mode, profile) =
        text_capture_request::parse_text_capture(&args[1..], &subtitle_scene::SUBTITLE_MODES)?;
    let index = subtitle_scene::SUBTITLE_MODES
        .iter()
        .position(|known| *known == mode)
        .unwrap();
    let (rgba, depth) = subtitle_native_assets::subtitle_native_pair(profile, index);
    let capture = subtitle_replay_capture(mode)?;
    native_golden::assert_capture(&capture, rgba, depth, mode);
    Ok(())
}
