//! Real sampled-image surfaces exercise ordinary retained Graphic3d interfaces on either device.
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/device_options.rs"]
mod device_options;
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
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() != 3 {
        return Err(format!(
            "received arguments={arguments:?}; expected fixture mode and capture prefix"
        )
        .into());
    }
    let policy = if arguments[1].contains("generated") || arguments[1].contains("fixed") {
        martensite_vulkan::RenderTextureCoordinatePipeline::GeneratedCompatibility
    } else {
        martensite_vulkan::RenderTextureCoordinatePipeline::Programmable
    };
    let driver = Graphic3dRenderDriver::with_texture_coordinate_pipeline(
        device_options::validation_options(),
        policy,
    )?;
    let fixture = capping_texture_scene::CappingTextureScene::new(&driver, &arguments[1]);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(&arguments[1]);
    view_capture::capture_view(&fixture.scene.view, 64, 64, &arguments[2], false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
