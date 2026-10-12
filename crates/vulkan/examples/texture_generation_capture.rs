//! Capture actual retained coordinate generation on either local Vulkan device.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/texture_generation_scene.rs"]
mod generation_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderTextureCoordinatePipeline};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 4 {
        return Err(format!(
            "received arguments={arguments:?}; expected programmable|fixed mode prefix"
        )
        .into());
    }
    let policy = match arguments[1].as_str() {
        "programmable" => RenderTextureCoordinatePipeline::Programmable,
        "fixed" => RenderTextureCoordinatePipeline::GeneratedCompatibility,
        other => return Err(format!("received path={other}; expected programmable|fixed").into()),
    };
    capture_generation(policy, &arguments[2], &arguments[3])
}
fn capture_generation(
    policy: RenderTextureCoordinatePipeline,
    mode: &str,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::with_texture_coordinate_pipeline(
        device_options::validation_options(),
        policy,
    )?;
    let fixture = generation_scene::TextureGenerationScene::new(&driver, mode);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(mode);
    view_capture::capture_view(&fixture.scene.view, 64, 64, prefix, false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
