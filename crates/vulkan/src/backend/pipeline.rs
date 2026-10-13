//! The initial unlit pipeline consumes real vertex/index buffers and a per-draw transform.
#[cfg(test)]
use super::pipeline_layout::descriptor_layout_handles;
use super::pipeline_layout::{create_layout, fixed_marker_push_range, push_range};
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
    pub kind: super::pipeline_variants::PipelineVariant,
    pub oit_layout: Option<Arc<super::oit_descriptor::OitDescriptorLayout>>,
    shaders: Vec<vk::ShaderModule>,
    fragment_bytes: &'static [u8],
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
        Self::build(
            runtime,
            kind,
            None,
            vk::SampleCountFlags::TYPE_1,
            false,
            None,
        )
    }
    pub fn new_oit(
        runtime: &Arc<VulkanRuntime>,
        kind: super::pipeline_variants::PipelineVariant,
        target: super::oit_pipeline::OitPipelineTarget,
        layout: &Arc<super::oit_descriptor::OitDescriptorLayout>,
    ) -> Result<Self, RenderError> {
        Self::build(
            runtime,
            kind,
            Some((target, layout.clone())),
            target.samples,
            false,
            None,
        )
    }
    pub fn new_multisample(
        runtime: &Arc<VulkanRuntime>,
        kind: super::pipeline_variants::PipelineVariant,
        samples: vk::SampleCountFlags,
        alpha_coverage: bool,
    ) -> Result<Self, RenderError> {
        Self::build(runtime, kind, None, samples, alpha_coverage, None)
    }
    pub fn new_native_alpha(
        runtime: &Arc<VulkanRuntime>,
        kind: super::pipeline_variants::PipelineVariant,
        samples: vk::SampleCountFlags,
        alpha_coverage: bool,
        layout: &Arc<super::oit_descriptor::OitDescriptorLayout>,
    ) -> Result<Self, RenderError> {
        Self::build(
            runtime,
            kind,
            None,
            samples,
            alpha_coverage,
            Some(layout.clone()),
        )
    }
    fn build(
        runtime: &Arc<VulkanRuntime>,
        kind: super::pipeline_variants::PipelineVariant,
        oit: Option<(
            super::oit_pipeline::OitPipelineTarget,
            Arc<super::oit_descriptor::OitDescriptorLayout>,
        )>,
        samples: vk::SampleCountFlags,
        alpha_coverage: bool,
        alpha_layout: Option<Arc<super::oit_descriptor::OitDescriptorLayout>>,
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
            kind,
            oit_layout: oit
                .as_ref()
                .map(|(_, layout)| layout.clone())
                .or_else(|| alpha_layout.clone()),
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
            fragment_bytes: &[],
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
        if kind.fixed_lighting {
            let shaders = if kind.mesh_edges {
                super::fixed_lighting_pipeline::fixed_mesh_triple(kind).to_vec()
            } else {
                super::fixed_lighting_pipeline::fixed_lighting_pair(kind).to_vec()
            };
            for bytes in shaders {
                pipeline.add_shader(bytes)?;
            }
        } else if kind.fixed_line_stipple && !kind.lighting {
            for bytes in super::fixed_lighting_pipeline::fixed_unlit_line_pair(kind.clipped) {
                pipeline.add_shader(bytes)?;
            }
        } else if kind.point && kind.encoded_surface && !kind.lighting {
            for bytes in super::fixed_lighting_pipeline::fixed_unlit_point_pair(kind.clipped) {
                pipeline.add_shader(bytes)?;
            }
        } else if kind.mesh_edges {
            for bytes in super::mesh_edge_pipeline::edge_shader_bytes(kind) {
                pipeline.add_shader(bytes)?;
            }
        } else if kind.surface_texture || kind.surface_alpha {
            for bytes in super::surface_texture_pipeline::surface_shaders(kind) {
                pipeline.add_shader(bytes)?;
            }
        } else {
            pipeline.load_shaders(line || kind.polygon_edges, point, text, clipped, lighting)?;
        }
        if let Some((target, _)) = &oit {
            pipeline.replace_oit_fragment(*target)?;
        }
        if alpha_layout.is_some() {
            pipeline.replace_fragment(super::native_alpha_shader::native_alpha_fragment(
                pipeline.fragment_bytes,
            )?)?;
        }
        pipeline.layout = create_layout(
            runtime,
            fixed_marker_push_range(
                line || kind.polygon_edges
                    || point
                    || text
                    || kind.surface_texture
                    || kind.surface_alpha,
                kind.mesh_edges,
                point && kind.encoded_surface,
            ),
            pipeline
                .sprite_layout
                .as_ref()
                .map(|v| v.handle)
                .or_else(|| pipeline.surface_layout.as_ref().map(|v| v.handle)),
            pipeline.lighting_layout.as_ref(),
            pipeline.clipping_layout.as_ref(),
            pipeline.oit_layout.as_ref().map(|layout| layout.handle),
        )?;
        pipeline.create(
            kind,
            oit.as_ref().map(|(target, _)| *target),
            samples,
            alpha_coverage,
        )?;
        Ok(pipeline)
    }
    fn replace_oit_fragment(
        &mut self,
        target: super::oit_pipeline::OitPipelineTarget,
    ) -> Result<(), RenderError> {
        let bytes = super::oit_pipeline::oit_fragment_bytes(self.kind, target)?;
        self.replace_fragment(bytes)
    }
    fn add_shader(&mut self, bytes: &'static [u8]) -> Result<(), RenderError> {
        if self.shaders.len() == 1 {
            self.fragment_bytes = bytes;
        }
        self.shaders.push(load_shader(&self.runtime, bytes)?);
        Ok(())
    }
    fn replace_fragment(&mut self, bytes: &'static [u8]) -> Result<(), RenderError> {
        let replacement = load_shader(&self.runtime, bytes)?;
        unsafe {
            self.runtime
                .device()
                .destroy_shader_module(self.shaders[1], None);
        }
        self.shaders[1] = replacement;
        self.fragment_bytes = bytes;
        Ok(())
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
            self.add_shader(bytes)?;
        }
        Ok(())
    }
    fn create(
        &mut self,
        kind: super::pipeline_variants::PipelineVariant,
        oit: Option<super::oit_pipeline::OitPipelineTarget>,
        samples: vk::SampleCountFlags,
        alpha_coverage: bool,
    ) -> Result<(), RenderError> {
        let stages = shader_stages(&self.shaders);
        let mut state = super::pipeline_state::PipelineState::new(kind);
        state.set_multisampling(samples, alpha_coverage);
        let color_format = vec![if kind.text_logic_xor || kind.encoded_surface {
            vk::Format::R8G8B8A8_UNORM
        } else {
            vk::Format::R8G8B8A8_SRGB
        }];
        let color_format = oit
            .map(|target| target.formats[..target.count].to_vec())
            .unwrap_or(color_format);
        let blends =
            oit.map(|target| super::oit_pipeline::geometry_blend_attachments(target, kind));
        let blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(blends.as_deref().unwrap_or(&[]));
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
        if oit.is_some() {
            info = info.color_blend_state(&blend_info);
        }
        let mut line_state = vk::PipelineRasterizationLineStateCreateInfoEXT::default()
            .line_rasterization_mode(vk::LineRasterizationModeEXT::BRESENHAM)
            .stippled_line_enable(kind.fixed_line_stipple)
            .line_stipple_factor(1)
            .line_stipple_pattern(0xffff);
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
pub(super) fn load_shader(
    runtime: &VulkanRuntime,
    bytes: &[u8],
) -> Result<vk::ShaderModule, RenderError> {
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
pub(super) fn shader_stages(
    shaders: &[vk::ShaderModule],
) -> Vec<vk::PipelineShaderStageCreateInfo<'static>> {
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
pub(super) fn create_pipeline(
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
include!("pipeline_tests.rs");
