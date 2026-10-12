//! Actual retained fixed-lighting redraw/readback timings, with twenty warmups and one hundred samples.
#[path = "../tests/support/fixed_caps_assets_0.rs"]
mod cap_assets;
#[allow(dead_code)]
#[path = "../tests/support/capping_texture_scene.rs"]
mod capping_texture_scene;
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/fixed_cap_lights.rs"]
mod fixed_cap_lights;
#[allow(dead_code)]
#[path = "../tests/support/fixed_lighting_scene.rs"]
mod fixed_scene;
#[path = "../tests/support/fixed_texture_scene.rs"]
mod fixed_texture_scene;
#[path = "support/fixed_lighting_updates.rs"]
mod fixed_updates;
#[path = "../tests/support/fixed_lighting_workload.rs"]
mod fixed_workload;
#[path = "../tests/support/fixed_lighting_assets_0.rs"]
mod lighting_assets_0;
#[path = "../tests/support/fixed_lighting_assets_1.rs"]
mod lighting_assets_1;
#[path = "../tests/support/fixed_lighting_assets_2.rs"]
mod lighting_assets_2;
#[allow(dead_code)]
#[path = "../tests/support/lighting_scene.rs"]
mod lighting_scene;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[path = "../tests/support/fixed_textures_assets_0.rs"]
mod texture_assets;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::{Graphic3dRenderDriver, RenderError};
use rustycad_tkernel::standard::Handle;
type NativeFixedLightingAsset = (
    &'static str,
    bool,
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [u8],
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = fixed_scene::fixed_lighting_driver(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for &(mode, texture, _, _, _, _) in fixed_assets() {
        measure_fixed_workload(&driver, mode, texture, false)?;
    }
    for &(mode, texture) in fixed_updates::LIVE_WORKLOADS {
        measure_fixed_workload(&driver, mode, texture, true)?;
    }
    Ok(())
}
fn fixed_assets() -> impl Iterator<Item = &'static NativeFixedLightingAsset> {
    lighting_assets_0::ASSETS
        .iter()
        .chain(lighting_assets_1::ASSETS)
        .chain(lighting_assets_2::ASSETS)
        .chain(texture_assets::ASSETS)
        .chain(cap_assets::ASSETS)
}
fn measure_fixed_workload(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
    texture: bool,
    live: bool,
) -> Result<(), RenderError> {
    let workload = fixed_workload::FixedLightingWorkload::new(driver, mode, texture);
    workload.scene().render_view().try_redraw()?;
    workload.mutate(mode);
    let samples = measure_fixed_frames(&workload, mode, live)?;
    let family = if texture { "texture-" } else { "" };
    let suffix = if live { "-each-frame" } else { "" };
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"fixed-{family}{mode}{suffix}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    workload.scene().view.remove();
    Ok(())
}
fn measure_fixed_frames(
    workload: &fixed_workload::FixedLightingWorkload,
    mode: &str,
    live: bool,
) -> Result<Vec<f64>, RenderError> {
    let mut samples = Vec::with_capacity(100);
    for frame in 0..120 {
        let start = std::time::Instant::now();
        if live {
            fixed_updates::refresh_fixed_workload(workload, mode, frame);
        }
        workload.scene().render_view().try_redraw()?;
        if frame >= 20 {
            samples.push(start.elapsed().as_secs_f64() * 1000.);
        }
    }
    assert!(workload.scene().render_view().capture().is_ok());
    Ok(samples)
}

#[cfg(test)]
#[allow(dead_code)]
#[path = "../tests/support/recording_device.rs"]
mod recording_device;
