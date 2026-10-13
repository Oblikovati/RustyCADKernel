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
    fragment_alpha: crate::legacy_fragment_alpha::LegacyFragmentAlpha,
    stencil: crate::RenderStencilState,
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
            fragment_alpha: Default::default(),
            stencil: Default::default(),
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
    /// Read exact selected Vulkan implementation provenance; `renderer.driver_identity()`.
    pub fn driver_identity(&self) -> crate::RenderDriverIdentity {
        driver_identity::query_driver_identity(&self.runtime.instance, self.runtime.physical)
    }
    /// Read validation errors emitted so far; `renderer.validation_errors()`.
    pub fn validation_errors(&self) -> Vec<String> {
        self.runtime.validation.errors()
    }
}
mod buffer;
mod driver_identity;
mod fixed_lighting_pipeline;
mod fixed_vertex_layout;
mod fixed_vertex_sources;
mod frame;
mod image;
mod memory_selection;
mod pipeline;
mod pipeline_state;
mod pipeline_variants;
mod record;
mod submission;
mod transform_constants;
impl RenderBackend for VulkanRenderer {
    fn render(&mut self, frame: &RenderFrame) -> Result<FrameCapture, RenderError> {
        let resources = self.frame_resources(frame)?;
        let commands = submission::CommandSubmission::new(&self.runtime)?;
        record::record_frame(&self.runtime, commands.command, &resources, frame);
        commands.finish()?;
        self.commit_fragment_alpha(frame, resources.fragment_alpha.ending);
        self.commit_stencil(frame, resources.stencil.ending);
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
        self.commit_fragment_alpha(frame, resources.fragment_alpha.ending);
        self.commit_stencil(frame, resources.stencil.ending);
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
    fn commit_fragment_alpha(
        &mut self,
        frame: &RenderFrame,
        ending: crate::legacy_fragment_alpha::LegacyFragmentAlpha,
    ) {
        if let Some(snapshot) = &frame.fragment_alpha_context {
            snapshot.commit(ending);
            return;
        }
        self.fragment_alpha = ending;
    }
    fn commit_stencil(&mut self, frame: &RenderFrame, ending: crate::RenderStencilState) {
        if let Some(snapshot) = &frame.stencil_context {
            snapshot.commit(ending);
            return;
        }
        self.stencil = ending;
    }
    fn frame_resources(
        &mut self,
        frame: &RenderFrame,
    ) -> Result<frame::FrameResources, RenderError> {
        let initial_stencil = frame.initial_stencil(self.stencil);
        let capped = frame.stencil_clear_value != 0
            || initial_stencil.is_enabled()
            || frame
                .layers
                .iter()
                .any(|layer| !layer.stencil_boundaries.is_empty())
            || frame
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
        frame::FrameResources::new(
            &self.runtime,
            frame,
            pipeline,
            frame.initial_fragment_alpha(self.fragment_alpha),
            initial_stencil,
        )
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

mod mesh_edge_pipeline;

mod fragment_alpha_plan;
mod fragment_alpha_upload;
mod frame_raster_plan;
mod multisample_targets;
mod native_alpha_shader;
mod native_alpha_shader_table;
mod oit_barriers;
mod oit_blend;
mod oit_composite;
mod oit_descriptor;
mod oit_frame;
mod oit_pipeline;
mod oit_pipeline_cache;
mod oit_plan;
mod oit_record;
mod oit_render_commands;
#[cfg(test)]
mod oit_resource_tests;
mod oit_shader_table;
mod oit_support;
mod oit_targets;
mod oit_upload;
mod pipeline_layout;
mod raster_limits;
mod raster_samples;
mod render_target_record;

mod ordered_stencil_record;
