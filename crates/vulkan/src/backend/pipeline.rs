//! The initial unlit pipeline consumes real vertex/index buffers and a per-draw transform.
use super::runtime::{VulkanRuntime, vk_error};
use crate::{RenderError, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct DrawPipeline {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub push_stages: vk::ShaderStageFlags,
    shaders: Vec<vk::ShaderModule>,
    pub clipping_layout: Option<super::storage_descriptor::StorageDescriptorLayout>,
    pub sprite_layout: Option<super::storage_descriptor::StorageDescriptorLayout>,
}
impl DrawPipeline {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        depth_clamp: bool,
        line: bool,
        point: bool,
        text: bool,
        text_blend: bool,
        clipped: bool,
    ) -> Result<Self, RenderError> {
        let mut pipeline = Self {
            runtime: runtime.clone(),
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            push_stages: push_range(line || point || text).stage_flags,
            shaders: vec![],
            clipping_layout: if clipped {
                Some(super::storage_descriptor::StorageDescriptorLayout::new(
                    runtime,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )?)
            } else {
                None
            },
            sprite_layout: if point || text {
                Some(super::storage_descriptor::StorageDescriptorLayout::new(
                    runtime,
                    vk::ShaderStageFlags::FRAGMENT,
                )?)
            } else {
                None
            },
        };
        pipeline.load_shaders(line, point, text, clipped)?;
        pipeline.layout = create_layout(
            runtime,
            line || point || text,
            pipeline.sprite_layout.as_ref(),
            pipeline.clipping_layout.as_ref(),
        )?;
        pipeline.create(depth_clamp, line, point, text, text_blend)?;
        Ok(pipeline)
    }
    fn load_shaders(
        &mut self,
        line: bool,
        point: bool,
        text: bool,
        clipped: bool,
    ) -> Result<(), RenderError> {
        let shaders: [&[u8]; 2] = if clipped {
            clipping_shaders(line, point, text)
        } else if text {
            [
                include_bytes!("../../shaders/text.vert.spv"),
                include_bytes!("../../shaders/text.frag.spv"),
            ]
        } else if point {
            [
                include_bytes!("../../shaders/point.vert.spv"),
                include_bytes!("../../shaders/point.frag.spv"),
            ]
        } else if line {
            [
                include_bytes!("../../shaders/line.vert.spv"),
                include_bytes!("../../shaders/line.frag.spv"),
            ]
        } else {
            [
                include_bytes!("../../shaders/unlit.vert.spv"),
                include_bytes!("../../shaders/unlit.frag.spv"),
            ]
        };
        for bytes in shaders {
            self.shaders.push(load_shader(&self.runtime, bytes)?);
        }
        Ok(())
    }
    fn create(
        &mut self,
        depth_clamp: bool,
        line: bool,
        point: bool,
        text: bool,
        text_blend: bool,
    ) -> Result<(), RenderError> {
        let stages = shader_stages(&self.shaders);
        let state =
            super::pipeline_state::PipelineState::new(depth_clamp, line, point, text, text_blend);
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&[vk::Format::R8G8B8A8_SRGB])
            .depth_attachment_format(vk::Format::D32_SFLOAT);
        let mut info = state
            .description(self.layout, &stages)
            .push_next(&mut rendering);
        let mut line_state = vk::PipelineRasterizationLineStateCreateInfoEXT::default()
            .line_rasterization_mode(vk::LineRasterizationModeEXT::BRESENHAM);
        let mut raster = super::pipeline_state::raster_state(depth_clamp);
        if line {
            raster = raster.push_next(&mut line_state);
            info = info.rasterization_state(&raster);
        }
        self.handle = create_pipeline(&self.runtime, info)?;
        Ok(())
    }
}
impl Drop for DrawPipeline {
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

fn create_layout(
    runtime: &VulkanRuntime,
    line: bool,
    sprite: Option<&super::storage_descriptor::StorageDescriptorLayout>,
    clipping: Option<&super::storage_descriptor::StorageDescriptorLayout>,
) -> Result<vk::PipelineLayout, RenderError> {
    // Fragment constants force redundant lavapipe state work on every surface draw.
    let ranges = [push_range(line)];
    let mut layouts: Vec<_> = sprite.or(clipping).iter().map(|s| s.handle).collect();
    layouts.extend(clipping.iter().map(|s| s.handle));
    let info = vk::PipelineLayoutCreateInfo::default()
        .push_constant_ranges(&ranges)
        .set_layouts(&layouts);
    let layout = unsafe { runtime.device().create_pipeline_layout(&info, None) }
        .map_err(|e| vk_error("create pipeline layout", e))?;
    Ok(layout)
}

fn push_range(line: bool) -> vk::PushConstantRange {
    let stages = if line {
        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
    } else {
        vk::ShaderStageFlags::VERTEX
    };
    vk::PushConstantRange::default()
        .stage_flags(stages)
        .size(if line { 80 } else { 64 })
}

fn clipping_shaders(line: bool, point: bool, text: bool) -> [&'static [u8]; 2] {
    if text {
        return [
            include_bytes!("../../shaders/text.clipped.vert.spv"),
            include_bytes!("../../shaders/text.clipped.frag.spv"),
        ];
    }
    if point {
        return [
            include_bytes!("../../shaders/point.clipped.vert.spv"),
            include_bytes!("../../shaders/point.clipped.frag.spv"),
        ];
    }
    if line {
        return [
            include_bytes!("../../shaders/line.clipped.vert.spv"),
            include_bytes!("../../shaders/line.clipped.frag.spv"),
        ];
    }
    [
        include_bytes!("../../shaders/unlit.clipped.vert.spv"),
        include_bytes!("../../shaders/unlit.clipped.frag.spv"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triangle_constants_do_not_invalidate_fragment_state() {
        assert_eq!(push_range(false).stage_flags, vk::ShaderStageFlags::VERTEX);
        assert_eq!(push_range(false).size, 64);
        assert_eq!(push_range(true).size, 80);
        assert!(
            push_range(true)
                .stage_flags
                .contains(vk::ShaderStageFlags::FRAGMENT)
        );
    }
}
