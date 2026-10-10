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
    if request.mode.starts_with("line-") {
        line_arrays::replace_lines(&*scene.group, &request.mode, 1);
    }
    if request.mode.starts_with("stdprs-") {
        stdprs_scene::replace_curve(&scene.structure, &request.mode, 1);
    }
    let transform = Transform3::translation_vector(Vector3::new(request.offset, 0., 0.));
    scene
        .structure
        .set_transformation(&Handle::new(Datum3::new(transform)));
    if request.mode.starts_with("hom-") {
        configure_homogeneous(&scene, &request.mode);
    }
    if request.mode.starts_with("cap-") {
        capping_scene::configure_capping(&scene, &request.mode);
    }
    if request.mode.starts_with("clip-") {
        clipping_scene::configure_clipping(&scene, &request.mode);
    }
    if request.mode.starts_with("pers-") {
        persistence_scene::configure_persistence(
            &scene.structure,
            &*scene.group,
            &scene.vertices,
            &scene.view.camera(),
            &request.mode,
        );
    }
    if request.mode.starts_with("animcam-") {
        camera_animation_scene::apply(&scene.view, &request.mode);
    }
    let _shadow = (request.mode == "cap-shadow").then(|| capping_scene::cap_shadow(&scene));
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

#[path = "../tests/support/line_arrays.rs"]
mod line_arrays;

#[path = "../tests/support/stdprs_scene.rs"]
mod stdprs_scene;

#[path = "../tests/support/clipping_scene.rs"]
mod clipping_scene;

#[path = "../tests/support/homogeneous_arrays.rs"]
mod homogeneous_arrays;
fn configure_homogeneous(scene: &scene::ViewerScene, mode: &str) {
    homogeneous_arrays::replace_homogeneous(&*scene.group, mode);
    if mode.ends_with("clipped") {
        let planes = Handle::new(Graphic3dSequenceOfHClipPlane::default());
        let equation = rustycad_tkernel::ncollection::Vec4::new(1., 0., 0., -1.);
        planes.append(Handle::new(
            Graphic3dClipPlane::from_equation(equation).unwrap(),
        ));
        scene.view.set_clip_planes(&planes);
    }
    scene.structure.calculate_bound_box();
}

#[path = "../tests/support/capping_scene.rs"]
mod capping_scene;

#[allow(dead_code)]
#[path = "../tests/support/camera_animation_scene.rs"]
mod camera_animation_scene;
