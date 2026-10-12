//! Explicit pixel submissions reuse the retained frontend's exact warmup/sample contract.
use martensite_vulkan::{RenderBackend, RenderFrame, VulkanRenderer};
/// Time complete pixel-frame submission/readback; `measure_pixel_frame(&mut renderer, &frame, "pixel-basic")`.
pub fn measure_pixel_frame(renderer: &mut VulkanRenderer, frame: &RenderFrame, mode: &str) {
    super::retained_text_performance::measure_text_operation(mode, &mut || {
        renderer.render(frame).unwrap();
    });
}
