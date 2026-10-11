//! Capture one ordinary retained text family using the native default-layer viewport.
use super::{decal_scene, device_options};
use martensite_vulkan::{FrameCapture, Graphic3dRenderDriver};
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, graphic3d::*};
/// Capture all native glyph passes; `capture_decorated_text("decal-basic", AspectTypeOfDisplayText::Dekale)` enables validation.
pub fn capture_decorated_text(
    mode: &str,
    family: AspectTypeOfDisplayText,
) -> Result<FrameCapture, Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = decal_scene::DecalScene::new(&driver, mode);
    fixture.text.style.set_text_display_type(family);
    let view = &fixture.text.scene;
    view.window.set_size(128, 128);
    view.view.must_be_resized();
    view.cview().set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    view.cview().set_z_layer_redraw_mode(true);
    view.render_view().try_redraw()?;
    let capture = view.render_view().capture().unwrap();
    assert_text_capture_validation(&driver);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    view.view.remove();
    Ok(capture)
}

fn assert_text_capture_validation(driver: &Graphic3dRenderDriver) {
    assert!(
        driver.validation_errors().is_empty(),
        "validation {:?}; expected no Vulkan errors",
        driver.validation_errors()
    );
}
