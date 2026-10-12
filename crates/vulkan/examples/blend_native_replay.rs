//! Opt-in full strict native XOR replay; every residual remains an ordinary gate failure.
#[path = "support/blend_capture.rs"]
mod blend_capture;
#[path = "support/blend_native_assets.rs"]
mod blend_native_assets;
#[path = "../tests/support/blend_scene.rs"]
mod blend_scene;
#[path = "../tests/support/decal_scene.rs"]
mod decal_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/native_golden.rs"]
mod native_golden;
#[path = "../tests/support/pixel_scene.rs"]
mod pixel_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(format!("received {args:?}; expected retained|pixel mode").into());
    }
    let is_pixel = replay_family(&args[1])?;
    let (rgba, depth) = blend_native_assets::blend_capture(&args[2], is_pixel);
    let capture = if is_pixel {
        blend_capture::capture_pixel_xor(&args[2])?
    } else {
        blend_capture::capture_retained_xor(&args[2])?
    };
    native_golden::assert_capture(&capture, rgba, depth, &args[2]);
    Ok(())
}
fn replay_family(value: &str) -> Result<bool, String> {
    match value {
        "retained" => Ok(false),
        "pixel" => Ok(true),
        actual => Err(format!("family {actual}; expected retained or pixel")),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_family_rejects_unknown_shape_and_preserves_both_native_frontends() {
        assert_eq!(replay_family("retained"), Ok(false));
        assert_eq!(replay_family("pixel"), Ok(true));
        let error = replay_family("unknown").unwrap_err();
        assert!(error.contains("unknown") && error.contains("retained or pixel"));
    }
}
