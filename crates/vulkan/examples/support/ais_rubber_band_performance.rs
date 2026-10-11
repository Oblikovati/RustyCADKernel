//! Filling updates include triangulation, retained cache writes and actual GPU readback.
use super::rubber_band_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::ais::AisRubberBandSource;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = rubber_band_scene::RubberBandScene::new(driver).unwrap();
    scene.apply("rubber-filled", 0.).unwrap();
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for index in 0..warmup {
        timed_rubber_band(&scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_rubber_band(&scene, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"ais-rubber-band-updates\",\"draws\":2,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_rubber_band(scene: &rubber_band_scene::RubberBandScene, index: usize) -> f64 {
    let start = Instant::now();
    let edge = if index.is_multiple_of(2) { 36 } else { 40 };
    scene.object.set_rectangle(8, 8, edge, edge);
    scene
        .context
        .redisplay(&scene.object.cast(), false, false)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    assert!(scene.render_view().capture().is_ok());
    start.elapsed().as_secs_f64() * 1000.
}
