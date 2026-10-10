//! Rebuilding 32 B-spline presentations includes curve sampling, extraction and GPU readback.
use super::{scene::ViewerScene, stdprs_scene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for _ in 0..warmup {
        timed_curves(&scene);
    }
    let samples: Vec<f64> = (0..count).map(|_| timed_curves(&scene)).collect();
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"curve-presentations\",\"draws\":32,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_curves(scene: &ViewerScene) -> f64 {
    let start = Instant::now();
    stdprs_scene::replace_curve(&scene.structure, "stdprs-bspline", 32);
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
