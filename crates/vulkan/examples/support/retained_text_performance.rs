//! Measure complete retained text redraw/readback without introducing a second timing contract.
use martensite_vulkan::Graphic3dRenderView;
use std::time::Instant;
/// Record 100 complete samples after 20 warmups; `measure_text_view(&render, "subtitle-basic")`.
pub fn measure_text_view(render: &Graphic3dRenderView, mode: &str) {
    for _ in 0..20 {
        timed_text_view(render);
    }
    let samples: Vec<f64> = (0..100).map(|_| timed_text_view(render)).collect();
    println!(
        "MARTENSITE_PERFORMANCE={{\"workload\":\"{mode}\",\"warmup\":20,\"validation\":false,\"samples_ms\":{samples:?}}}"
    );
}
fn timed_text_view(render: &Graphic3dRenderView) -> f64 {
    let start = Instant::now();
    render.try_redraw().unwrap();
    start.elapsed().as_secs_f64() * 1000.
}
