//! Live plane resizing and shaded recomputation include production redraw/readback.
use super::ais_plane_scene::AisPlaneScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
/// Exercise both native plane presentations; `benchmark(&driver,20,100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    for (mode, name) in [
        ("aisplane-wire", "ais-plane-wire-updates"),
        ("aisplane-shaded-alpha", "ais-plane-shaded-updates"),
    ] {
        let scene = AisPlaneScene::new(driver, mode, 0.);
        scene.window.set_size(256, 256);
        scene.view.must_be_resized();
        for index in 0..warmup {
            timed_plane(&scene, index);
        }
        let samples: Vec<f64> = (0..count).map(|index| timed_plane(&scene, index)).collect();
        println!(
            "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
        );
        assert_eq!(scene.viewer.active_views().len(), 1);
        scene.view.remove();
    }
}
fn timed_plane(scene: &AisPlaneScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .object
        .set_size_xy(12. + (index % 2) as f64, 10.)
        .unwrap();
    if !scene.shaded.is_null() {
        scene.refresh_shaded();
    }
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
