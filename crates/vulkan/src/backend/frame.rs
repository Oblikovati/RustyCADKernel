//! Frame resources are private and never outlive the device that created them.
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
    pub sprite: Option<super::point_sprite::UploadedSprite>,
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
        check_extent(runtime, frame)?;
        let extent = vk::Extent2D {
            width: frame.width,
            height: frame.height,
        };
        let bytes = u64::from(frame.width) * u64::from(frame.height) * 4;
        Ok(Self {
            extent,
            color: TargetImage::new(runtime, extent, false)?,
            depth: TargetImage::new(runtime, extent, true)?,
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
    if let Some(line) = draw.line_style {
        runtime
            .lines
            .validate(line, runtime.limits.line_width_range)?;
    }
    let vertices = encode_vertices(draw);
    let indices: Vec<u8> = draw.indices.iter().flat_map(|i| i.to_ne_bytes()).collect();
    let vertex_buffer = upload_bytes(runtime, &vertices, vk::BufferUsageFlags::VERTEX_BUFFER)?;
    let index_buffer = upload_bytes(runtime, &indices, vk::BufferUsageFlags::INDEX_BUFFER)?;
    let count = u32::try_from(draw.indices.len())
        .map_err(|_| rejected(draw.indices.len(), "a 32-bit index count"))?;
    let pipeline = pipelines.select(
        draw.depth_clamp,
        draw.line_style.is_some(),
        draw.point_style.is_some(),
    )?;
    let sprite = draw
        .point_style
        .as_ref()
        .map(|style| {
            super::point_sprite::UploadedSprite::new(
                runtime,
                pipeline
                    .sprite_layout
                    .as_ref()
                    .expect("point pipeline layout"),
                style,
            )
        })
        .transpose()?;
    Ok(UploadedDraw {
        sprite,
        pipeline,
        vertices: vertex_buffer,
        indices: index_buffer,
        count,
        line_width: draw.line_style.map(|style| style.width),
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
    draw.vertices
        .iter()
        .flat_map(|v| v.position.iter().chain(&v.color))
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}

fn encode_transform(draw: &PrimitiveDraw, extent: vk::Extent2D) -> Vec<u8> {
    let mut bytes: Vec<u8> = draw
        .clip_transform
        .iter()
        .flat_map(|v| v.to_ne_bytes())
        .collect();
    if let Some(style) = draw.line_style {
        bytes.extend(
            [extent.width as f32, extent.height as f32]
                .iter()
                .flat_map(|v| v.to_ne_bytes()),
        );
        bytes.extend(u32::from(style.pattern).to_ne_bytes());
        bytes.extend(u32::from(style.factor).to_ne_bytes());
    }
    if let Some(style) = &draw.point_style {
        bytes.extend(style.size.to_ne_bytes());
        bytes.extend(style.extent.into_iter().flat_map(|v| v.to_ne_bytes()));
        bytes.extend(0u32.to_ne_bytes());
    }
    bytes
}
