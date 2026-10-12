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
    pub sprite_set: u32,
    shaders: Vec<vk::ShaderModule>,
    pub clipping_layout: Option<super::storage_descriptor::StorageDescriptorLayout>,
    pub lighting_layout: Option<super::storage_descriptor::StorageDescriptorLayout>,
    pub sprite_layout: Option<super::storage_descriptor::StorageDescriptorLayout>,
    pub surface_layout: Option<super::surface_image_descriptor::SurfaceImageLayout>,
}
impl DrawPipeline {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        kind: super::pipeline_variants::PipelineVariant,
    ) -> Result<Self, RenderError> {
        let super::pipeline_variants::PipelineVariant {
            clamp: _,
            line,
            point,
            text,
            text_blend: _,
            clipped,
            lighting,
            ..
        } = kind;
        let mut pipeline = Self {
            runtime: runtime.clone(),
            handle: vk::Pipeline::null(),
            layout: vk::PipelineLayout::null(),
            push_stages: push_range(
                line || kind.polygon_edges
                    || point
                    || text
                    || kind.surface_texture
                    || kind.surface_alpha,
                kind.mesh_edges,
            )
            .stage_flags,
            sprite_set: if lighting && (point || kind.surface_texture) {
                2
            } else {
                0
            },
            shaders: vec![],
            clipping_layout: if clipped {
                Some(super::storage_descriptor::StorageDescriptorLayout::new(
                    runtime,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )?)
            } else {
                None
            },
            lighting_layout: if lighting {
                Some(super::storage_descriptor::StorageDescriptorLayout::new(
                    runtime,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                )?)
            } else {
                None
            },
            surface_layout: if kind.surface_texture {
                Some(super::surface_image_descriptor::SurfaceImageLayout::new(
                    runtime,
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
        if kind.mesh_edges {
            for bytes in super::mesh_edge_pipeline::edge_shader_bytes(kind) {
                pipeline.shaders.push(load_shader(runtime, bytes)?);
            }
        } else if kind.surface_texture || kind.surface_alpha {
            for bytes in super::surface_texture_pipeline::surface_shaders(kind) {
                pipeline.shaders.push(load_shader(runtime, bytes)?);
            }
        } else {
            pipeline.load_shaders(line || kind.polygon_edges, point, text, clipped, lighting)?;
        }
        pipeline.layout = create_layout(
            runtime,
            line || kind.polygon_edges
                || point
                || text
                || kind.surface_texture
                || kind.surface_alpha,
            kind.mesh_edges,
            pipeline
                .sprite_layout
                .as_ref()
                .map(|v| v.handle)
                .or_else(|| pipeline.surface_layout.as_ref().map(|v| v.handle)),
            pipeline.lighting_layout.as_ref(),
            pipeline.clipping_layout.as_ref(),
        )?;
        pipeline.create(kind)?;
        Ok(pipeline)
    }
    fn load_shaders(
        &mut self,
        line: bool,
        point: bool,
        text: bool,
        clipped: bool,
        lighting: bool,
    ) -> Result<(), RenderError> {
        let shaders: [&[u8]; 2] = if lighting {
            lighting_shaders(clipped, point, line)
        } else if clipped {
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
        kind: super::pipeline_variants::PipelineVariant,
    ) -> Result<(), RenderError> {
        let stages = shader_stages(&self.shaders);
        let state = super::pipeline_state::PipelineState::new(kind);
        let color_format = [if kind.text_logic_xor || kind.encoded_surface {
            vk::Format::R8G8B8A8_UNORM
        } else {
            vk::Format::R8G8B8A8_SRGB
        }];
        let mut rendering = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_format)
            .depth_attachment_format(kind.depth_format())
            .stencil_attachment_format(if kind.stencil {
                kind.depth_format()
            } else {
                vk::Format::UNDEFINED
            });
        let mut info = state
            .description(self.layout, &stages)
            .push_next(&mut rendering);
        let mut line_state = vk::PipelineRasterizationLineStateCreateInfoEXT::default()
            .line_rasterization_mode(vk::LineRasterizationModeEXT::BRESENHAM);
        let mut raster =
            super::pipeline_state::raster_state(kind.clamp).polygon_mode(if kind.polygon_edges {
                vk::PolygonMode::LINE
            } else {
                vk::PolygonMode::FILL
            });
        if kind.line {
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
fn shader_stages(shaders: &[vk::ShaderModule]) -> Vec<vk::PipelineShaderStageCreateInfo<'static>> {
    let mut stages = vec![
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(shaders[0])
            .name(c"main"),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(shaders[1])
            .name(c"main"),
    ];
    if shaders.len() == 3 {
        stages.push(
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::GEOMETRY)
                .module(shaders[2])
                .name(c"main"),
        );
    }
    stages
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
    edges: bool,
    sprite: Option<vk::DescriptorSetLayout>,
    lighting: Option<&super::storage_descriptor::StorageDescriptorLayout>,
    clipping: Option<&super::storage_descriptor::StorageDescriptorLayout>,
) -> Result<vk::PipelineLayout, RenderError> {
    // Fragment constants force redundant lavapipe state work on every surface draw.
    let ranges = [push_range(line, edges)];
    let layouts = descriptor_layout_handles(
        sprite,
        lighting.map(|s| s.handle),
        clipping.map(|s| s.handle),
    );
    let info = vk::PipelineLayoutCreateInfo::default()
        .push_constant_ranges(&ranges)
        .set_layouts(&layouts);
    let layout = unsafe { runtime.device().create_pipeline_layout(&info, None) }
        .map_err(|e| vk_error("create pipeline layout", e))?;
    Ok(layout)
}

fn push_range(line: bool, edges: bool) -> vk::PushConstantRange {
    let stages = if edges {
        vk::ShaderStageFlags::VERTEX
            | vk::ShaderStageFlags::GEOMETRY
            | vk::ShaderStageFlags::FRAGMENT
    } else if line {
        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT
    } else {
        vk::ShaderStageFlags::VERTEX
    };
    vk::PushConstantRange::default()
        .stage_flags(stages)
        .size(if edges {
            128
        } else if line {
            80
        } else {
            64
        })
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

fn lighting_shaders(clipped: bool, point: bool, line: bool) -> [&'static [u8]; 2] {
    if line {
        return line_lighting_shaders(clipped);
    }
    if point {
        return point_lighting_shaders(clipped);
    }
    surface_lighting_shaders(clipped)
}
fn line_lighting_shaders(clipped: bool) -> [&'static [u8]; 2] {
    if clipped {
        return [
            include_bytes!("../../shaders/lighting.line.clipped.vert.spv"),
            include_bytes!("../../shaders/lighting.line.clipped.frag.spv"),
        ];
    }
    [
        include_bytes!("../../shaders/lighting.line.vert.spv"),
        include_bytes!("../../shaders/lighting.line.frag.spv"),
    ]
}
fn point_lighting_shaders(clipped: bool) -> [&'static [u8]; 2] {
    if clipped {
        return [
            include_bytes!("../../shaders/lighting.point.clipped.vert.spv"),
            include_bytes!("../../shaders/lighting.point.clipped.frag.spv"),
        ];
    }
    [
        include_bytes!("../../shaders/lighting.point.vert.spv"),
        include_bytes!("../../shaders/lighting.point.frag.spv"),
    ]
}
fn surface_lighting_shaders(clipped: bool) -> [&'static [u8]; 2] {
    if clipped {
        return [
            include_bytes!("../../shaders/lighting.clipped.vert.spv"),
            include_bytes!("../../shaders/lighting.clipped.frag.spv"),
        ];
    }
    [
        include_bytes!("../../shaders/lighting.vert.spv"),
        include_bytes!("../../shaders/lighting.frag.spv"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triangle_constants_do_not_invalidate_fragment_state() {
        assert_eq!(
            push_range(false, false).stage_flags,
            vk::ShaderStageFlags::VERTEX
        );
        assert_eq!(push_range(false, false).size, 64);
        assert_eq!(push_range(true, false).size, 80);
        assert!(
            push_range(true, false)
                .stage_flags
                .contains(vk::ShaderStageFlags::FRAGMENT)
        );
    }
    #[test]
    fn native_mesh_constants_are_visible_to_all_three_shader_stages() {
        let range = push_range(false, true);
        assert_eq!(range.size, 128);
        assert_eq!(range.offset, 0);
        assert_eq!(
            range.stage_flags,
            vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::GEOMETRY
                | vk::ShaderStageFlags::FRAGMENT
        );
    }
}

fn descriptor_layout_handles(
    sprite: Option<vk::DescriptorSetLayout>,
    lighting: Option<vk::DescriptorSetLayout>,
    clipping: Option<vk::DescriptorSetLayout>,
) -> Vec<vk::DescriptorSetLayout> {
    if let (Some(sprite), Some(lighting)) = (sprite, lighting) {
        // Keep clipping at set 1; an unused compatible layout fills its slot when absent.
        return vec![lighting, clipping.unwrap_or(lighting), sprite];
    }
    sprite
        .or(lighting)
        .or(clipping)
        .into_iter()
        .chain(clipping)
        .collect()
}

#[cfg(test)]
mod marker_descriptor_tests {
    use super::*;
    use ash::vk::Handle;
    #[test]
    fn lit_marker_keeps_lighting_clipping_and_sprite_in_distinct_slots() {
        let [sprite, lighting, clipping] = [1, 2, 3].map(vk::DescriptorSetLayout::from_raw);
        assert_eq!(
            descriptor_layout_handles(Some(sprite), Some(lighting), Some(clipping)),
            vec![lighting, clipping, sprite]
        );
        assert_eq!(
            descriptor_layout_handles(Some(sprite), Some(lighting), None),
            vec![lighting, lighting, sprite]
        );
        assert_eq!(
            descriptor_layout_handles(Some(sprite), None, Some(clipping)),
            vec![sprite, clipping]
        );
        assert_eq!(
            descriptor_layout_handles(None, Some(lighting), Some(clipping)),
            vec![lighting, clipping]
        );
        assert_eq!(
            descriptor_layout_handles(None, None, Some(clipping)),
            vec![clipping, clipping]
        );
        assert!(descriptor_layout_handles(None, None, None).is_empty());
    }
}
