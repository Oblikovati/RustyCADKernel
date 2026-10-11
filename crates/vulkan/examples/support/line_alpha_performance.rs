//! Live alpha policy changes include retained extraction, submission and readback.
use super::{line_arrays, scene::ViewerScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{quantity::ColorRgba, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use std::time::Instant;
/// Time alpha mutations through retained redraw/readback; `benchmark(&driver, 20, 100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    line_arrays::replace_lines(&*scene.group, "line-alpha-forced", 1);
    for index in 0..warmup {
        timed_alpha(&scene, index);
    }
    let samples: Vec<f64> = (0..count).map(|index| timed_alpha(&scene, index)).collect();
    assert!(scene.render_view().capture().is_ok());
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"line-alpha-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_alpha(scene: &ViewerScene, index: usize) -> f64 {
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
