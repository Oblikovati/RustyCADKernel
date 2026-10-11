//! Capture real ViewCube presentation geometry and oriented labels with native display defaults.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_cube_adapter.rs"]
mod view_cube_adapter;
#[allow(dead_code)]
#[path = "../tests/support/view_cube_scene.rs"]
mod view_cube_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let prefix = cube_capture_prefix(&arguments)?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = view_cube_scene::ViewCubeCaptureScene::new(&driver)?;
    view_capture::capture_view(&scene.view, 320, 320, prefix, true)?;
    capture_without_labels(&scene, prefix)?;
    capture_lit_cube(&scene, prefix)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    scene.view.remove();
    Ok(())
}

fn cube_capture_prefix(arguments: &[String]) -> Result<&str, Box<dyn std::error::Error>> {
    if arguments.len() != 2 {
        return Err(
            format!("received arguments={arguments:?}; expected one capture prefix").into(),
        );
    }
    Ok(&arguments[1])
}
fn capture_without_labels(
    scene: &view_cube_scene::ViewCubeCaptureScene,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    scene.hide_labels()?;
    let hidden = format!("{prefix}-unlabeled");
    view_capture::capture_view(&scene.view, 320, 320, &hidden, true)?;
    let visible = std::fs::read(format!("{prefix}.rgba"))?;
    let invisible = std::fs::read(format!("{hidden}.rgba"))?;
    assert!(
        visible != invisible,
        "ViewCube label captures prefix={prefix}; expected changed pixels after clearing label strings"
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn capture_prefix_parser_reports_invalid_argument_shapes() {
        assert_eq!(
            super::cube_capture_prefix(&["probe".into(), "capture".into()]).unwrap(),
            "capture"
        );
        assert!(
            super::cube_capture_prefix(&[])
                .unwrap_err()
                .to_string()
                .contains("expected one")
        );
    }
}

#[cfg(test)]
#[path = "../tests/support/fixed_glyph_source.rs"]
mod fixed_glyph_source;
#[cfg(test)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;

fn capture_lit_cube(
    scene: &view_cube_scene::ViewCubeCaptureScene,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    scene.restore_labels()?;
    scene.enable_lights()?;
    let lit = format!("{prefix}-lit");
    view_capture::capture_view(&scene.view, 320, 320, &lit, true)?;
    capture_without_labels(scene, &lit)
}
