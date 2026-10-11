//! End-to-end lighting timings include live normal/light mutations and GPU readback.
use crate::scene;
#[allow(dead_code)]
#[path = "../../tests/support/lighting_scene.rs"]
mod lighting_scene;
use lighting_scene::LightingScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = LightingScene::new(driver, "lit-phong");
    scene.base.window.set_size(256, 256);
    scene.base.view.must_be_resized();
    for index in 0..warmup {
        timed_redraw(&scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, index))
        .collect();
    assert_ne!(
        scene.base.render_view().capture().unwrap().pixel(128, 128),
        Some([0, 0, 255, 255])
    );
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"lighting-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.base.view.remove();
}
fn timed_redraw(scene: &LightingScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .light
        .set_direction_coords(if index.is_multiple_of(2) { 0. } else { -1. }, 0., -1.)
        .unwrap();
    scene
        .vertices
        .set_vertex_normal(2, if index.is_multiple_of(2) { 0. } else { 0.6 }, 0., 0.8)
        .unwrap();
    scene.base.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
