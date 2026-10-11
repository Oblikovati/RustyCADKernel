//! Runtime checks cover mutations on one live view and device, beyond static oracle scenes.
use crate::view_setup;
#[path = "../../tests/support/viewer_scene.rs"]
pub(super) mod scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::*;
pub fn check_viewer_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    assert_center(&scene, [255, 0, 0, 255]);
    check_mutations(&scene);
    check_resize_and_clear(&scene);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    scene.view.remove();
    assert!(scene.render_view().capture().is_err());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn check_mutations(scene: &scene::ViewerScene) {
    scene.structure.set_visible(false);
    assert_center(scene, [0, 0, 255, 255]);
    scene.structure.set_visible(true);
    assert_center(scene, [255, 0, 0, 255]);
    scene.vertices.set_vertice(3, 0., -7., 0.).unwrap();
    assert_center(scene, [0, 0, 255, 255]);
    scene.vertices.set_vertice(3, 0., 8., 0.).unwrap();
    assert_center(scene, [255, 0, 0, 255]);
    scene.structure.erase();
    assert_center(scene, [0, 0, 255, 255]);
    scene.structure.display();
    assert_center(scene, [255, 0, 0, 255]);
}
fn assert_center(scene: &scene::ViewerScene, color: [u8; 4]) {
    scene.view.redraw();
    let capture = scene.render_view().capture().unwrap();
    assert_eq!(
        capture.pixel(capture.width / 2, capture.height / 2),
        Some(color)
    );
}

fn check_resize_and_clear(scene: &scene::ViewerScene) {
    scene.window.set_size(96, 48);
    scene.view.must_be_resized();
    assert_center(scene, [255, 0, 0, 255]);
    assert_eq!(scene.render_view().capture().unwrap().width, 96);
    assert_eq!(scene.viewer.active_views().len(), 1);
    scene.group.clear(false);
    assert_center(scene, [0, 0, 255, 255]);
}
