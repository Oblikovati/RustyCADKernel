//! A thousand native sprites share one indexed GPU submission.
use super::point_scene::PointScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = PointScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    scene.batch().unwrap();
    for _ in 0..warmup {
        timed_points(&scene);
    }
    let samples: Vec<f64> = (0..count).map(|_| timed_points(&scene)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"point-sprites\",\"draws\":1,\"points\":1024,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_points(scene: &PointScene) -> f64 {
    let start = Instant::now();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
