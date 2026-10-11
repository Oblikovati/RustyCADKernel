//! Point cloud rebuild, selection recompute and production redraw/readback timings.
use super::cloud_scene::PointCloudScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
/// Measure actual point cloud update and redraw cost; `benchmark(&driver,20,100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    benchmark_mode(
        driver,
        warmup,
        count,
        "cloud-colors",
        "ais-point-cloud-updates",
    );
    benchmark_mode(
        driver,
        warmup,
        count,
        "cloud-lit-phong",
        "lit-point-normal-updates",
    );
}
fn benchmark_mode(
    driver: &Handle<Graphic3dRenderDriver>,
    warmup: usize,
    count: usize,
    mode: &str,
    workload: &str,
) {
    let scene = PointCloudScene::new(driver, mode, 0.);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    if mode == "cloud-lit-phong" {
        scene.update_points(0);
    }
    for index in 0..warmup {
        timed_cloud(&scene, index, mode == "cloud-lit-phong");
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_cloud(&scene, index, mode == "cloud-lit-phong"))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{workload}\",\"draws\":1,\"points\":1024,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_cloud(scene: &PointCloudScene, index: usize, lit: bool) -> f64 {
    let start = Instant::now();
    if lit {
        scene.update_lit_normals(index);
    } else {
        scene.update_points(index);
    }
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
