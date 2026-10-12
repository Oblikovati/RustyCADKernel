//! Rust-only native replay shares retained and explicit pixel scene constructors.
use super::{blend_scene, decal_scene, device_options, pixel_scene};
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, graphic3d::*};
/// Capture one retained native scene; `capture_retained_xor("blend-colored")?` enables validation.
pub fn capture_retained_xor(mode: &str) -> Result<FrameCapture, Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = if blend_scene::BLEND_EXTRA_MODES.contains(&mode) {
        blend_scene::blend_extra_fixture(&driver, mode)
    } else {
        decal_scene::DecalScene::new(&driver, mode)
    };
    fixture
        .text
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    Ok(capture_xor_view(&fixture, &driver)?)
}
fn capture_xor_view(
    fixture: &decal_scene::DecalScene,
    driver: &Handle<Graphic3dRenderDriver>,
) -> Result<FrameCapture, RenderError> {
    let scene = &fixture.text.scene;
    scene.window.set_size(128, 128);
    scene.view.must_be_resized();
    scene
        .cview()
        .set_z_layer_target(GRAPHIC3D_ZLAYER_ID_DEFAULT);
    scene.cview().set_z_layer_redraw_mode(true);
    scene.render_view().try_redraw()?;
    let capture = scene.render_view().capture().unwrap();
    verify_xor_capture(driver)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    scene.view.remove();
    Ok(capture)
}
/// Capture caller-owned true2D input; `capture_pixel_xor("pixel-reuse-cached")?` keeps the native cached layout.
pub fn capture_pixel_xor(mode: &str) -> Result<FrameCapture, Box<dyn std::error::Error>> {
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = decal_scene::DecalScene::new(
        &driver,
        if mode.starts_with("pixel-reuse") {
            "decal-basic"
        } else {
            mode
        },
    );
    let frame = pixel_scene::pixel_case_frame(&fixture, mode)?;
    let mut renderer = VulkanRenderer::new(device_options::validation_options())?;
    let capture = renderer.render(&frame)?;
    assert!(renderer.validation_errors().is_empty());
    println!("MARTENSITE_DEVICE={}", renderer.device_name());
    fixture.text.scene.view.remove();
    Ok(capture)
}

fn verify_xor_capture(driver: &Graphic3dRenderDriver) -> Result<(), RenderError> {
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    Ok(())
}
