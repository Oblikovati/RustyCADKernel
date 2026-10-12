//! Capture the explicit true2D API against actual native OpenGl_Text::Set2D.
#[path = "../tests/support/decal_scene.rs"]
mod decal_scene;
#[path = "support/device_options.rs"]
mod device_options;
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
use martensite_vulkan::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err(format!("received {args:?}; expected mode prefix").into());
    }
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let reuse = args[1].starts_with("pixel-reuse");
    let mode = if reuse { "decal-basic" } else { &args[1] };
    let scene = decal_scene::DecalScene::new(&driver, mode);
    capture_pixel_fixture(&scene, &args[1], &args[2])
}
fn capture_pixel_fixture(
    scene: &decal_scene::DecalScene,
    mode: &str,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut renderer = VulkanRenderer::new(device_options::validation_options())?;
    let frame = pixel_scene::pixel_case_frame(scene, mode)?;
    let capture = renderer.render(&frame)?;
    write_pixel_capture(&capture, prefix)?;
    assert!(
        renderer.validation_errors().is_empty(),
        "{:?}",
        renderer.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    Ok(())
}
fn write_pixel_capture(capture: &FrameCapture, prefix: &str) -> std::io::Result<()> {
    std::fs::write(format!("{prefix}.rgba"), &capture.rgba)?;
    let bytes: Vec<u8> = capture.depth.iter().flat_map(|v| v.to_ne_bytes()).collect();
    std::fs::write(format!("{prefix}.depth"), bytes)?;
    Ok(())
}
