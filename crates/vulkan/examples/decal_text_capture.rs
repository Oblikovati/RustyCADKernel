//! Capture actual Decal outline passes and foreground glyphs against native OpenGL.
#[path = "../tests/support/decal_scene.rs"]
mod decal_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "support/text_capture_request.rs"]
mod text_capture_request;
#[path = "support/text_family_request.rs"]
mod text_family_request;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let family = text_family_request::text_family(args.get(3).map(String::as_str))?;
    let request = if args.len() == 4 {
        &args[1..3]
    } else {
        &args[1..]
    };
    let (mode, prefix) =
        text_capture_request::parse_text_capture(request, &decal_scene::DECAL_MODES)?;
    capture_text_family(mode, prefix, family)
}
fn capture_text_family(
    mode: &str,
    prefix: &str,
    family: rustycad_tkservice::aspect::AspectTypeOfDisplayText,
) -> Result<(), Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = decal_scene::DecalScene::new(&driver, mode);
    scene.text.style.set_text_display_type(family);
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
