//! Actual context-owned Martensite XR presentations capture mesh, texture and ray draws.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[allow(dead_code)]
#[path = "../tests/support/xr_device_scene.rs"]
mod xr_scene;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    validate_xr_capture_arguments(&arguments)?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = xr_scene::XrDeviceScene::new(&driver, &arguments[1])?;
    fixture.render_view().try_redraw()?;
    fixture.mutate(&arguments[1])?;
    view_capture::capture_view(&fixture.view, 64, 64, &arguments[2], false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.view.remove();
    Ok(())
}

fn validate_xr_capture_arguments(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if arguments.len() != 3 {
        return Err(format!(
            "received arguments={arguments:?}; expected fixture mode and capture prefix"
        )
        .into());
    }
    Ok(())
}
