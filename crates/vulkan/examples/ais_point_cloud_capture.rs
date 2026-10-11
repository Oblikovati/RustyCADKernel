//! Capture production Martensite point cloud presentation and forced subset highlights.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "../tests/support/ais_point_cloud_scene.rs"]
mod cloud_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "cloud-points")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = cloud_scene::PointCloudScene::new(&driver, &request.mode, request.offset);
    request.configure(&scene.window, &scene.view);
    if request.mode.contains("subset") {
        scene.detect_subset(request.mode == "cloud-subset-selected");
    }
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        request.mode == "cloud-subset-detected",
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
