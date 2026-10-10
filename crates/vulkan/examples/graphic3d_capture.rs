//! Viewer-driven capture for comparison with the pinned OpenGL renderer.
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::{
    gp::{Transform3, Vector3},
    toploc::Datum3,
};
use rustycad_tkservice::graphic3d::*;
#[path = "support/view_capture.rs"]
mod view_capture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let request = capture_request::ViewerCaptureRequest::parse(&arguments, "default")?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = scene::ViewerScene::new(&driver);
    request.configure(&scene.window, &scene.view);
    if request.mode.starts_with("surface-") {
        surface_arrays::replace_surface(&*scene.group, &request.mode, 1);
    }
    let transform = Transform3::translation_vector(Vector3::new(request.offset, 0., 0.));
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
    if request.mode.starts_with("pers-") {
        persistence_scene::configure_persistence(
            &scene.structure,
            &*scene.group,
            &scene.vertices,
            &scene.view.camera(),
            &request.mode,
        );
    }
    view_capture::capture_view(
        &scene.view,
        request.width,
        request.height,
        &request.prefix,
        request.mode == "full",
    )?;
    check_capture(&driver, &scene);
    Ok(())
}

fn check_capture(driver: &Graphic3dRenderDriver, scene: &scene::ViewerScene) {
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    println!("PASS: V3d viewer capture; validation errors: 0");
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(!scene.group.is_null() && scene.vertices.vertex_number() == 3);
    assert!(scene.render_view().capture().is_ok());
}

#[path = "support/device_options.rs"]
mod device_options;

#[path = "support/capture_request.rs"]
mod capture_request;

#[path = "../tests/support/surface_arrays.rs"]
mod surface_arrays;

#[path = "../tests/support/persistence_scene.rs"]
mod persistence_scene;
