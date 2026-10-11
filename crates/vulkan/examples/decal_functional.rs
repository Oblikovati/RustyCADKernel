//! Strict stable native buffers exercise outline/alpha/culling/depth contracts on both devices.
#[path = "support/decal_native_assets.rs"]
mod decal_native_assets;
#[path = "../tests/support/decal_scene.rs"]
mod decal_scene;
#[path = "support/decorated_text_capture.rs"]
mod decorated_text_capture;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/native_golden.rs"]
mod native_golden;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "support/text_family_request.rs"]
mod text_family_request;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    for family in ["decal", "shadow", "subtitle"] {
        for mode in STABLE_NATIVE_TEXT_CASES {
            check_native_text_buffer(family, mode)?;
        }
    }
    Ok(())
}
const STABLE_NATIVE_TEXT_CASES: [&str; 6] = [
    "decal-opaque-translucent",
    "decal-front-culled",
    "decal-empty",
    "decal-whitespace",
    "decal-zero-alpha-no-depth-write",
    "decal-annotation-depth",
];
fn check_native_text_buffer(family: &str, mode: &str) -> Result<(), Box<dyn std::error::Error>> {
    let display = text_family_request::text_family(Some(family))?;
    let capture = decorated_text_capture::capture_decorated_text(mode, display)?;
    let (rgba, depth) = decal_native_assets::native_text_capture(family, mode);
    native_golden::assert_capture(&capture, rgba, depth, mode);
    Ok(())
}
