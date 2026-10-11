//! Native depth/ordering witnesses and strict stable Subtitle buffers, without OCCT linkage.
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
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use subtitle_capture_scene::subtitle_replay_capture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for mode in subtitle_scene::SUBTITLE_MODES {
        let capture = subtitle_replay_capture(mode)?;
        let index = subtitle_scene::SUBTITLE_MODES
            .iter()
            .position(|known| *known == mode)
            .unwrap();
        let (rgba, depth) = subtitle_native_assets::subtitle_native_pair("host", index);
        native_golden::assert_golden_depth(&capture.depth, depth, mode);
        if strict_stable_subtitle_mode(mode) {
            native_golden::assert_capture(&capture, rgba, depth, mode);
        }
        assert_subtitle_depth_witness(&capture, mode);
    }
    Ok(())
}
fn strict_stable_subtitle_mode(mode: &str) -> bool {
    matches!(
        mode,
        "subtitle-opaque-translucent"
            | "subtitle-front-culled"
            | "subtitle-empty"
            | "subtitle-whitespace"
            | "subtitle-zero-alpha-no-depth-write"
            | "subtitle-annotation-depth"
    )
}
fn assert_subtitle_depth_witness(capture: &martensite_vulkan::FrameCapture, mode: &str) {
    let green = capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == [0, 255, 0, 255])
        .count();
    match mode {
        "subtitle-zero-alpha-depth" => assert_eq!(green, 15985, "{mode}"),
        "subtitle-zero-alpha-no-depth-write" | "subtitle-annotation-depth" => {
            assert_eq!(green, 16384, "{mode}")
        }
        _ => {}
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_stable_policy_keeps_rounding_cases_in_separate_full_replay() {
        assert!(strict_stable_subtitle_mode(
            "subtitle-zero-alpha-no-depth-write"
        ));
        assert!(!strict_stable_subtitle_mode("subtitle-basic"));
    }
}
