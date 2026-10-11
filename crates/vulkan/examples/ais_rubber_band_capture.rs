//! Martensite AIS_RubberBand capture through the live context and Vulkan 1.3 renderer.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ais_rubber_band_scene.rs"]
mod rubber_band_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&arguments, "rubber-rectangle")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = rubber_band_scene::RubberBandScene::new(&driver)?;
    request.configure(&scene.window, &scene.view);
    scene.apply(&request.mode, request.offset)?;
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        true,
    )?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    assert!(scene.render_view().capture().is_ok());
    scene.view.remove();
    Ok(())
}
