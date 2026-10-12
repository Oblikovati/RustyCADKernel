//! Actual context-owned Martensite XR presentations measure retained redraw workloads.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
#[allow(dead_code)]
#[path = "../tests/support/xr_device_scene.rs"]
mod xr_scene;
use martensite_vulkan::Graphic3dRenderDriver;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in xr_scene::XR_CAPTURE_MODES {
        measure_xr_frames(&driver, mode, false)?;
    }
    for mode in ["finite-ray", "infinite-ray"] {
        measure_xr_frames(&driver, mode, true)?;
    }
    Ok(())
}
fn measure_xr_frames(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    mode: &str,
    updates: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = xr_scene::XrDeviceScene::new(driver, mode)?;
    let render = fixture.render_view();
    render.try_redraw()?;
    fixture.mutate(mode)?;
    let samples = xr_redraw_samples(&fixture, updates);
    let suffix = if updates { "-each-frame" } else { "" };
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"xr-device-{mode}{suffix}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    assert!(render.capture().is_ok());
    fixture.view.remove();
    Ok(())
}

fn xr_redraw_samples(fixture: &xr_scene::XrDeviceScene, updates: bool) -> Vec<f64> {
    let render = fixture.render_view();
    (0..120)
        .filter_map(|frame| {
            let start = std::time::Instant::now();
            refresh_measured_xr_ray(fixture, updates, frame);
            render.try_redraw().unwrap();
            (frame >= 20).then(|| start.elapsed().as_secs_f64() * 1000.)
        })
        .collect()
}

fn refresh_measured_xr_ray(fixture: &xr_scene::XrDeviceScene, updates: bool, frame: usize) {
    use rustycad_tkernel::quantity::{Color, NamedColor};
    use rustycad_tkv3d::ais::AisXrTrackedDeviceMembers;
    if !updates {
        return;
    }
    let color = if frame.is_multiple_of(2) {
        NamedColor::RED
    } else {
        NamedColor::GREEN
    };
    fixture.device.set_laser_color(Color::named(color)).unwrap();
}
