//! Complete redraw/readback frames alternate the live annotation viewing orientation.
use super::{group_flipping_scene, scene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Direction3;
use std::time::Instant;
/// Measure camera-dependent group commands; `benchmark(&driver,20,100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = scene::ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    group_flipping_scene::apply(&scene, "flip-nested", 0.);
    for index in 0..warmup {
        timed_flipping(&scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_flipping(&scene, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"group-flipping-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_flipping(scene: &scene::ViewerScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .view
        .camera()
        .set_up(&Direction3::new(0., if index.is_multiple_of(2) { 1. } else { -1. }, 0.).unwrap());
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
