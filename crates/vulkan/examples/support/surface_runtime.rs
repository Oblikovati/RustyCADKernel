//! Live bound colors and connectivity must reach actual GPU readback with validation enabled.
use super::viewer_runtime::scene;
#[path = "../../tests/support/surface_arrays.rs"]
mod surface_arrays;
use martensite_vulkan::Graphic3dRenderDriver;
pub fn check_surface_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    let array = surface_arrays::replace_surface(&*scene.group, "surface-strip-indexed", 1);
    assert_surface(&scene, [255, 0, 0, 255]);
    array.set_bound_color_rgb(1, 0., 1., 1.).unwrap();
    assert_surface(&scene, [0, 255, 255, 255]);
    array.bounds().set_bound(0, 0);
    array.bounds().set_bound(1, 0);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(
        scene.render_view().capture().unwrap().pixel(16, 32),
        Some([0, 0, 255, 255])
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
}
fn assert_surface(scene: &scene::ViewerScene, left: [u8; 4]) {
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert_eq!(capture.pixel(16, 32), Some(left));
    assert_eq!(capture.pixel(48, 32), Some([0, 255, 0, 255]));
    assert_eq!(capture.pixel(32, 32), Some([0, 0, 255, 255]));
}
