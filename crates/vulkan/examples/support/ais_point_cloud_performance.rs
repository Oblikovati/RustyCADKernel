//! Point cloud rebuild, selection recompute and production redraw/readback timings.
use super::cloud_scene::PointCloudScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
/// Measure actual point cloud update and redraw cost; `benchmark(&driver,20,100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = PointCloudScene::new(driver, "cloud-colors", 0.);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for index in 0..warmup {
        timed_cloud(&scene, index);
    }
    let samples: Vec<f64> = (0..count).map(|index| timed_cloud(&scene, index)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"ais-point-cloud-updates\",\"draws\":1,\"points\":1024,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_cloud(scene: &PointCloudScene, index: usize) -> f64 {
    let start = Instant::now();
    scene.update_points(index);
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
