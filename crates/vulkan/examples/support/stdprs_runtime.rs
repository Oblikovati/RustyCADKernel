//! Rebuilding standard presentations replaces live groups and reaches actual GPU readback.
use super::viewer_runtime::scene;
#[path = "../../tests/support/stdprs_scene.rs"]
mod stdprs_scene;
use martensite_vulkan::Graphic3dRenderDriver;
pub fn check_curve_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    let scene = scene::ViewerScene::new(&driver);
    for mode in stdprs_scene::MODES {
        stdprs_scene::replace_curve(&scene.structure, mode, 1);
        scene.render_view().try_redraw().unwrap();
        let capture = scene.render_view().capture().unwrap();
        let foreground = capture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| **pixel != [0, 0, 255, 255])
            .count();
        assert_eq!(
            foreground == 0,
            mode == "stdprs-collect",
            "mode {mode}: unexpected foreground count {foreground}"
        );
    }
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.view.remove();
}
