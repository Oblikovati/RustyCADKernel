//! Capture the same real-font plane/culling cases as the native OpenGL oracle.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/text_culling_scene.rs"]
mod text_culling_scene;
#[path = "../tests/support/text_scene.rs"]
mod text_scene;
#[path = "support/view_capture.rs"]
mod view_capture;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let (mode, prefix) = parse_culling_arguments(&args[1..])?;
    let driver = Graphic3dRenderDriver::new(device_options::validation_options())?;
    let scene = text_culling_scene::TextCullingScene::new(&driver, mode);
    scene.text.scene.window.set_size(128, 128);
    scene.text.scene.view.must_be_resized();
    view_capture::capture_view(&scene.text.scene.view, 128, 128, prefix, false)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    assert!(
        driver.validation_errors().is_empty(),
        "{:?}",
        driver.validation_errors()
    );
    scene.text.scene.view.remove();
    Ok(())
}

fn parse_culling_arguments(
    args: &[String],
) -> Result<(&str, &str), martensite_vulkan::RenderError> {
    if args.len() != 2 {
        return Err(martensite_vulkan::RenderError::rejected(
            format!("arguments {args:?}"),
            "mode prefix",
        ));
    }
    if !text_culling_scene::CULLING_MODES.contains(&args[0].as_str()) {
        return Err(martensite_vulkan::RenderError::rejected(
            format!("mode {:?}", args[0]),
            "one of CULLING_MODES",
        ));
    }
    Ok((&args[0], &args[1]))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_mode_and_argument_shape_are_explicit() {
        let valid = vec!["front-back".to_owned(), "capture".to_owned()];
        assert_eq!(
            parse_culling_arguments(&valid).unwrap(),
            ("front-back", "capture")
        );
        assert!(parse_culling_arguments(&[]).is_err());
        assert!(parse_culling_arguments(&["invalid".to_owned(), "capture".to_owned()]).is_err());
    }
}
