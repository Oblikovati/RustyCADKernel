//! The initial unlit pipeline consumes real vertex/index buffers and a per-draw transform.
use super::runtime::{VulkanRuntime, vk_error};
use crate::{RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct TrianglePipeline {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    shaders: Vec<vk::ShaderModule>,
}
impl TrianglePipeline {
    pub fn new(runtime: &Arc<VulkanRuntime>, depth_clamp: bool) -> Result<Self, RenderError> {
        let mut pipeline = Self {
            runtime: runtime.clone(),
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            shaders: vec![],
        };
        pipeline.shaders.push(load_shader(
            runtime,
            include_bytes!("../../shaders/unlit.vert.spv"),
        )?);
        pipeline.shaders.push(load_shader(
            runtime,
            include_bytes!("../../shaders/unlit.frag.spv"),
        )?);
        pipeline.layout = create_layout(runtime)?;
        pipeline.create(depth_clamp)?;
        Ok(pipeline)
    }
    fn create(&mut self, depth_clamp: bool) -> Result<(), RenderError> {
        let stages = shader_stages(&self.shaders);
        let state = super::pipeline_state::PipelineState::new(depth_clamp);
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[vk::Format::R8G8B8A8_UNORM])
            .depth_attachment_format(vk::Format::D32_SFLOAT);
        let info = state
            .description(self.layout, &stages)
            .push_next(&mut rendering);
        self.handle = create_pipeline(&self.runtime, info)?;
        Ok(())
    }
}
impl Drop for TrianglePipeline {
    fn drop(&mut self) {
        unsafe {
            let device = self.runtime.device();
            if self.handle != vk::Pipeline::null() {
                device.destroy_pipeline(self.handle, None);
            }
            if self.layout != vk::PipelineLayout::null() {
                device.destroy_pipeline_layout(self.layout, None);
            }
            for shader in self.shaders.drain(..) {
                device.destroy_shader_module(shader, None);
            }
        }
    }
}
fn load_shader(runtime: &VulkanRuntime, bytes: &[u8]) -> Result<vk::ShaderModule, RenderError> {
    if !bytes.len().is_multiple_of(4) {
        return Err(rejected(bytes.len(), "SPIR-V words aligned to four bytes"));
    }
    let words: Vec<u32> = bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|word| u32::from_le_bytes(*word))
        .collect();
    unsafe {
        runtime
            .device()
            .create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&words), None)
    }
    .map_err(|e| vk_error("create shader module", e))
}
fn shader_stages(shaders: &[vk::ShaderModule]) -> [vk::PipelineShaderStageCreateInfo<'static>; 2] {
    [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(shaders[0])
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(shaders[1])
            .name(c"main"),
    ]
}
fn create_pipeline(
    runtime: &VulkanRuntime,
    info: vk::GraphicsPipelineCreateInfo<'_>,
) -> Result<vk::Pipeline, RenderError> {
    match unsafe {
        runtime
            .device()
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
    } {
        Ok(pipelines) => Ok(pipelines[0]),
        Err((partial, error)) => {
            for handle in partial {
                unsafe { runtime.device().destroy_pipeline(handle, None) };
            }
            Err(vk_error("create graphics pipeline", error))
        }
    }
}

fn create_layout(runtime: &VulkanRuntime) -> Result<vk::PipelineLayout, RenderError> {
    let ranges = [vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        .size(64)];
    let info = vk::PipelineLayoutCreateInfo::default().push_constant_ranges(&ranges);
    let layout = unsafe { runtime.device().create_pipeline_layout(&info, None) }
        .map_err(|e| vk_error("create pipeline layout", e))?;
    Ok(layout)
}
