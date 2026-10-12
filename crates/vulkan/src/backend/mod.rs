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
    capping_pipeline: Option<pipeline_variants::RasterPipelines>,
}
impl VulkanRenderer {
    /// Load Vulkan and select a capable graphics device; `VulkanRenderer::new(options)?`.
    pub fn new(options: VulkanOptions) -> Result<Self, RenderError> {
        let runtime = VulkanRuntime::create(options)?;
        // Formats and topology are fixed; viewport and layer depth policies are dynamic.
        let pipeline = pipeline_variants::RasterPipelines::new(&runtime, false)?;
        Ok(Self {
            runtime,
            pipeline,
            capping_pipeline: None,
        })
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
        let resources = self.frame_resources(frame)?;
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
        let resources = self.frame_resources(frame)?;
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

mod storage_descriptor;

mod capping_state;
impl VulkanRenderer {
    fn frame_resources(
        &mut self,
        frame: &RenderFrame,
    ) -> Result<frame::FrameResources, RenderError> {
        let capped = frame
            .draws
            .iter()
            .any(|draw| draw.capping_pass != crate::RenderCappingPass::None);
        if capped && self.capping_pipeline.is_none() {
            self.capping_pipeline = Some(pipeline_variants::RasterPipelines::new(
                &self.runtime,
                true,
            )?);
        }
        let pipeline = if capped {
            self.capping_pipeline
                .as_ref()
                .expect("created stencil pipelines")
        } else {
            &self.pipeline
        };
        frame::FrameResources::new(&self.runtime, frame, pipeline)
    }
}

mod capping_record;

mod sampled_surface_image;
mod sampled_surface_upload;
mod surface_image_cache;
mod surface_image_descriptor;
mod surface_image_format;
mod surface_sampler;
mod surface_texture_pipeline;
