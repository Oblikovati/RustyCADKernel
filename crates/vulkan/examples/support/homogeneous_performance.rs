//! Weighted position updates include live buffer extraction, GPU upload and readback.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::{Graphic3dBuffer, Graphic3dBufferMembers};
use std::time::Instant;
/// Measure weighted vertex updates; `benchmark(driver, 20, 100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    let buffer =
        super::homogeneous_arrays::replace_homogeneous(&*scene.group, "hom-triangle-mixed");
    for index in 0..warmup {
        timed_redraw(&scene, &buffer, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, &buffer, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"homogeneous-position-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_redraw(scene: &ViewerScene, buffer: &Graphic3dBuffer, index: usize) -> f64 {
    let start = Instant::now();
    assert!(buffer.write_at(12, if index.is_multiple_of(2) { 0.5f32 } else { 2. }));
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
