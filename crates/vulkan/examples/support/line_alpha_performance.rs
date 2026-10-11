//! Live alpha policy changes include retained extraction, submission and readback.
use super::{line_arrays, scene::ViewerScene};
use martensite_vulkan::Graphic3dRenderDriver;
use rustycad_tkernel::standard::Handle;
/// Time alpha mutations through retained redraw/readback; `benchmark(&driver, 20, 100)`.
pub fn benchmark(driver: &Handle<Graphic3dRenderDriver>, warmup: usize, count: usize) {
    let scene = ViewerScene::new(driver);
    scene.window.set_size(256, 256);
    scene.view.must_be_resized();
    line_arrays::replace_lines(&*scene.group, "line-alpha-forced", 1);
    super::alpha_redraw_performance::sample_alpha_redraws(
        &scene,
        warmup,
        count,
        "line-alpha-updates",
    );
    scene.view.remove();
}
