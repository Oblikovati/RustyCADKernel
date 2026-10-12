//! Real standard2D textures under native fixed lighting; `fixed_lighting_texture_capture phong /tmp/fixed`.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/fixed_lighting_scene.rs"]
mod fixed_scene;
#[path = "../tests/support/fixed_texture_scene.rs"]
mod fixed_texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/lighting_scene.rs"]
mod lighting_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;

#[path = "support/fixed_capture_arguments.rs"]
mod fixed_capture_arguments;
fn capture_fixed_texture(mode: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let driver = fixed_scene::fixed_lighting_driver(device_options::validation_options())?;
    let fixture = fixed_texture_scene::fixed_texture_scene(&driver, mode);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(mode.strip_prefix("lit-").unwrap_or(mode));
    view_capture::capture_view(&fixture.scene.view, 64, 64, prefix, false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [mode, prefix] = fixed_capture_arguments::capture_arguments(std::env::args().collect())?;
    capture_fixed_texture(&mode, &prefix)
}
