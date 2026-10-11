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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for mode in texture_scene::modes::SURFACE_TEXTURE_MODES {
        measure_texture_workload(&driver, mode)?;
    }
    Ok(())
}
fn measure_texture_workload(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    mode: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let fixture = texture_scene::SurfaceTextureScene::with_mode(driver, mode);
    let render = fixture.scene.render_view();
    render.try_redraw()?;
    fixture.mutate(mode);
    for _ in 0..20 {
        render.try_redraw()?;
    }
    let samples = (0..100)
        .map(|_| {
            let start = std::time::Instant::now();
            render.try_redraw().unwrap();
            start.elapsed().as_secs_f64() * 1000.
        })
        .collect::<Vec<_>>();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"surface-texture-{mode}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    assert!(render.capture().is_ok());
    fixture.scene.view.remove();
    Ok(())
}
