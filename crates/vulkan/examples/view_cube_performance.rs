//! Completed-frame timings for the real default ViewCube presentation, without validation overhead.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/view_cube_scene.rs"]
mod view_cube_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_cube_widget.rs"]
mod view_cube_widget;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, Graphic3dRenderView};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    let scene = view_cube_scene::ViewCubeCaptureScene::new(&driver)?;
    let render = scene.render_view();
    let samples = measure_cube_frames(&render);
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    print_cube_samples("view-cube-default", &samples);
    scene.enable_lights()?;
    print_cube_samples("view-cube-lit", &measure_cube_frames(&render));
    measure_widget_frames(&scene)?;
    view_cube_widget::apply_widget_appearance(&scene)?;
    print_cube_samples("view-cube-appearance", &measure_cube_frames(&render));
    assert!(render.capture().is_ok());
    scene.view.remove();
    Ok(())
}
fn print_cube_samples(name: &str, samples: &[f64]) {
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
}

fn measure_cube_frames(render: &Graphic3dRenderView) -> Vec<f64> {
    for _ in 0..20 {
        render.try_redraw().unwrap();
    }
    (0..100)
        .map(|_| {
            let start = std::time::Instant::now();
            render.try_redraw().unwrap();
            start.elapsed().as_secs_f64() * 1000.
        })
        .collect()
}

#[cfg(test)]
#[path = "../tests/support/fixed_glyph_source.rs"]
mod fixed_glyph_source;
#[cfg(test)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;

fn measure_widget_frames(
    scene: &view_cube_scene::ViewCubeCaptureScene,
) -> Result<(), Box<dyn std::error::Error>> {
    use rustycad_tkv3d::ais::AisViewCubeSource;
    let clock = view_cube_widget::configure_widget_timer(scene);
    view_cube_widget::detect_widget_center(scene)?;
    print_cube_samples(
        "view-cube-hover",
        &measure_cube_frames(&scene.render_view()),
    );
    view_cube_widget::click_widget_center(scene)?;
    for (name, time) in [
        ("view-cube-animation-quarter", 0.125),
        ("view-cube-animation-half", 0.25),
        ("view-cube-animation-end", 0.5),
    ] {
        clock.seek(time);
        scene.cube.update_animation(false)?;
        print_cube_samples(name, &measure_cube_frames(&scene.render_view()));
    }
    Ok(())
}
