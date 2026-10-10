//! Sixty-four independently persistent groups track a changing camera every completed frame.
use super::{persistence_scene, scene::ViewerScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = persistent_groups(driver);
    for index in 0..warmup {
        timed_camera(&scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_camera(&scene, index))
        .collect();
    assert_eq!(scene.structure.groups().len(), 64);
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"persistence-groups\",\"draws\":64,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn persistent_groups(driver: &Handle<Graphic3dRenderDriver>) -> ViewerScene {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    persistence_scene::configure_persistence(
        &scene.structure,
        &*scene.group,
        &scene.vertices,
        &scene.view.camera(),
        "pers-group-zoom",
    );
    for index in 1..64 {
        let group = scene.structure.new_group().unwrap();
        group.set_group_primitives_aspect(&scene.group.aspects());
        group.add_primitive_array(&scene.vertices, true);
        let persistence = persistence_scene::scene_persistence("pers-group-zoom");
        persistence
            .set_anchor_point(&Point3::new(
                (index % 8) as f64 - 4.,
                (index / 8) as f64 - 4.,
                0.,
            ))
            .unwrap();
        group.set_transform_persistence(&persistence);
    }
    scene
}
fn timed_camera(scene: &ViewerScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .view
        .camera()
        .set_scale(if index.is_multiple_of(2) { 20. } else { 30. });
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
