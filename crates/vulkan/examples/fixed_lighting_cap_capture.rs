//! Real homogeneous section geometry under fixed lighting; `fixed_lighting_cap_capture cap-oblique-fixed-phong /tmp/cap`.
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/fixed_cap_lights.rs"]
mod fixed_cap_lights;
#[path = "support/fixed_capture_arguments.rs"]
mod fixed_capture_arguments;
#[allow(dead_code)]
#[path = "../tests/support/fixed_lighting_scene.rs"]
mod fixed_scene;
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
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let [mode, prefix] = fixed_capture_arguments::capture_arguments(std::env::args().collect())?;
    capture_fixed_cap(&mode, &prefix)
}
fn capture_fixed_cap(mode: &str, prefix: &str) -> Result<(), Box<dyn std::error::Error>> {
    let driver = fixed_scene::fixed_lighting_driver(device_options::validation_options())?;
    let fixture = capping_texture_scene::CappingTextureScene::new(&driver, mode);
    fixed_cap_lights::fixed_cap_light_source(&fixture.scene, mode);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(mode);
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
