//! Capture a retained plane presentation through the Martensite production driver.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/plane_presentation_scene.rs"]
mod plane_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "prsplane-edges")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = plane_scene::PlanePresentationScene::new(&driver, &request.mode, request.offset);
    request.configure(&scene.window, &scene.view);
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        false,
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
