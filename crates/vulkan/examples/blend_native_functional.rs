//! Stable exact native buffers form a normal functional gate; full strict residuals stay opt-in.
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
    for mode in RETAINED_STABLE {
        check_stable_native(mode, false)?;
    }
    for mode in PIXEL_STABLE {
        check_stable_native(mode, true)?;
    }
    Ok(())
}
fn check_stable_native(mode: &str, is_pixel: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (rgba, depth) = blend_native_assets::blend_capture(mode, is_pixel);
    let capture = if is_pixel {
        blend_capture::capture_pixel_xor(mode)?
    } else {
        blend_capture::capture_retained_xor(mode)?
    };
    native_golden::assert_capture(&capture, rgba, depth, mode);
    Ok(())
}

const RETAINED_STABLE: [&str; 9] = [
    "decal-basic",
    "decal-foreground-alpha",
    "blend-colored",
    "blend-colored-alpha",
    "blend-alpha-zero",
    "blend-double",
    "blend-colored-overlap",
    "blend-palette",
    "blend-highlight",
];
const PIXEL_STABLE: [&str; 4] = [
    "decal-basic",
    "decal-centered",
    "pixel-reuse",
    "pixel-reuse-cached",
];
