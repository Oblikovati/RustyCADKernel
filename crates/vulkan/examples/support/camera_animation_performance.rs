//! Measure camera interpolation, invalidation and production GPU redraw/readback together.
use super::{camera_animation_scene, scene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::ais::{AisAnimationCamera, AisAnimationSource};
use std::time::Instant;
/// Collect complete camera-update frame timings; `benchmark(&driver, 20, 100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = scene::ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    let animation = camera_animation_scene::animation(&scene.view);
    animation.start(false).unwrap();
    for index in 0..warmup {
        timed_camera(&scene, &animation, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_camera(&scene, &animation, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"camera-animation-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_camera(scene: &scene::ViewerScene, animation: &AisAnimationCamera, index: usize) -> f64 {
    let start = Instant::now();
    animation
        .update(if index.is_multiple_of(2) { 0.5 } else { 1.5 })
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
