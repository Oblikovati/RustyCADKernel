//! Capture actual fixed-lighting retained views; `fixed_lighting_capture lit-basic /tmp/fixed`.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/fixed_capture_arguments.rs"]
mod fixed_capture_arguments;
#[path = "../tests/support/fixed_lighting_scene.rs"]
mod fixed_scene;
#[allow(dead_code)]
#[path = "../tests/support/lighting_scene.rs"]
mod lighting_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [mode, prefix] = fixed_capture_arguments::capture_arguments(std::env::args().collect())?;
    let driver = fixed_scene::fixed_lighting_driver(device_options::validation_options())?;
    let fixture = fixed_scene::fixed_lighting_scene(&driver, &mode);
    fixture.base.render_view().try_redraw()?;
    fixture.mutate(&mode);
    view_capture::capture_view(&fixture.base.view, 64, 64, &prefix, false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.base.view.remove();
    Ok(())
}
