//! Synchronization2 upload and actual per-level blits precede any sampled-image access.
use super::{
    buffer::DeviceBuffer, memory_selection::HostBufferAccess,
    sampled_surface_image::SampledSurfaceImage, submission::CommandSubmission,
    surface_image_format::image_range,
};
use crate::{RenderError, surface_texture::SurfaceTextureImage};
use ash::vk;
/// Transfer actual source texels and mip buffers; `upload_sampled_image(image, source)?`.
pub(super) fn upload_sampled_image(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
) -> Result<(), RenderError> {
    let staging = stage_surface_texels(image, source)?;
    let submission = CommandSubmission::new(&image.runtime)?;
    record_surface_initial_copy(image, source, submission.command, staging.handle);
    finish_surface_mips(image, source, submission.command);
    submission.finish()
}
fn stage_surface_texels(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
) -> Result<DeviceBuffer, RenderError> {
    let staging = DeviceBuffer::new(
        &image.runtime,
        source.bytes.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        HostBufferAccess::Upload,
    )?;
    staging.write(&source.bytes)?;
    Ok(staging)
}
fn record_surface_initial_copy(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
    command: vk::CommandBuffer,
    staging: vk::Buffer,
) {
    let destination = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    transition(
        image,
        command,
        0,
        image.levels,
        vk::ImageLayout::UNDEFINED,
        destination,
    );
    let copies = copy_regions(image, source);
    unsafe {
        image.runtime.device().cmd_copy_buffer_to_image(
            command,
            staging,
            image.handle,
            destination,
            &copies,
        );
    }
}
fn finish_surface_mips(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
    command: vk::CommandBuffer,
) {
    if source.supplied_levels.is_empty() {
        generate_sampled_mips(image, command);
        return;
    }
    transition(
        image,
        command,
        0,
        image.levels,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
}
fn copy_regions(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
) -> Vec<vk::BufferImageCopy> {
    let lengths = if source.supplied_levels.is_empty() {
        vec![source.bytes.len()]
    } else {
        source.supplied_levels.clone()
    };
    let mut offset = 0;
    lengths
        .into_iter()
        .enumerate()
        .map(|(level, length)| {
            let result = surface_copy_region(image, level as u32, offset);
            offset += length as u64;
            result
        })
        .collect()
}
fn surface_copy_region(
    image: &SampledSurfaceImage,
    level: u32,
    offset: u64,
) -> vk::BufferImageCopy {
    vk::BufferImageCopy::default()
        .buffer_offset(offset)
        .image_subresource(surface_mip_layers(level))
        .image_extent(mip_extent(image.extent, level))
}
fn mip_extent(extent: vk::Extent3D, level: u32) -> vk::Extent3D {
    vk::Extent3D {
        width: (extent.width >> level).max(1),
        height: (extent.height >> level).max(1),
        depth: 1,
    }
}
fn surface_mip_layers(level: u32) -> vk::ImageSubresourceLayers {
    vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(level)
        .layer_count(1)
}
fn generate_sampled_mips(image: &SampledSurfaceImage, command: vk::CommandBuffer) {
    let destination = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    let source = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    let shader = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    for level in 1..image.levels {
        transition_level(image, command, level - 1, destination, source);
        record_surface_mip_blit(image, command, level);
        transition_level(image, command, level - 1, source, shader);
    }
    transition_level(image, command, image.levels - 1, destination, shader);
}
fn record_surface_mip_blit(image: &SampledSurfaceImage, command: vk::CommandBuffer, level: u32) {
    let source = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
    let destination = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    let blit = mip_blit(image, level);
    unsafe {
        image.runtime.device().cmd_blit_image(
            command,
            image.handle,
            source,
            image.handle,
            destination,
            &[blit],
            vk::Filter::LINEAR,
        );
    }
}
fn mip_blit(image: &SampledSurfaceImage, level: u32) -> vk::ImageBlit {
    let before = mip_extent(image.extent, level - 1);
    let after = mip_extent(image.extent, level);
    vk::ImageBlit::default()
        .src_subresource(surface_mip_layers(level - 1))
        .dst_subresource(surface_mip_layers(level))
        .src_offsets(mip_offsets(before))
        .dst_offsets(mip_offsets(after))
}
fn mip_offsets(extent: vk::Extent3D) -> [vk::Offset3D; 2] {
    [
        vk::Offset3D::default(),
        vk::Offset3D {
            x: extent.width as i32,
            y: extent.height as i32,
            z: 1,
        },
    ]
}
fn transition_level(
    image: &SampledSurfaceImage,
    command: vk::CommandBuffer,
    level: u32,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
) {
    transition(image, command, level, 1, old, new);
}
fn transition(
    image: &SampledSurfaceImage,
    command: vk::CommandBuffer,
    first: u32,
    count: u32,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
) {
    let barrier = surface_image_barrier(image, first, count, old, new);
    let dependency =
        vk::DependencyInfo::default().image_memory_barriers(std::slice::from_ref(&barrier));
    unsafe {
        image
            .runtime
            .device()
            .cmd_pipeline_barrier2(command, &dependency);
    }
}
fn surface_image_barrier(
    image: &SampledSurfaceImage,
    first: u32,
    count: u32,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
) -> vk::ImageMemoryBarrier2<'static> {
    let (source_stage, source_access) = surface_transfer_access(old);
    let (destination_stage, destination_access) = surface_transfer_access(new);
    vk::ImageMemoryBarrier2::default()
        .image(image.handle)
        .subresource_range(image_range(first, count))
        .old_layout(old)
        .new_layout(new)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .src_stage_mask(source_stage)
        .src_access_mask(source_access)
        .dst_stage_mask(destination_stage)
        .dst_access_mask(destination_access)
}
fn surface_transfer_access(layout: vk::ImageLayout) -> (vk::PipelineStageFlags2, vk::AccessFlags2) {
    match layout {
        vk::ImageLayout::UNDEFINED => (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE),
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => (
            vk::PipelineStageFlags2::FRAGMENT_SHADER,
            vk::AccessFlags2::SHADER_SAMPLED_READ,
        ),
        _ => (
            vk::PipelineStageFlags2::TRANSFER,
            vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE,
        ),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn odd_mip_dimensions_floor_to_one() {
        let extent = vk::Extent3D {
            width: 7,
            height: 3,
            depth: 1,
        };
        assert_eq!(
            mip_extent(extent, 1),
            vk::Extent3D {
                width: 3,
                height: 1,
                depth: 1
            }
        );
        assert_eq!(
            mip_extent(extent, 2),
            vk::Extent3D {
                width: 1,
                height: 1,
                depth: 1
            }
        );
        assert_eq!(mip_offsets(extent)[1], vk::Offset3D { x: 7, y: 3, z: 1 });
    }
    #[test]
    fn transfer_access_reaches_shader_sampling() {
        assert_eq!(surface_mip_layers(2).mip_level, 2);
        assert_eq!(
            surface_transfer_access(vk::ImageLayout::UNDEFINED),
            (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE)
        );
        assert_eq!(
            surface_transfer_access(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
            (
                vk::PipelineStageFlags2::FRAGMENT_SHADER,
                vk::AccessFlags2::SHADER_SAMPLED_READ
            )
        );
    }
}
