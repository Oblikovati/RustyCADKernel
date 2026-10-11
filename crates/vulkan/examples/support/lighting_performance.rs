//! End-to-end lighting timings include live normal/light mutations and GPU readback.
use crate::scene;
#[allow(dead_code)]
#[path = "../../tests/support/lighting_scene.rs"]
mod lighting_scene;
use lighting_scene::LightingScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkservice::graphic3d::Graphic3dAspectsMembers;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    for (mode, workload) in [
        ("lit-phong", "lighting-updates"),
        ("lit-line-phong", "lit-line-normal-updates"),
        ("lit-alpha-phong", "lit-surface-opacity-updates"),
    ] {
        benchmark_configuration(driver, mode, workload, warmup, count);
    }
}
fn benchmark_configuration(
    driver: &Handle<Graphic3dRenderDriver>,
    mode: &str,
    workload: &str,
    warmup: usize,
    count: usize,
) {
    let scene = LightingScene::new(driver, mode);
    scene.base.window.set_size(256, 256);
    scene.base.view.must_be_resized();
    for index in 0..warmup {
        timed_redraw(&scene, index, mode);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_redraw(&scene, index, mode))
        .collect();
    check_lighting_workload(&scene);
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{workload}\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.base.view.remove();
}
fn check_lighting_workload(scene: &LightingScene) {
    let capture = scene.base.render_view().capture().unwrap();
    assert!(
        capture
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .any(|pixel| *pixel != [0, 0, 255, 255])
    );
}

fn timed_redraw(scene: &LightingScene, index: usize, mode: &str) -> f64 {
    let start = Instant::now();
    mutate_lighting_workload(scene, index, mode);
    scene.base.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}

fn mutate_lighting_workload(scene: &LightingScene, index: usize, mode: &str) {
    scene
        .light
        .set_direction_coords(if index.is_multiple_of(2) { 0. } else { -1. }, 0., -1.)
        .unwrap();
    scene
        .vertices
        .set_vertex_normal(2, if index.is_multiple_of(2) { 0. } else { 0.6 }, 0., 0.8)
        .unwrap();
    if mode == "lit-alpha-phong" {
        let mut material = scene.aspect.front_material();
        material
            .set_transparency(if index.is_multiple_of(2) { 0. } else { 0.5 })
            .unwrap();
        scene.aspect.set_front_material(material);
    }
}
