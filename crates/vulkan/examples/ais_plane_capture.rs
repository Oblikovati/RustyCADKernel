//! Capture AIS_Plane geometry and native transparent shading through Vulkan 1.3.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ais_plane_scene.rs"]
mod plane_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "aisplane-wire")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = plane_scene::AisPlaneScene::new(&driver, &request.mode, request.offset);
    request.configure(&scene.window, &scene.view);
    if request.mode == "aisplane-detected" {
        scene.detect();
    }
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        request.mode == "aisplane-detected",
    )?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
    Ok(())
}
