//! Compare native quad assembly and transparent ordering through Vulkan captures.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/quadrangle_scene.rs"]
mod quad_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "quad-gradient")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = quad_scene::QuadrangleScene::new(&driver, &request.mode, request.offset);
    let scene = &fixture.scene;
    request.configure(&scene.window, &scene.view);
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        matches!(request.mode.as_str(), "quad-layers" | "quad-clear-depth"),
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
