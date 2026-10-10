//! Each timed AIS cycle includes selection lifecycle and two completed GPU readbacks.
#[path = "../../tests/support/ais_scene.rs"]
mod ais_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use rustycad_tkv3d::ais::AisInteractiveObjectMembers;
use std::{sync::atomic::Ordering, time::Instant};
pub fn benchmark(
    driver: &Handle<Graphic3dRenderDriver>,
    warmup: usize,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    benchmark_cycle(driver, warmup, count, false)?;
    benchmark_cycle(driver, warmup, count, true)
}
fn benchmark_cycle(
    driver: &Handle<Graphic3dRenderDriver>,
    warmup: usize,
    count: usize,
    remove: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let scene = ais_scene::AisViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for _ in 0..warmup {
        timed_cycle(&scene, remove)?;
    }
    let samples = (0..count)
        .map(|_| timed_cycle(&scene, remove))
        .collect::<Result<Vec<f64>, _>>()?;
    assert_eq!(
        scene.object.computations.load(Ordering::Relaxed),
        if remove { 1 + warmup + count } else { 1 }
    );
    assert_eq!(scene.viewer.active_views().len(), 1);
    assert!(scene.object.has_interactive_context());
    assert!(scene.render_view().capture().is_ok());
    let name = if remove {
        "ais-remove-display"
    } else {
        "ais-display-erase"
    };
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{name}\",\"draws\":1,\"frames_per_sample\":2,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
    Ok(())
}
fn timed_cycle(
    scene: &ais_scene::AisViewerScene,
    remove: bool,
) -> Result<f64, Box<dyn std::error::Error>> {
    let start = Instant::now();
    // Explicit redraw results stop an erased-frame failure being hidden by redisplay.
    if remove {
        scene.context.remove(&scene.object.cast(), false)?;
    } else {
        scene.context.erase(&scene.object.cast(), false)?;
    }
    scene.render_view().try_redraw()?;
    scene.context.display(&scene.object.cast(), false)?;
    scene.render_view().try_redraw()?;
    Ok(start.elapsed().as_secs_f64() * 1000.)
}
