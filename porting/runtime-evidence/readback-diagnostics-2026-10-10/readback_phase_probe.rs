//! Temporary diagnostic: the original AIS two-frame cycle, isolated from other workloads.
#[path = "support/device_options.rs"]
mod device_options;
#[path = "../tests/support/ais_scene.rs"]
mod scene;
#[path = "../tests/support/view_setup.rs"]
mod view_setup;
use martensite_vulkan::Graphic3dRenderDriver;
use std::{sync::atomic::Ordering, time::Instant};
fn main() {
    let mut options = device_options::validation_options();
    options.validation = false;
    let driver = Graphic3dRenderDriver::new(options).unwrap();
    println!("MARTENSITE_DEVICE={}", driver.device_name());
    let scene = scene::AisViewerScene::new(&driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for _ in 0..20 {
        cycle(&scene);
    }
    let samples: Vec<f64> = (0..100).map(|_| cycle(&scene)).collect();
    assert_eq!(scene.object.computations.load(Ordering::Relaxed), 1);
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.render_view().capture().is_ok());
    println!("MARTENSITE_ISOLATION={{\"samples_ms\":{samples:?}}}");
    scene.view.remove();
}
fn cycle(scene: &scene::AisViewerScene) -> f64 {
    let start = Instant::now();
    scene.context.erase(&scene.object.cast(), false).unwrap();
    scene.render_view().try_redraw().unwrap();
    scene.context.display(&scene.object.cast(), false).unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
