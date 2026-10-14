//! Fullscreen composites are geometry-free and preserve the opaque scene depth.
use super::{
    oit_descriptor::OitDescriptorLayout,
    pipeline::{create_pipeline, load_shader, shader_stages},
    runtime::{VulkanRuntime, vk_error},
};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OitCompositePhase {
    Weighted,
    Back,
    Peeling,
}
#[derive(Clone, Copy)]
pub(super) struct OitCompositeTarget {
    pub phase: OitCompositePhase,
    pub format: vk::Format,
    pub depth: vk::Format,
    pub stencil: bool,
    pub samples: vk::SampleCountFlags,
}
pub(super) struct OitCompositePipeline {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptors: Arc<OitDescriptorLayout>,
    shaders: Vec<vk::ShaderModule>,
}
impl OitCompositePipeline {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        target: OitCompositeTarget,
    ) -> Result<Self, RenderError> {
        let descriptors = OitDescriptorLayout::new(runtime, false)?;
        let mut pipeline = Self {
            runtime: runtime.clone(),
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            descriptors,
            shaders: vec![],
        };
        pipeline.initialize_shaders(target)?;
        pipeline.create(target)?;
        Ok(pipeline)
    }
    fn initialize_shaders(&mut self, target: OitCompositeTarget) -> Result<(), RenderError> {
        for bytes in composite_shaders(target) {
            self.shaders.push(load_shader(&self.runtime, bytes)?);
        }
        let layouts = [self.descriptors.handle];
        let info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);
        self.layout = unsafe { self.runtime.device().create_pipeline_layout(&info, None) }
            .map_err(|e| vk_error("create OIT composite layout", e))?;
        Ok(())
    }
    fn create(&mut self, target: OitCompositeTarget) -> Result<(), RenderError> {
        let state = CompositeFixedState::new(target);
        let stages = shader_stages(&self.shaders);
        let blends = [composite_blend(target.phase)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blends);
        let formats = [target.format];
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&formats)
            .depth_attachment_format(target.depth)
            .stencil_attachment_format(composite_stencil_format(target));
        let info = state
            .description(self.layout, &stages)
            .color_blend_state(&blend)
            .push_next(&mut rendering);
        self.handle = create_pipeline(&self.runtime, info)?;
        Ok(())
    }
}
impl Drop for OitCompositePipeline {
    fn drop(&mut self) {
        unsafe {
            let device = self.runtime.device();
            device.destroy_pipeline(self.handle, None);
            device.destroy_pipeline_layout(self.layout, None);
            for shader in self.shaders.drain(..) {
                device.destroy_shader_module(shader, None);
            }
        }
    }
}
fn composite_stencil_format(target: OitCompositeTarget) -> vk::Format {
    if target.stencil {
        return target.depth;
    }
    vk::Format::UNDEFINED
}
fn composite_shaders(target: OitCompositeTarget) -> [&'static [u8]; 2] {
    let fragment = match target.phase {
        OitCompositePhase::Weighted if target.samples != vk::SampleCountFlags::TYPE_1 => {
            include_bytes!("../../shaders/oit.weighted.msaa.frag.spv").as_slice()
        }
        OitCompositePhase::Weighted => {
            include_bytes!("../../shaders/oit.weighted.frag.spv").as_slice()
        }
        OitCompositePhase::Back => include_bytes!("../../shaders/oit.back.frag.spv").as_slice(),
        OitCompositePhase::Peeling => {
            include_bytes!("../../shaders/oit.peeling.frag.spv").as_slice()
        }
    };
    [
        include_bytes!("../../shaders/oit_composite.vert.spv"),
        fragment,
    ]
}
fn composite_blend(phase: OitCompositePhase) -> vk::PipelineColorBlendAttachmentState {
    match phase {
        OitCompositePhase::Weighted => super::oit_blend::weighted_composite_blend(),
        OitCompositePhase::Back => super::oit_blend::peeling_back_blend(),
        OitCompositePhase::Peeling => super::oit_blend::peeling_composite_blend(),
    }
}
struct CompositeFixedState {
    vertex: vk::PipelineVertexInputStateCreateInfo<'static>,
    assembly: vk::PipelineInputAssemblyStateCreateInfo<'static>,
    viewport: vk::PipelineViewportStateCreateInfo<'static>,
    raster: vk::PipelineRasterizationStateCreateInfo<'static>,
    samples: vk::PipelineMultisampleStateCreateInfo<'static>,
    depth: vk::PipelineDepthStencilStateCreateInfo<'static>,
    dynamic: vk::PipelineDynamicStateCreateInfo<'static>,
}
impl CompositeFixedState {
    fn new(target: OitCompositeTarget) -> Self {
        Self {
            vertex: Default::default(),
            assembly: vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
            viewport: vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1),
            raster: vk::PipelineRasterizationStateCreateInfo::default()
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.),
            samples: vk::PipelineMultisampleStateCreateInfo::default()
                .rasterization_samples(target.samples)
                .sample_shading_enable(target.samples != vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.),
            depth: vk::PipelineDepthStencilStateCreateInfo::default(),
            dynamic: vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(super::ordered_stencil_record::COMPOSITE_DYNAMIC),
        }
    }
    fn description<'a>(
        &'a self,
        layout: vk::PipelineLayout,
        stages: &'a [vk::PipelineShaderStageCreateInfo<'a>],
    ) -> vk::GraphicsPipelineCreateInfo<'a> {
        vk::GraphicsPipelineCreateInfo::default()
            .layout(layout)
            .stages(stages)
            .vertex_input_state(&self.vertex)
            .input_assembly_state(&self.assembly)
            .viewport_state(&self.viewport)
            .rasterization_state(&self.raster)
            .multisample_state(&self.samples)
            .depth_stencil_state(&self.depth)
            .dynamic_state(&self.dynamic)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn composite_modes_keep_sample_id_and_scene_depth_independent() {
        let mut target = OitCompositeTarget {
            phase: OitCompositePhase::Weighted,
            format: vk::Format::R8G8B8A8_SRGB,
            depth: vk::Format::D32_SFLOAT,
            stencil: false,
            samples: vk::SampleCountFlags::TYPE_1,
        };
        let single = composite_shaders(target)[1];
        target.samples = vk::SampleCountFlags::TYPE_4;
        assert_ne!(single, composite_shaders(target)[1]);
        let state = CompositeFixedState::new(target);
        assert_eq!(state.samples.sample_shading_enable, vk::TRUE);
        assert_eq!(state.depth.depth_write_enable, vk::FALSE);
        assert_eq!(state.vertex.vertex_binding_description_count, 0);
        assert_eq!(
            composite_blend(OitCompositePhase::Peeling).src_color_blend_factor,
            vk::BlendFactor::ONE
        );
    }
}
