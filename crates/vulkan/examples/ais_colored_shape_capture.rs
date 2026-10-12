//! Capture real AIS_ColoredShape presentations; `cargo run --example ais_colored_shape_capture -- custom-color /tmp/shape`.
#[path = "../tests/support/ais_colored_shape_scene.rs"]
mod colored_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = colored_capture_arguments(std::env::args().collect())?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = colored_scene::AisColoredShapeScene::with_mode(&driver, &arguments[1])?;
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(&arguments[1])?;
    view_capture::capture_view(&fixture.scene.view, 128, 128, &arguments[2], false)?;
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    fixture.scene.view.remove();
    Ok(())
}
fn colored_capture_arguments(
    arguments: Vec<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if arguments.len() != 3 {
        return Err(
            format!("received arguments={arguments:?}; expected mode and capture prefix").into(),
        );
    }
    Ok(arguments)
}

#[test]
fn colored_capture_argument_validation_preserves_received_values() {
    let valid = vec!["capture".into(), "custom-color".into(), "prefix".into()];
    assert_eq!(colored_capture_arguments(valid.clone()).unwrap(), valid);
    let failure = colored_capture_arguments(vec!["bad".into()]).unwrap_err();
    assert!(failure.to_string().contains("bad"));
    assert!(failure.to_string().contains("mode and capture prefix"));
}
