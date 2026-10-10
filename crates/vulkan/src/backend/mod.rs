//! Only this private boundary uses Ash or Vulkan handles.
mod runtime;
mod validation;
use crate::{FrameCapture, RenderBackend, RenderError, RenderFrame};
pub use runtime::VulkanOptions;
use runtime::VulkanRuntime;
/// Vulkan 1.3 offscreen renderer; `VulkanRenderer::new(VulkanOptions::default())?`.
pub struct VulkanRenderer {
    runtime: std::sync::Arc<VulkanRuntime>,
}
impl VulkanRenderer {
    /// Load Vulkan and select a capable graphics device; `VulkanRenderer::new(options)?`.
    pub fn new(options: VulkanOptions) -> Result<Self, RenderError> {
        Ok(Self {
            runtime: VulkanRuntime::create(options)?,
        })
    }
    /// Inspect the selected device name; `renderer.device_name()`.
    pub fn device_name(&self) -> &str {
        &self.runtime.name
    }
    /// Read validation errors emitted so far; `renderer.validation_errors()`.
    pub fn validation_errors(&self) -> Vec<String> {
        self.runtime.validation.errors()
    }
}
mod buffer;
mod frame;
mod image;
mod pipeline;
mod pipeline_state;
mod record;
mod submission;
impl RenderBackend for VulkanRenderer {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let resources = frame::FrameResources::new(&self.runtime, frame)?;
        let commands = submission::CommandSubmission::new(&self.runtime)?;
        record::record_frame(&self.runtime, commands.command, &resources, frame);
        commands.finish()?;
        resources.capture()
    }
}
