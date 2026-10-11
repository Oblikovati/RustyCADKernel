//! Capture native annotation brackets through the production Martensite driver.
#[path = "../tests/support/capping_scene.rs"]
mod capping_scene;
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/group_flipping_scene.rs"]
mod group_flipping_scene;
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "flip-back")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = scene::ViewerScene::new(&driver);
    request.configure(&scene.window, &scene.view);
    group_flipping_scene::apply(&scene, &request.mode, request.offset);
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        false,
    )?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.render_view().capture().is_ok());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
    Ok(())
}
