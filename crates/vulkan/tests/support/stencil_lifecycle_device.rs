//! Named pre-submission I/O fault wraps the real renderer; no simulated successful GPU state.
use martensite_vulkan::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
pub struct StencilLifecycleDevice {
    renderer: VulkanRenderer,
    failure: Arc<AtomicBool>,
}
impl StencilLifecycleDevice {
    pub fn new(options: VulkanOptions, failure: Arc<AtomicBool>) -> Self {
        let renderer = VulkanRenderer::new(options).unwrap();
        eprintln!(
            "MARTENSITE_DEVICE_IDENTITY={:?}",
            renderer.driver_identity()
        );
        Self { renderer, failure }
    }
}
impl RenderBackend for StencilLifecycleDevice {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        if self.failure.load(Ordering::Relaxed) {
            return Err(RenderError::rejected(
                "named pre-submission fault",
                "a successful submission",
            ));
        }
        let capture = self
            .renderer
            .render(&frame.clone().with_stencil_clear_value(1))?;
        assert!(self.renderer.validation_errors().is_empty());
        Ok(capture)
    }
}
