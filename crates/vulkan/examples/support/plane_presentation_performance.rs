//! Include real plane reconstruction and redraw/readback in both GPU profiles.
use super::plane_presentation_scene::PlanePresentationScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
/// Measure rebuild plus readback; `benchmark(&driver, 20, 100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = PlanePresentationScene::new(driver, "prsplane-corners", 0.);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for index in 0..warmup {
        timed_plane(&scene, index);
    }
    let samples: Vec<f64> = (0..count).map(|index| timed_plane(&scene, index)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"plane-presentation-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_plane(scene: &PlanePresentationScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .drawer
        .plane_aspect()
        .set_plane_length(12. + (index % 2) as f64, 10.);
    scene.rebuild("prsplane-corners");
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
