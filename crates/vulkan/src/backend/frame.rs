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
    pub vertices: DeviceBuffer,
    pub indices: DeviceBuffer,
    pub count: u32,
    pub transform: Vec<u8>,
    pub pipeline: Arc<DrawPipeline>,
    pub line_width: Option<f32>,
    pub annotation: bool,
    pub text_shadow_bias: bool,
    pub text_logic_xor: bool,
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
    ) -> Result<Self, RenderError> {
        pipeline.surface_images.prune()?;
        check_extent(runtime, frame)?;
        let extent = vk::Extent2D {
            width: frame.width,
            height: frame.height,
        };
        let bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
        Ok(Self {
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
            draws: frame
                .draws
                .iter()
                .map(|draw| upload_draw(runtime, draw, pipeline, extent))
                .collect::<Result<_, _>>()?,
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
    let vertices = encode_vertices(draw);
    let indices: Vec<u8> = crate::quad_assembly::device_quad_indices(draw, runtime.quad_diagonal)
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
        pipelines.surface_images.touch(runtime, &style.image)?;
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
        lighting,
        clipping,
        sprite,
        surface_image,
        annotation: draw.annotation,
        text_shadow_bias: draw.text_shadow_bias,
        text_logic_xor: draw.text_logic_xor,
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

fn encode_vertices(draw: &PrimitiveDraw) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (index, vertex) in draw.vertices.iter().enumerate() {
        bytes.extend(encoded_vertex_header(vertex, draw.text_logic_xor));
        if let Some(style) = &draw.textured_style {
            bytes.extend(style.uv[index].iter().flat_map(|v| v.to_ne_bytes()));
        }
        append_surface_coordinates(&mut bytes, draw, index);
        if let Some(lighting) = &draw.lighting {
            bytes.extend(lighting.normals[index].iter().flat_map(|v| v.to_ne_bytes()));
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

fn encoded_vertex_header(vertex: &crate::RenderVertex, xor: bool) -> Vec<u8> {
    // Native framebuffer conversion precedes XOR; public submissions remain linear RGBA.
    let color = crate::text_logic::text_target_rgba(xor, vertex.color);
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
