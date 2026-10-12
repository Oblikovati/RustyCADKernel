//! Optimized native-family XOR distributions; strict functional checks run separately.
#[path = "../tests/support/blend_scene.rs"]
mod blend_scene;
#[path = "../tests/support/decal_scene.rs"]
mod decal_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/pixel_scene.rs"]
mod pixel_scene;
#[path = "support/pixel_text_performance.rs"]
mod pixel_text_performance;
#[path = "support/retained_text_performance.rs"]
mod retained_text_performance;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/subtitle_scene.rs"]
mod subtitle_scene;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::{aspect::AspectTypeOfDisplayText, graphic3d::Graphic3dAspectsTextMembers};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in decal_scene::DECAL_MODES
        .into_iter()
        .chain(blend_scene::BLEND_EXTRA_MODES)
    {
        benchmark_retained_blend(&driver, mode);
    }
    let mut renderer = VulkanRenderer::new(options)?;
    for mode in decal_scene::DECAL_MODES
        .into_iter()
        .chain(["pixel-reuse", "pixel-reuse-cached"])
    {
        benchmark_pixel_blend(&driver, &mut renderer, mode)?;
    }
    Ok(())
}
fn benchmark_retained_blend(driver: &Handle<Graphic3dRenderDriver>, mode: &str) {
    let fixture = if blend_scene::BLEND_EXTRA_MODES.contains(&mode) {
        blend_scene::blend_extra_fixture(driver, mode)
    } else {
        decal_scene::DecalScene::new(driver, mode)
    };
    fixture
        .text
        .style
        .set_text_display_type(AspectTypeOfDisplayText::Blend);
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    retained_text_performance::measure_text_view(
        &fixture.text.scene.render_view(),
        &format!("retained-xor-{mode}"),
    );
    fixture.text.scene.view.remove();
}
fn benchmark_pixel_blend(
    driver: &Handle<Graphic3dRenderDriver>,
    renderer: &mut VulkanRenderer,
    mode: &str,
) -> Result<(), RenderError> {
    let reuse = mode.starts_with("pixel-reuse");
    let fixture = decal_scene::DecalScene::new(driver, if reuse { "decal-basic" } else { mode });
    let frame = pixel_scene::pixel_case_frame(&fixture, mode)?;
    pixel_text_performance::measure_pixel_frame(
        renderer,
        &frame,
        &format!("explicit-pixel-xor-{mode}"),
    );
    fixture.text.scene.view.remove();
    Ok(())
}
