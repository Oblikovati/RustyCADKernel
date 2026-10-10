//! Release timings include acquisition, GPU rendering/blit, readback and queuing presentation.
use crate::window_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    for resize in [false, true] {
        benchmark_window(driver, warmup, count, resize);
    }
}
fn benchmark_window(
    driver: &Handle<Graphic3dRenderDriver>,
    warmup: usize,
    count: usize,
    resize: bool,
) {
    let window = window_scene::WindowScene::new(driver, 256, 256);
    for index in 0..warmup {
        timed_present(&window, index, resize);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_present(&window, index, resize))
        .collect();
    let name = if resize {
        "window-resize-present"
    } else {
        "window-present"
    };
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    window.scene.view.remove();
    assert!(window.scene.viewer.active_views().is_empty());
}
fn timed_present(window: &window_scene::WindowScene, index: usize, resize: bool) -> f64 {
    let start = Instant::now();
    if resize {
        let side = if index.is_multiple_of(2) { 256 } else { 128 };
        window.resize(side, side);
    }
    window.redraw();
    start.elapsed().as_secs_f64() * 1000.
}
