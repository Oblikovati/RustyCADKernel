//! Live appearance mutation, retained AIS group updates and GPU redraw/readback.
use super::line_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObject;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = line_scene::LineScene::new(driver, false);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for i in 0..warmup {
        timed_line(&scene, i);
    }
    let samples: Vec<f64> = (0..count).map(|i| timed_line(&scene, i)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"ais-line-appearance\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_line(scene: &line_scene::LineScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .line
        .set_color(Color::named(if index.is_multiple_of(2) {
            NamedColor::RED
        } else {
            NamedColor::GREEN
        }))
        .unwrap();
    scene
        .line
        .set_width(if index.is_multiple_of(2) { 1. } else { 3. })
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
