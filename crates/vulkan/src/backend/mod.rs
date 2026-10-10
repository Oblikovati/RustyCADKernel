//! Only this private boundary uses Ash or Vulkan handles.
mod runtime;
mod validation;
use crate::{FrameCapture, RenderBackend, RenderError, RenderFrame};
pub use runtime::VulkanOptions;
use runtime::VulkanRuntime;
/// Vulkan 1.3 offscreen renderer; `VulkanRenderer::new(VulkanOptions::default())?`.
pub struct VulkanRenderer {
    runtime: std::sync::Arc<VulkanRuntime>,
    pipeline: pipeline_variants::RasterPipelines,
}
impl VulkanRenderer {
    /// Load Vulkan and select a capable graphics device; `VulkanRenderer::new(options)?`.
    pub fn new(options: VulkanOptions) -> Result<Self, RenderError> {
        let runtime = VulkanRuntime::create(options)?;
        // Formats and topology are fixed; viewport and layer depth policies are dynamic.
        let pipeline = pipeline_variants::RasterPipelines::new(&runtime)?;
        Ok(Self { runtime, pipeline })
    }
    pub(crate) fn capture_limit(&self) -> u32 {
        let limits = &self.runtime.limits;
        limits
            .max_framebuffer_width
            .min(limits.max_framebuffer_height)
            .min(limits.max_image_dimension2_d)
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
mod memory_selection;
mod pipeline;
mod pipeline_state;
mod pipeline_variants;
mod record;
mod submission;
impl RenderBackend for VulkanRenderer {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let resources = frame::FrameResources::new(&self.runtime, frame, &self.pipeline)?;
        let commands = submission::CommandSubmission::new(&self.runtime)?;
        record::record_frame(&self.runtime, commands.command, &resources, frame);
        commands.finish()?;
        resources.capture()
    }
}

mod line_device;

mod draw_state;
