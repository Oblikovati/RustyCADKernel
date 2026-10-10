//! Capture and hit-test the actual datum, including retained group appearance updates.
use super::point_scene;
use martensite_vulkan::Graphic3dRenderDriver;
pub fn check_point_datums() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    for mode in point_scene::MODES {
        check_datum_mode(&driver, mode);
    }
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
}
fn check_datum_mode(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    mode: &str,
) {
    let scene = point_scene::PointScene::new(driver);
    scene.apply(mode).unwrap();
    scene.render_view().try_redraw().unwrap();
    let capture = scene.render_view().capture().unwrap();
    let foreground = capture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p != [0, 0, 255, 255])
        .count();
    assert_eq!(
        foreground == 0,
        matches!(mode, "point-erased" | "point-empty"),
        "mode {mode}: received {foreground} foreground pixels"
    );
    if mode == "point-plus" {
        check_datum_pick(&scene);
    }
    scene.view.remove();
}
fn check_datum_pick(scene: &point_scene::PointScene) {
    scene.context.move_to(32, 32, &scene.view, false).unwrap();
    assert_eq!(
        scene.context.detected_interactive().object_address(),
        scene.point.object_address()
    );
}
