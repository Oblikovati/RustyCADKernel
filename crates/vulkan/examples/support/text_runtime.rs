//! Real-font text must produce glyph coverage and preserve annotation depth policy.
use super::text_scene::{MODES, TextScene};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dCViewMembers;
pub fn check_text_lifecycle() {
    let driver = Graphic3dRenderDriver::new(super::device_options::validation_options()).unwrap();
    for mode in MODES {
        let capture = capture_mode(&driver, mode);
        let foreground = capture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| **pixel != [0, 0, 255, 255])
            .count();
        assert_eq!(
            foreground == 0,
            mode == "text-empty",
            "received {foreground} pixels for {mode}"
        );
    }
    check_annotation(&driver);
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
}
fn capture_mode(driver: &Handle<Graphic3dRenderDriver>, mode: &str) -> FrameCapture {
    let scene = TextScene::new(driver);
    scene.apply(mode);
    // Later empty overlay layers clear depth; inspect the same default layer as native captures.
    scene
        .scene
        .cview()
        .set_z_layer_target(rustycad_tkservice::graphic3d::GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.scene.cview().set_z_layer_redraw_mode(true);
    scene.scene.render_view().try_redraw().unwrap();
    let capture = scene.scene.render_view().capture().unwrap();
    scene.scene.view.remove();
    capture
}
fn check_annotation(driver: &Handle<Graphic3dRenderDriver>) {
    let occluded = capture_mode(driver, "text-depth");
    let annotation = capture_mode(driver, "text-annotation");
    assert_ne!(occluded.rgba, annotation.rgba);
    let plain = capture_mode(driver, "text-basic");
    assert_eq!(plain.rgba, capture_mode(driver, "text-cutoff").rgba);
    assert_ne!(plain.rgba, capture_mode(driver, "text-opaque").rgba);
    let covered = plain.depth.iter().filter(|z| **z < 1.).count();
    assert!(
        (20..250).contains(&covered),
        "received {covered}; expected sparse glyph coverage"
    );
    assert!(
        plain
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|p| p[0] > 0 && p[0] < 255)
    );
}
