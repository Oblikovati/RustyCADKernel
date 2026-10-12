//! Capture real ViewCube presentation geometry and oriented labels with native display defaults.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "support/view_capture.rs"]
mod view_capture;
#[allow(dead_code)]
#[path = "../tests/support/view_cube_scene.rs"]
mod view_cube_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_cube_widget.rs"]
mod view_cube_widget;
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
    capture_widget_interaction(&scene, prefix)?;
    view_cube_widget::apply_widget_appearance(&scene)?;
    view_capture::capture_view(&scene.view, 320, 320, &format!("{prefix}-appearance"), true)?;
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

fn capture_widget_interaction(
    scene: &view_cube_scene::ViewCubeCaptureScene,
    prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use rustycad_tkv3d::ais::AisViewCubeMembers;
    scene.restore_labels()?;
    let clock = view_cube_widget::configure_widget_timer(scene);
    let orientation = view_cube_widget::detect_widget_center(scene)?;
    println!("MARTENSITE_WIDGET_ORIENTATION={orientation}");
    view_cube_widget::trace_widget_view("source", scene)?;
    view_capture::capture_view(&scene.view, 320, 320, &format!("{prefix}-hover"), true)?;
    view_cube_widget::click_widget_center(scene)?;
    view_cube_widget::trace_widget_camera("start", &scene.cube.view_animation().camera_start());
    view_cube_widget::trace_widget_camera("end", &scene.cube.view_animation().camera_end());
    capture_widget_frames(scene, prefix, &clock)
}

fn capture_widget_frames(
    scene: &view_cube_scene::ViewCubeCaptureScene,
    prefix: &str,
    clock: &view_cube_widget::CubeWidgetClock,
) -> Result<(), Box<dyn std::error::Error>> {
    use rustycad_tkv3d::ais::{AisViewCubeMembers, AisViewCubeSource};
    for (label, time) in view_cube_widget::widget_animation_frames() {
        clock.seek(time);
        let continuing = scene.cube.update_animation(false)?;
        println!(
            "MARTENSITE_WIDGET_FRAME={label},{continuing},{}",
            scene.cube.has_animation()?
        );
        view_cube_widget::trace_widget_view(label, scene)?;
        view_capture::capture_view(&scene.view, 320, 320, &format!("{prefix}-{label}"), true)?;
        view_cube_widget::trace_widget_view(&format!("{label}-after-dump"), scene)?;
    }
    Ok(())
}
