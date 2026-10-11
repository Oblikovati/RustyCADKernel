//! Production object placement, selection update and GPU redraw/readback timing.
use super::{ais_scene, object_animation_scene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::ais::{AisAnimationObject, AisAnimationSource};
use std::time::Instant;
/// Measure complete moving-object frames; `benchmark(&driver,20,100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ais_scene::AisViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    let animation = object_animation_scene::animation(&scene, 0.);
    animation.start(false).unwrap();
    for index in 0..warmup {
        timed_object(&scene, &animation, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_object(&scene, &animation, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"object-animation-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_object(
    scene: &ais_scene::AisViewerScene,
    animation: &AisAnimationObject,
    index: usize,
) -> f64 {
    let start = Instant::now();
    animation
        .update(if index.is_multiple_of(2) { 0.5 } else { 1.5 })
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
