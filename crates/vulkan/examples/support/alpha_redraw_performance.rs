//! Shared opacity/mode redraw timing preserves comparable line/point work.
use super::scene::ViewerScene;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
/// Retain complete redraw samples; `sample_alpha_redraws(&scene, 20, 100, "point-alpha-updates")`.
pub fn sample_alpha_redraws(scene: &ViewerScene, warmup: usize, count: usize, workload: &str) {
    for index in 0..warmup {
        timed_alpha_redraw(scene, index);
    }
    let samples: Vec<f64> = (0..count)
        .map(|index| timed_alpha_redraw(scene, index))
        .collect();
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{workload}\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
}
fn timed_alpha_redraw(scene: &ViewerScene, index: usize) -> f64 {
    let start = Instant::now();
    let style = scene.group.aspects();
    let forced = index.is_multiple_of(2);
    style.set_alpha_mode(
        if forced {
            Graphic3dAlphaMode::MaskBlend
        } else {
            Graphic3dAlphaMode::Opaque
        },
        0.9,
    );
    style.set_interior_color_rgba(
        ColorRgba::new(1., 0., 0., if forced { 0.25 } else { 0.75 }).unwrap(),
    );
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
