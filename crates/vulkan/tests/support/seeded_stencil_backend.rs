//! A named real-device test adaptor controls target contents without altering retained commands.
use martensite_vulkan::{
    FrameCapture, RenderBackend, RenderError, RenderFrame, VulkanOptions, VulkanRenderer,
};
pub struct SeededStencilBackend {
    renderer: VulkanRenderer,
    seed: u8,
}
impl SeededStencilBackend {
    pub fn new(options: VulkanOptions, seed: u8) -> Result<Self, RenderError> {
        Ok(Self {
            renderer: VulkanRenderer::new(options)?,
            seed,
        })
    }
}
impl SeededStencilBackend {
    pub fn driver_identity(&self) -> martensite_vulkan::RenderDriverIdentity {
        self.renderer.driver_identity()
    }
}
impl RenderBackend for SeededStencilBackend {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let capture = self
            .renderer
            .render(&frame.clone().with_stencil_clear_value(self.seed))?;
        let errors = self.renderer.validation_errors();
        if !errors.is_empty() {
            return Err(RenderError::rejected(
                errors,
                "zero Vulkan validation errors",
            ));
        }
        Ok(capture)
    }
}
