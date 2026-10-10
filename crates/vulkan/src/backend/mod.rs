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

mod point_sprite;

mod window_blit;
mod window_chain;
mod window_support;
mod window_surface;
mod window_sync;
pub(crate) use window_surface::WindowPresenter;

impl VulkanRenderer {
    pub(crate) fn window_presenter(
        &self,
        window: &rustycad_tkernel::standard::Handle<dyn rustycad_tkservice::aspect::AspectWindow>,
    ) -> Result<WindowPresenter, RenderError> {
        WindowPresenter::new(&self.runtime, window)
    }
    pub(crate) fn render_window(
        &mut self,
        frame: &RenderFrame,
        presenter: &mut WindowPresenter,
        vsync: bool,
    ) -> Result<FrameCapture, RenderError> {
        let resources = frame::FrameResources::new(&self.runtime, frame, &self.pipeline)?;
        let commands = submission::CommandSubmission::new(&self.runtime)?;
        let index = presenter.acquire([frame.width, frame.height], vsync)?;
        let chain = presenter.chain.as_mut().expect("acquired swapchain");
        record::record_frame(&self.runtime, commands.command, &resources, frame);
        window_blit::record(&self.runtime, commands.command, &resources, chain, index);
        let waits = [ash::vk::SemaphoreSubmitInfo::default()
            .semaphore(chain.sync.acquired)
            .stage_mask(ash::vk::PipelineStageFlags2::TRANSFER)];
        let signals = [ash::vk::SemaphoreSubmitInfo::default()
            .semaphore(chain.sync.rendered)
            .stage_mask(ash::vk::PipelineStageFlags2::ALL_COMMANDS)];
        commands.finish_synced(&waits, &signals)?;
        chain.present(
            self.runtime.queues[presenter.present_family as usize],
            index,
        )?;
        resources.capture()
    }
}
