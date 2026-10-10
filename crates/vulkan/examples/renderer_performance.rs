//! Release-mode end-to-end V3d redraw/readback timings; validation is tested separately.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/viewer_scene.rs"]
mod scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments: Vec<String> = std::env::args().collect();
    let samples = count_argument(&arguments, 1, 100)?;
    let warmup = count_argument(&arguments, 2, 20)?;
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options)?;
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    for (name, draws, resize) in [
        ("single-triangle", 1, false),
        ("many-draws", 256, false),
        ("resize", 1, true),
    ] {
        benchmark(&driver, name, draws, resize, warmup, samples);
    }
    Ok(())
}
fn count_argument(
    arguments: &[String],
    index: usize,
    fallback: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    let count = arguments
        .get(index)
        .map_or(Ok(fallback), |value| value.parse::<usize>())?;
    if !(1..=10_000).contains(&count) {
        return Err(format!("received sample count {count}; expected 1..=10000").into());
    }
    Ok(count)
}
fn benchmark(
    driver: &rustycad_tkernel::standard::Handle<Graphic3dRenderDriver>,
    name: &str,
    draws: usize,
    resize: bool,
    warmup: usize,
    count: usize,
) {
    let scene = scene::ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for _ in 1..draws {
        scene.group.add_primitive_array(&scene.vertices, true);
    }
    for index in 0..warmup {
        timed_redraw(&scene, index, resize);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, index, resize))
        .collect();
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.structure.is_displayed());
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"draws\":{draws},\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_redraw(scene: &scene::ViewerScene, index: usize, resize: bool) -> f64 {
    let start = Instant::now();
    if resize {
        let width = if index.is_multiple_of(2) { 128 } else { 256 };
        scene.window.set_size(width, 256);
        scene.view.must_be_resized();
    }
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
