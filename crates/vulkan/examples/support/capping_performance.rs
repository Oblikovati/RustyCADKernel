//! Live section planes include closed-solid mask construction and cap rasterization.
use super::scene::ViewerScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{ncollection::Vec4, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    let plane = super::capping_scene::configure_capping(&scene, "cap-basic");
    for index in 0..warmup {
        timed_redraw(&scene, &plane, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, &plane, index))
        .collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"capping-plane-updates\",\"draws\":3,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_redraw(scene: &ViewerScene, plane: &Graphic3dClipPlane, index: usize) -> f64 {
    let start = Instant::now();
    plane
        .set_equation(Vec4::new(
            0.,
            0.,
            -1.,
            if index.is_multiple_of(2) { -2. } else { 2. },
        ))
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
