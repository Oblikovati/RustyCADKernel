//! Native AIS_PlaneTrihedron capture through the interactive context and Martensite Vulkan renderer.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ais_line_scene.rs"]
mod line_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkmath::gp::{Transform3, Vector3};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObjectMembers;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "plane-base")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = plane_scene::PlaneScene::new(&driver, &request.mode);
    request.configure(&scene.window, &scene.view);
    scene
        .object
        .set_local_transformation(&Transform3::translation_vector(Vector3::new(
            request.offset,
            0.,
            0.,
        )))?;
    scene.apply(&request.mode)?;
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        request.mode == "plane-detected",
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

#[path = "../tests/support/plane_trihedron_scene.rs"]
mod plane_scene;
