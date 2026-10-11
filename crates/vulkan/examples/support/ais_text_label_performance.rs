//! Include real label measurement, presentation rebuilding, selection and redraw/readback.
use super::text_label_scene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::{standard::Handle, tcollection::ExtendedString};
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = text_label_scene::TextLabelScene::new(driver, "label-flip", 0.);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    for index in 0..warmup {
        timed_label(&scene, index);
    }
    let samples: Vec<f64> = (0..count).map(|index| timed_label(&scene, index)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"ais-text-label-updates\",\"draws\":1,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.view.remove();
}
fn timed_label(scene: &text_label_scene::TextLabelScene, index: usize) -> f64 {
    let start = Instant::now();
    scene
        .label
        .set_text(&ExtendedString::new(if index.is_multiple_of(2) {
            "AV"
        } else {
            "VA"
        }));
    scene
        .context
        .redisplay(&scene.label.cast(), false, false)
        .unwrap();
    scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
