//! Plane labels, live appearance and selection through the real Vulkan driver.
use super::plane_scene::{MODES, PlaneScene};
use martensite_vulkan::Graphic3dRenderDriver;
pub fn check_plane_datums() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    for mode in MODES {
        let scene = PlaneScene::new(&driver, mode);
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
            mode == "plane-erased",
            "mode {mode}: {foreground} foreground pixels"
        );
        if mode == "plane-detected" {
            assert_eq!(
                scene.context.detected_interactive().object_address(),
                scene.object.object_address()
            );
        }
        scene.view.remove();
    }
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
}
