//! A timed indexed strip array exercises 256 native-style bound draws per redraw.
use super::{scene::ViewerScene, surface_arrays};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    let array = surface_arrays::replace_surface(&*scene.group, "surface-strip-indexed", 128);
    for _ in 0..warmup {
        timed_surface(&scene);
    }
    let samples: Vec<f64> = (0..count).map(|_| timed_surface(&scene)).collect();
    assert_eq!(array.bound_number(), 256);
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"surface-bounds\",\"draws\":256,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_surface(scene: &ViewerScene) -> f64 {
    let start = Instant::now();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
