//! Live appearance mutation, retained AIS group updates and GPU redraw/readback.
use super::plane_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{
    quantity::{Color, NamedColor},
    standard::Handle,
};
use rustycad_tkv3d::prs_mgr::PrsMgrPresentableObject;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = plane_scene::PlaneScene::new(driver, "plane-base");
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for i in 0..warmup {
        timed_plane(&scene, i);
    }
    let samples: Vec<f64> = (0..count).map(|i| timed_plane(&scene, i)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"plane-trihedron-updates\",\"draws\":6,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_plane(scene: &plane_scene::PlaneScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .object
        .set_color(Color::named(if index.is_multiple_of(2) {
            NamedColor::RED
        } else {
            NamedColor::GREEN
        }))
        .unwrap();
    scene
        .object
        .set_length(if index.is_multiple_of(2) { 4. } else { 6. });
    scene
        .context
        .redisplay(&scene.object.cast(), false, false)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
