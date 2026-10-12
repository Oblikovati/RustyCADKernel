//! Complete concrete AIS presentation redraw/readback timings, measured only under a coordinated slot.
#[path = "../tests/support/ais_colored_shape_scene.rs"]
mod colored_scene;
#[allow(dead_code)]
#[path = "support/retained_text_performance.rs"]
mod complete_operation;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in colored_scene::COLORED_CAPTURE_MODES {
        measure_colored_shape(&driver, mode)?;
    }
    Ok(())
}
fn measure_colored_shape(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = colored_scene::AisColoredShapeScene::with_mode(driver, mode)?;
    let render = fixture.scene.render_view();
    render.try_redraw()?;
    fixture.mutate(mode)?;
    // Reuse the existing complete-operation contract without changing warmups or sample counts.
    complete_operation::measure_text_operation(&format!("ais-colored-shape-{mode}"), &mut || {
        render.try_redraw().unwrap()
    });
    assert!(render.capture().is_ok());
    fixture.scene.view.remove();
    Ok(())
}
