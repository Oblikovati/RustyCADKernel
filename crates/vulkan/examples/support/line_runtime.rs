//! Live widths, empty patterns and mixed topologies must reach actual validated readback.
use super::viewer_runtime::scene;
#[path = "../../tests/support/line_arrays.rs"]
mod line_arrays;
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkservice::graphic3d::*;
pub fn check_line_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    line_arrays::replace_lines(&*scene.group, "line-polyline-indexed", 1);
    scene.render_view().try_redraw().unwrap();
    let thin = ink(&scene.render_view().capture().unwrap());
    assert!(thin > 0);
    scene.group.aspects().set_line_width(3.).unwrap();
    scene.render_view().try_redraw().unwrap();
    assert!(ink(&scene.render_view().capture().unwrap()) > thin);
    check_empty_and_mixed(&scene);
    check_persistent_lines(&scene);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
}
fn check_empty_and_mixed(scene: &scene::ViewerScene) {
    scene.group.aspects().set_line_pattern(0);
    scene.render_view().try_redraw().unwrap();
    assert_eq!(ink(&scene.render_view().capture().unwrap()), 0);
    scene.group.add_primitive_array(&scene.vertices, true);
    scene.render_view().try_redraw().unwrap();
    let triangles = ink(&scene.render_view().capture().unwrap());
    assert!(triangles > 0);
    scene.group.aspects().set_line_pattern(0xffff);
    scene.render_view().try_redraw().unwrap();
    assert!(ink(&scene.render_view().capture().unwrap()) > triangles);
}
fn ink(capture: &FrameCapture) -> usize {
    capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|pixel| **pixel != [0, 0, 255, 255])
        .count()
}

fn check_persistent_lines(scene: &scene::ViewerScene) {
    use rustycad_tkernel::standard::Handle;
    let array = line_arrays::replace_lines(&*scene.group, "line-polyline", 1);
    for index in 1..=array.vertex_number() {
        let vertex = array.vertice(index).unwrap();
        array
            .set_vertice(index, vertex.x() as f32, vertex.y() as f32, -0.5)
            .unwrap();
    }
    scene.structure.set_transform_persistence(
        &Handle::new(Graphic3dTransformPers::new(Graphic3dTransModeFlags::CAMERA_PERS).unwrap())
            .cast(),
    );
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    assert!(ink(&capture) > 0);
    for (index, pixel) in capture.rgba.as_chunks::<4>().0.iter().enumerate() {
        if *pixel != [0, 0, 255, 255] {
            assert_eq!(capture.depth[index], 0.);
        }
    }
}
