//! Shaded datum geometry updates include recomputation, submission and GPU readback.
use super::trihedron_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = trihedron_scene::TrihedronScene::new(driver, "tri-shaded").unwrap();
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for index in 0..warmup {
        timed_trihedron(&scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_trihedron(&scene, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"ais-trihedron-shaded-updates\",\"draws\":10,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_trihedron(scene: &trihedron_scene::TrihedronScene, index: usize) -> f64 {
    use rustycad_tkv3d::ais::AisTrihedronSource;
    let start = Instant::now();
    scene
        .object
        .set_size(if index.is_multiple_of(2) { 4. } else { 6. })
        .unwrap();
    scene
        .context
        .redisplay(&scene.object.cast(), false, false)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
