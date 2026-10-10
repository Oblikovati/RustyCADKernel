//! Live appearance mutation, retained AIS group updates and GPU redraw/readback.
use super::circle_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = circle_scene::CircleScene::new(driver, "circle-full");
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for i in 0..warmup {
        timed_circle(&scene, i);
    }
    let samples: Vec<f64> = (0..count).map(|i| timed_circle(&scene, i)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"circle-datum-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_circle(scene: &circle_scene::CircleScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .circle
        .circle()
        .set_radius(if index.is_multiple_of(2) { 4. } else { 6. })
        .unwrap();
    scene.redisplay().unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
