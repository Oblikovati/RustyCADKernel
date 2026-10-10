//! Capture and hit-test the actual datum, including retained group appearance updates.
use super::line_scene;
use martensite_vulkan::Graphic3dRenderDriver;
pub fn check_line_datums() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    for mode in line_scene::MODES {
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
    let scene = line_scene::LineScene::new(driver, mode == "datum-infinite");
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
        mode == "datum-erased",
        "mode {mode}: received {foreground} foreground pixels"
    );
    if mode == "datum-line" {
        check_datum_pick(&scene);
    }
    scene.view.remove();
}
fn check_datum_pick(scene: &line_scene::LineScene) {
    scene.context.move_to(32, 32, &scene.view, false).unwrap();
    assert_eq!(
        scene.context.detected_interactive().object_address(),
        scene.line.object_address()
    );
}
