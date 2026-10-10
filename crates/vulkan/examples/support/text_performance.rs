//! Include live layout, atlas upload, GPU submission and readback in text timings.
use super::text_scene::TextScene;
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
use std::time::Instant;
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = TextScene::new(driver);
    scene.scene.window.set_size(512, 512);
    scene.scene.view.must_be_resized();
    scene
        .text
        .set_text_str(&vec!["AV".repeat(16); 32].join("\n"));
    scene.text.set_height(8.);
    scene.apply("text-basic");
    for _ in 0..warmup {
        timed_text(&scene);
    }
    let samples: Vec<f64> = (0..count).map(|_| timed_text(&scene)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"text-glyphs\",\"draws\":1,\"glyphs\":1024,\"warmup\":{warmup},\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
    scene.scene.view.remove();
}
fn timed_text(scene: &TextScene) -> f64 {
    let start = Instant::now();
    scene.scene.render_view().try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
