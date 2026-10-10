//! Capture the actual interactive-context lifecycle through Martensite's Vulkan driver.
#[path = "support/ais_lifecycle.rs"]
mod ais_lifecycle;
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ais_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&arguments, "ais-displayed")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = scene::AisViewerScene::new(&driver);
    request.configure(&scene.window, &scene.view);
    scene
        .object
        .set_local_transformation(&Transform3::translation_vector(Vector3::new(
            request.offset,
            0.,
            0.,
        )))?;
    ais_lifecycle::apply_capture_transition(&scene, &request.mode)?;
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
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.render_view().capture().is_ok());
    Ok(())
}
