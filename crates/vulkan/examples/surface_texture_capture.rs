//! Real sampled-image surfaces exercise ordinary retained Graphic3d interfaces on either device.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
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
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = texture_scene::SurfaceTextureScene::with_mode(&driver, &arguments[1]);
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
