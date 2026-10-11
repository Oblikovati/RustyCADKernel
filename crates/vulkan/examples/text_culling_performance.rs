//! Release-mode complete V3d redraw/readback distributions for native-matched text culling.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/text_culling_scene.rs"]
mod text_culling_scene;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;

#[path = "support/retained_text_performance.rs"]
mod retained_text_performance;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in text_culling_scene::CULLING_MODES {
        benchmark_culling_case(&driver, mode);
    }
    Ok(())
}
fn benchmark_culling_case(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    mode: &str,
) {
    let fixture = text_culling_scene::TextCullingScene::new(driver, mode);
    fixture.text.scene.window.set_size(128, 128);
    fixture.text.scene.view.must_be_resized();
    retained_text_performance::measure_text_view(&fixture.text.scene.render_view(), mode);
    fixture.text.scene.view.remove();
}
