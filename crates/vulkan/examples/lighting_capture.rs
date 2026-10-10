//! Native-comparable Phong-family view captures, including live retained changes.
#[path = "support/capture_request.rs"]
mod capture_request;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/lighting_scene.rs"]
mod lighting_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::Graphic3dStructureMembers;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&args, "lit-phong")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = lighting_scene::LightingScene::new(&driver, &request.mode);
    request.configure(&scene.base.window, &scene.base.view);
    scene
        .base
        .structure
        .set_transformation(&Handle::new(Datum3::new(Transform3::translation_vector(
            Vector3::new(request.offset, 0., 0.),
        ))));
    scene.base.view.redraw();
    scene.mutate(&request.mode);
    view_capture::capture_view(
        &scene.base.view,
        request.width,
        request.height,
        &request.prefix,
        false,
    )?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    scene.base.view.remove();
    Ok(())
}
