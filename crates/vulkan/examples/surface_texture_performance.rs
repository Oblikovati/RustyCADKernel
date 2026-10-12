//! Real retained standard2D redraw/readback timings cover uploaded resources and live updates.
#[path = "support/device_options.rs"]
mod device_options;
#[allow(dead_code)]
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
#[allow(dead_code)]
#[path = "../tests/support/surface_texture_scene.rs"]
mod texture_scene;
#[allow(dead_code)]
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::Graphic3dTextureRootMembers;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in texture_scene::modes::SURFACE_TEXTURE_MODES {
        measure_texture_workload(&driver, mode, false)?;
    }
    for mode in ["image-updated", "sampler-updated"] {
        measure_texture_workload(&driver, mode, true)?;
    }
    Ok(())
}
fn measure_texture_workload(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    mode: &str,
    live_updates: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = texture_scene::SurfaceTextureScene::with_mode(driver, mode);
    let render = fixture.scene.render_view();
    render.try_redraw()?;
    fixture.mutate(mode);
    let samples = measure_texture_frames(&fixture, mode, live_updates);
    let suffix = if live_updates { "-each-frame" } else { "" };
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"surface-texture-{mode}{suffix}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    assert!(render.capture().is_ok());
    fixture.scene.view.remove();
    Ok(())
}

fn measure_texture_frames(
    fixture: &texture_scene::SurfaceTextureScene,
    mode: &str,
    live_updates: bool,
) -> Vec<f64> {
    let render = fixture.scene.render_view();
    (0..120)
        .filter_map(|frame| {
            let start = std::time::Instant::now();
            refresh_timed_texture(fixture, mode, live_updates, frame);
            render.try_redraw().unwrap();
            (frame >= 20).then(|| start.elapsed().as_secs_f64() * 1000.)
        })
        .collect()
}

fn refresh_timed_texture(
    fixture: &texture_scene::SurfaceTextureScene,
    mode: &str,
    live_updates: bool,
    frame: usize,
) {
    if !live_updates {
        return;
    }
    if mode == "image-updated" {
        fixture.mutate(mode);
        return;
    }
    let filter = if frame.is_multiple_of(2) {
        rustycad_tkservice::graphic3d::Graphic3dTypeOfTextureFilter::Nearest
    } else {
        rustycad_tkservice::graphic3d::Graphic3dTypeOfTextureFilter::Bilinear
    };
    fixture.texture.get_params().set_filter(filter);
    fixture.scene.group.synchronize_aspects();
}
