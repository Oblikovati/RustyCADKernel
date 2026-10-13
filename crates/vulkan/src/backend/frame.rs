//! Frame resources are private and never outlive the device that created them.
use super::transform_constants::encode_transform;
use super::{
    buffer::DeviceBuffer, image::TargetImage, memory_selection::HostBufferAccess,
    pipeline::DrawPipeline, pipeline_variants::RasterPipelines, runtime::VulkanRuntime,
};
use crate::{FrameCapture, PrimitiveDraw, RenderError, RenderFrame, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct UploadedDraw {
    pub native_alpha: Option<Arc<super::oit_descriptor::OitPassDescriptors>>,
    pub oit_alpha: Vec<(
        vk::DescriptorSet,
        Arc<super::oit_descriptor::OitPassDescriptors>,
    )>,
    pub sample_alpha_cutoff: bool,
    pub oit: Option<super::oit_upload::OitUploadedVariants>,
    pub vertices: DeviceBuffer,
    pub indices: DeviceBuffer,
    pub count: u32,
    pub transform: Vec<u8>,
    pub pipeline: Arc<DrawPipeline>,
    pub line_width: Option<f32>,
    pub line_stipple: Option<(u32, u16)>,
    pub annotation: bool,
    pub text_shadow_bias: bool,
    pub text_logic_xor: bool,
    pub encoded_surface: bool,
    pub polygon_offset: Option<crate::RenderPolygonOffset>,
    pub polygon: bool,
    pub polygon_edge: bool,
    pub surface_blend: bool,
    pub capping: crate::RenderCappingPass,
    pub mirrored: bool,
    pub face_culling: crate::RenderFaceCulling,
    pub clipping: Option<super::storage_descriptor::UploadedStorage>,
    pub lighting: Option<super::storage_descriptor::UploadedStorage>,
    pub sprite: Option<super::storage_descriptor::UploadedStorage>,
    pub surface_image: Option<Arc<super::surface_image_descriptor::SurfaceImageBinding>>,
}
pub(super) struct FrameResources {
    pub fragment_alpha: super::fragment_alpha_plan::FragmentAlphaPlan,
    pub multisample: Option<super::multisample_targets::MultisampleTargets>,
    pub oit: Option<super::oit_frame::OitFramePass>,
    pub extent: vk::Extent2D,
    pub color: TargetImage,
    pub depth: TargetImage,
    pub color_read: DeviceBuffer,
    pub depth_read: DeviceBuffer,
    pub draws: Vec<UploadedDraw>,
}
impl FrameResources {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        frame: &RenderFrame,
        pipeline: &RasterPipelines,
        initial_alpha: crate::legacy_fragment_alpha::LegacyFragmentAlpha,
    ) -> Result<Self, RenderError> {
        let fragment_alpha =
            super::fragment_alpha_plan::FragmentAlphaPlan::new(frame, initial_alpha);
        pipeline.surface_images.prune()?;
        check_extent(runtime, frame)?;
        let extent = vk::Extent2D {
            width: frame.width,
            height: frame.height,
        };
        let bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
        let plan = super::frame_raster_plan::frame_raster_plan(runtime, frame, pipeline.stencil);
        let mut oit = super::oit_frame::OitFramePass::new(runtime, frame, pipeline, extent, plan)?;
        if let Some(pass) = &mut oit {
            pass.prepare_alpha(&fragment_alpha)?;
        }
        let mut draws = frame
            .draws
            .iter()
            .map(|draw| upload_draw(runtime, draw, pipeline, extent))
            .collect::<Result<Vec<_>, _>>()?;
        for draw in &mut draws {
            draw.pipeline = pipeline.oit.multisample(
                runtime,
                &draw.pipeline,
                plan.samples,
                frame.raster.alpha_to_coverage && draw.sample_alpha_cutoff,
            )?;
        }
        if let Some(pass) = &oit {
            super::oit_upload::upload_oit_variants(runtime, pipeline, &mut draws, pass)?;
        }
        super::fragment_alpha_upload::upload_fragment_alpha(
            runtime,
            pipeline,
            &mut draws,
            &fragment_alpha,
            oit.as_ref(),
            plan.samples,
            frame.raster.alpha_to_coverage,
        )?;
        Ok(Self {
            fragment_alpha,
            multisample: super::multisample_targets::MultisampleTargets::new(
                runtime,
                extent,
                pipeline.stencil,
                plan.samples,
            )?,
            oit,
            extent,
            color: TargetImage::new(runtime, extent, false, false)?,
            depth: TargetImage::new(runtime, extent, true, pipeline.stencil)?,
            color_read: DeviceBuffer::new(
                runtime,
                bytes,
                vk::BufferUsageFlags::TRANSFER_DST,
                HostBufferAccess::Readback,
            )?,
            depth_read: DeviceBuffer::new(
                runtime,
                bytes,
                vk::BufferUsageFlags::TRANSFER_DST,
                HostBufferAccess::Readback,
            )?,
            draws,
        })
    }
    pub fn capture(&self) -> Result<FrameCapture, RenderError> {
        let size = self.extent.width as usize * self.extent.height as usize * 4;
        let rgba = self.color_read.read(size)?;
        let bytes = self.depth_read.read(size)?;
        let depth = bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|value| f32::from_ne_bytes(*value))
            .collect();
        Ok(FrameCapture {
            width: self.extent.width,
            height: self.extent.height,
            rgba,
            depth,
        })
    }
    pub fn render_color(&self) -> &TargetImage {
        self.multisample
            .as_ref()
            .map_or(&self.color, |targets| &targets.color)
    }
    pub fn render_depth(&self) -> &TargetImage {
        self.multisample
            .as_ref()
            .map_or(&self.depth, |targets| &targets.depth)
    }
}
fn check_extent(runtime: &VulkanRuntime, frame: &RenderFrame) -> Result<(), RenderError> {
    if frame.width > runtime.limits.max_framebuffer_width
        || frame.height > runtime.limits.max_framebuffer_height
        || frame.width > runtime.limits.max_image_dimension2_d
        || frame.height > runtime.limits.max_image_dimension2_d
    {
        return Err(rejected(
            [frame.width, frame.height],
            "extent within device image and framebuffer limits",
        ));
    }
    Ok(())
}
fn upload_draw(
    runtime: &Arc<VulkanRuntime>,
    draw: &PrimitiveDraw,
    pipelines: &RasterPipelines,
    extent: vk::Extent2D,
) -> Result<UploadedDraw, RenderError> {
    if let Some(line) = draw.line_style.or(draw.polygon_edge_style) {
        runtime
            .lines
            .validate(line, runtime.limits.line_width_range)?;
    }
    if let Some(style) = &draw.point_style {
        super::point_sprite::validate_point_size(
            style.size,
            runtime.large_points,
            runtime.limits.point_size_range,
        )?;
    }
    let original_indices = crate::quad_assembly::device_quad_indices(draw, runtime.quad_diagonal);
    let sources = super::fixed_vertex_sources::fixed_vertex_sources(draw, &original_indices);
    let vertices = encode_vertex_sources(draw, &sources);
    let expanded = super::fixed_vertex_sources::fixed_expanded_indices(draw, sources.len());
    let indices: Vec<u8> = expanded
        .as_deref()
        .unwrap_or(&original_indices)
        .iter()
        .flat_map(|i| i.to_ne_bytes())
        .collect();
    let vertex_buffer = upload_bytes(runtime, &vertices, vk::BufferUsageFlags::VERTEX_BUFFER)?;
    let index_buffer = upload_bytes(runtime, &indices, vk::BufferUsageFlags::INDEX_BUFFER)?;
    let count = u32::try_from(draw.indices.len())
        .map_err(|_| rejected(draw.indices.len(), "a 32-bit index count"))?;
    let pipeline = pipelines.select(draw)?;
    let image = draw
        .point_style
        .as_ref()
        .map(|s| &s.image)
        .or_else(|| draw.textured_style.as_ref().map(|s| &s.image));
    let sprite = image
        .map(|image| {
            super::point_sprite::upload_sprite(
                runtime,
                pipeline
                    .sprite_layout
                    .as_ref()
                    .expect("sprite pipeline layout"),
                image,
            )
        })
        .transpose()?;
    if let Some(unavailable) = &draw.surface_texture_unavailable {
        pipelines.surface_images.forget(&unavailable.resource_id)?;
    }
    if draw.surface_texture_binding_only
        && let Some(style) = &draw.surface_texture
    {
        pipelines.surface_images.touch(
            runtime,
            &style.image,
            crate::fixed_surface_environment::uses_fixed_surface_environment(draw),
        )?;
    }
    let surface_image = draw
        .surface_texture
        .as_ref()
        .filter(|_| !draw.surface_texture_binding_only)
        .map(|style| {
            pipelines.surface_images.binding(
                runtime,
                pipeline
                    .surface_layout
                    .as_ref()
                    .expect("surface image layout"),
                style,
                crate::fixed_surface_environment::uses_fixed_surface_environment(draw),
            )
        })
        .transpose()?;
    let clipping = draw
        .clipping
        .as_ref()
        .map(|clipping| {
            super::storage_descriptor::UploadedStorage::new(
                runtime,
                pipeline
                    .clipping_layout
                    .as_ref()
                    .expect("clipping pipeline layout"),
                &clipping.encode(),
            )
        })
        .transpose()?;
    let lighting = draw
        .lighting
        .as_ref()
        .map(|lighting| {
            super::storage_descriptor::UploadedStorage::new(
                runtime,
                pipeline
                    .lighting_layout
                    .as_ref()
                    .expect("lighting pipeline layout"),
                &lighting.encode(),
            )
        })
        .transpose()?;
    Ok(UploadedDraw {
        native_alpha: None,
        oit_alpha: vec![],
        sample_alpha_cutoff: draw
            .native_material_cutoff
            .is_some_and(|cutoff| cutoff <= 1.)
            || draw.surface_cutoff.is_some(),
        oit: None,
        lighting,
        clipping,
        sprite,
        surface_image,
        annotation: draw.annotation,
        text_shadow_bias: draw.text_shadow_bias,
        text_logic_xor: draw.text_logic_xor,
        encoded_surface: super::pipeline_variants::encoded_surface_attachment(draw),
        polygon_offset: draw.polygon_offset,
        // Native font glyph triangles retain ordinary Fill slope/units; OpenGl_Text at 3d097a0.
        polygon: draw.topology() == crate::PrimitiveTopology::Triangles,
        polygon_edge: draw.polygon_edge_style.is_some(),
        surface_blend: draw.surface_blend,
        capping: draw.capping_pass,
        face_culling: draw.face_culling,
        mirrored: draw.mirrored,
        pipeline,
        vertices: vertex_buffer,
        indices: index_buffer,
        count,
        line_stipple: crate::fixed_line_stipple::fixed_line_stipple(draw),
        line_width: draw
            .line_style
            .or(draw.polygon_edge_style)
            .map(|style| style.raster_width()),
        transform: encode_transform(draw, extent),
    })
}

fn upload_bytes(
    runtime: &Arc<VulkanRuntime>,
    bytes: &[u8],
    usage: vk::BufferUsageFlags,
) -> Result<DeviceBuffer, RenderError> {
    let buffer = DeviceBuffer::new(runtime, bytes.len() as u64, usage, HostBufferAccess::Upload)?;
    buffer.write(bytes)?;
    Ok(buffer)
}

#[cfg(test)]
fn encode_vertices(draw: &PrimitiveDraw) -> Vec<u8> {
    let sources: Vec<_> = (0..draw.vertices.len())
        .map(|index| (index, index))
        .collect();
    encode_vertex_sources(draw, &sources)
}

#[cfg(test)]
mod fixed_pack_tests {
    use super::*;
    #[test]
    fn fixed_provoking_position_keeps_homogeneous_weight_and_full_stride() {
        let vertex = crate::RenderVertex::homogeneous([1., 2., 3., 4.], [0.25, 0.5, 0.75, 1.]);
        let mut draw =
            PrimitiveDraw::new(vec![vertex; 3], vec![0, 1, 2], crate::identity_clip()).unwrap();
        let mut lighting = crate::RenderLighting::new(
            vec![[0., 0., 1.]; 3],
            crate::RenderMaterial::default(),
            crate::identity_clip(),
            crate::identity_clip(),
            false,
            crate::RenderShading::Phong,
        )
        .unwrap();
        lighting.prepare_fixed_lighting().unwrap();
        draw.lighting = Some(lighting);
        let bytes = encode_vertices(&draw);
        assert_eq!(bytes.len(), 3 * 88);
        assert_eq!(f32::from_ne_bytes(bytes[56..60].try_into().unwrap()), 4.);
        assert_eq!(f32::from_ne_bytes(bytes[72..76].try_into().unwrap()), 0.25);
    }
}
fn encode_vertex_sources(draw: &PrimitiveDraw, sources: &[(usize, usize)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for &(index, source) in sources {
        let vertex = &draw.vertices[index];
        bytes.extend(encoded_vertex_header(
            vertex,
            draw.text_logic_xor && !draw.compatibility_color_target,
            draw.compatibility_color_target && draw.textured_style.is_some(),
        ));
        if let Some(style) = &draw.textured_style {
            bytes.extend(style.uv[index].iter().flat_map(|v| v.to_ne_bytes()));
        }
        append_surface_coordinates(&mut bytes, draw, index);
        if let Some(lighting) = &draw.lighting {
            bytes.extend(lighting.normals[index].iter().flat_map(|v| v.to_ne_bytes()));
            if lighting.is_fixed_lighting() {
                let selected = &draw.vertices[source];
                bytes.extend(
                    selected
                        .position
                        .into_iter()
                        .chain(std::iter::once(selected.weight))
                        .chain(lighting.normals[source])
                        .chain(selected.color)
                        .flat_map(f32::to_ne_bytes),
                );
            }
        }
    }
    bytes
}
fn append_surface_coordinates(bytes: &mut Vec<u8>, draw: &PrimitiveDraw, index: usize) {
    if draw.surface_texture_binding_only
        || (draw.surface_texture.is_none()
            && draw.surface_cutoff.is_none()
            && draw.surface_texture_unavailable.is_none()
            && draw.mesh_edges.is_none())
    {
        return;
    }
    let uv = draw
        .surface_texture
        .as_ref()
        .map_or([0., 0., 0., 1.], |style| style.uv[index]);
    bytes.extend(uv.iter().flat_map(|value| value.to_ne_bytes()));
}

fn encoded_vertex_header(vertex: &crate::RenderVertex, xor: bool, font_replace: bool) -> Vec<u8> {
    // Native framebuffer conversion precedes XOR; public submissions remain linear RGBA.
    let mut color = crate::text_logic::text_target_rgba(xor, vertex.color);
    // OCCT 3d097a0 OpenGl_Font.cxx133 + Sampler.cxx279: ALPHA/REPLACE ignores vertex alpha.
    if font_replace {
        color[3] = 1.;
    }
    vertex
        .position
        .iter()
        .chain(std::iter::once(&vertex.weight))
        .chain(&color)
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

#[cfg(test)]
mod xor_tests {
    use super::*;
    #[test]
    fn sampled_polygon_alpha_constants_preserve_surface_layout_before_line_parameters() {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        draw.polygon_edge_style = Some(crate::LineStyle::new(3., 0xff00, 2).unwrap());
        draw.surface_cutoff = Some(0.285);
        let bytes = encode_transform(
            &draw,
            vk::Extent2D {
                width: 64,
                height: 64,
            },
        );
        assert_eq!(bytes.len(), 80);
        assert_eq!(&bytes[64..68], &0.285_f32.to_ne_bytes());
        assert_eq!(&bytes[68..72], &8_u32.to_ne_bytes());
        draw.surface_texture_binding_only = true;
        let line = encode_transform(
            &draw,
            vk::Extent2D {
                width: 64,
                height: 64,
            },
        );
        assert_eq!(&line[72..76], &0xff00_u32.to_ne_bytes());
        assert_eq!(&line[76..80], &2_u32.to_ne_bytes());
    }
    #[test]
    fn fixed_font_alpha_replace_keeps_source_color_and_ordinary_alpha_unchanged() {
        let vertex = crate::RenderVertex::new([0.; 3], [0.25, 0.5, 0.75, 0.5]);
        let ordinary = encoded_vertex_header(&vertex, false, false);
        let compatibility = encoded_vertex_header(&vertex, false, true);
        assert_eq!(packed_vertex_rgba(&ordinary), vertex.color);
        assert_eq!(packed_vertex_rgba(&compatibility), [0.25, 0.5, 0.75, 1.]);
        assert_eq!(vertex.color[3], 0.5);
    }
    #[test]
    fn xor_pack_encodes_only_upload_rgb_and_preserves_linear_submission_and_alpha() {
        let color = [0.18, 0.4, 0.7, 0.4];
        let vertices = vec![crate::RenderVertex::new([0.; 3], color); 3];
        let mut draw = PrimitiveDraw::new(vertices, vec![0, 1, 2], crate::identity_clip()).unwrap();
        for xor in [false, true] {
            draw.text_logic_xor = xor;
            let encoded = encode_vertices(&draw);
            let packed = packed_vertex_rgba(&encoded);
            assert_eq!(packed, crate::text_logic::text_target_rgba(xor, color));
            assert_eq!(draw.vertices()[0].color, color);
            assert_eq!(packed[3], 0.4);
        }
    }
    fn packed_vertex_rgba(bytes: &[u8]) -> [f32; 4] {
        std::array::from_fn(|index| {
            let slice = &bytes[16 + index * 4..20 + index * 4];
            f32::from_ne_bytes(slice.try_into().unwrap())
        })
    }
    #[test]
    fn unavailable_surface_and_alpha_only_pack_projective_default_coordinates() {
        let mut draw = PrimitiveDraw::new(vec![], vec![], crate::identity_clip()).unwrap();
        let mut bytes = vec![];
        append_surface_coordinates(&mut bytes, &draw, 0);
        assert!(bytes.is_empty());
        draw.surface_cutoff = Some(0.5);
        append_surface_coordinates(&mut bytes, &draw, 0);
        let expected: Vec<u8> = [0_f32, 0., 0., 1.]
            .iter()
            .flat_map(|value| value.to_ne_bytes())
            .collect();
        assert_eq!(bytes, expected);
    }
}
