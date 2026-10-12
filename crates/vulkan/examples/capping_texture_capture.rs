//! Real sampled-image surfaces exercise ordinary retained Graphic3d interfaces on either device.
#[path = "../tests/support/capping_texture_policy.rs"]
#[allow(dead_code)]
mod cap_policy;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/device_options.rs"]
mod device_options;
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
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = validate_capture_arguments(std::env::args().collect())?;
    let driver = capture_driver(&arguments[1])?;
    let fixture = capping_texture_scene::CappingTextureScene::new(&driver, &arguments[1]);
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(&arguments[1]);
    view_capture::capture_view(&fixture.scene.view, 64, 64, &arguments[2], false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
fn validate_capture_arguments(arguments: Vec<String>) -> Result<Vec<String>, String> {
    if arguments.len() != 3 {
        return Err(format!(
            "received arguments={arguments:?}; expected fixture mode and capture prefix"
        ));
    }
    Ok(arguments)
}
fn capture_driver(
    mode: &str,
) -> Result<rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>, martensite_vulkan::RenderError>
{
    let (policy, edges) = cap_policy::cap_fixture_pipelines(mode);
    Graphic3dRenderDriver::with_surface_pipelines(
        device_options::validation_options(),
        policy,
        edges,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_arguments_require_mode_and_prefix_and_preserve_values() {
        let valid = vec!["capture".into(), "cap-oblique".into(), "/tmp/cap".into()];
        assert_eq!(validate_capture_arguments(valid.clone()).unwrap(), valid);
        let invalid = vec!["capture".into(), "cap-oblique".into()];
        let error = validate_capture_arguments(invalid).unwrap_err();
        assert!(error.contains("cap-oblique"));
        assert!(error.contains("expected fixture mode and capture prefix"));
    }
}
