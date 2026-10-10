//! Frame resources are private and never outlive the device that created them.
use super::{
    buffer::DeviceBuffer, image::TargetImage, pipeline::TrianglePipeline, runtime::VulkanRuntime,
};
use crate::{FrameCapture, RenderError, RenderFrame, TriangleDraw, error::rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct UploadedDraw {
    pub vertices: DeviceBuffer,
    pub indices: DeviceBuffer,
    pub count: u32,
    pub transform: Vec<u8>,
}
pub(super) struct FrameResources {
    pub extent: vk::Extent2D,
    pub color: TargetImage,
    pub depth: TargetImage,
    pub color_read: DeviceBuffer,
    pub depth_read: DeviceBuffer,
    pub pipeline: Arc<TrianglePipeline>,
    pub draws: Vec<UploadedDraw>,
}
impl FrameResources {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        frame: &RenderFrame,
        pipeline: &Arc<TrianglePipeline>,
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
            color_read: DeviceBuffer::new(runtime, bytes, vk::BufferUsageFlags::TRANSFER_DST)?,
            depth_read: DeviceBuffer::new(runtime, bytes, vk::BufferUsageFlags::TRANSFER_DST)?,
            pipeline: pipeline.clone(),
            draws: frame
                .draws
                .iter()
                .map(|draw| upload_draw(runtime, draw))
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
    draw: &TriangleDraw,
) -> Result<UploadedDraw, RenderError> {
    let vertices = encode_vertices(draw);
    let indices: Vec<u8> = draw.indices.iter().flat_map(|i| i.to_ne_bytes()).collect();
    let vertex_buffer = upload_bytes(runtime, &vertices, vk::BufferUsageFlags::VERTEX_BUFFER)?;
    let index_buffer = upload_bytes(runtime, &indices, vk::BufferUsageFlags::INDEX_BUFFER)?;
    let count = u32::try_from(draw.indices.len())
        .map_err(|_| rejected(draw.indices.len(), "a 32-bit index count"))?;
    Ok(UploadedDraw {
        vertices: vertex_buffer,
        indices: index_buffer,
        count,
        transform: draw
            .clip_transform
            .iter()
            .flat_map(|v| v.to_ne_bytes())
            .collect(),
    })
}

fn upload_bytes(
    runtime: &Arc<VulkanRuntime>,
    bytes: &[u8],
    usage: vk::BufferUsageFlags,
) -> Result<DeviceBuffer, RenderError> {
    let buffer = DeviceBuffer::new(runtime, bytes.len() as u64, usage)?;
    buffer.write(bytes)?;
    Ok(buffer)
}

fn encode_vertices(draw: &TriangleDraw) -> Vec<u8> {
    draw.vertices
        .iter()
        .flat_map(|v| v.position.iter().chain(&v.color))
        .flat_map(|value| value.to_ne_bytes())
        .collect()
}
