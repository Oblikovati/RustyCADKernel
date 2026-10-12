//! Capture real AIS_TexturedShape presentations; `cargo run --example ais_textured_shape_capture -- basic /tmp/shape`.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/ais_textured_shape_scene.rs"]
mod textured_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = textured_capture_arguments(std::env::args().collect())?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let fixture = textured_scene::AisTexturedShapeScene::with_mode(&driver, &arguments[1])?;
    fixture.scene.render_view().try_redraw()?;
    fixture.mutate(&arguments[1])?;
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
fn textured_capture_arguments(
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
fn textured_capture_argument_validation_preserves_received_values() {
    let valid = vec!["capture".into(), "basic".into(), "prefix".into()];
    assert_eq!(textured_capture_arguments(valid.clone()).unwrap(), valid);
    let failure = textured_capture_arguments(vec!["bad".into()]).unwrap_err();
    assert!(failure.to_string().contains("bad"));
    assert!(failure.to_string().contains("mode and capture prefix"));
}
