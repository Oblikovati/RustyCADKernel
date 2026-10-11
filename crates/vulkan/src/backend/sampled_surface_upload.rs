//! Synchronization2 upload and actual per-level blits precede any sampled-image access.
use super::{
    buffer::DeviceBuffer, memory_selection::HostBufferAccess,
    sampled_surface_image::SampledSurfaceImage, submission::CommandSubmission,
    surface_image_format::image_range,
};
use crate::{RenderError, surface_texture::SurfaceTextureImage};
use ash::vk;
pub(super) fn upload_sampled_image(
    image: &SampledSurfaceImage,
    source: &SurfaceTextureImage,
) -> Result<(), RenderError> {
    let runtime = &image.runtime;
    let staging = DeviceBuffer::new(
        runtime,
        source.bytes.len() as u64,
        vk::BufferUsageFlags::TRANSFER_SRC,
        HostBufferAccess::Upload,
    )?;
    staging.write(&source.bytes)?;
    let submission = CommandSubmission::new(runtime)?;
    let command = submission.command;
    transition(
        image,
        command,
        0,
        image.levels,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    );
    let copies = copy_regions(image, source);
    unsafe {
        runtime.device().cmd_copy_buffer_to_image(
            command,
            staging.handle,
            image.handle,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &copies,
        );
    }
    if source.supplied_levels.is_empty() {
        generate_sampled_mips(image, command);
    } else {
        transition(
            image,
            command,
            0,
            image.levels,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
    }
    submission.finish()
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
            let result = vk::BufferImageCopy::default()
                .buffer_offset(offset)
                .image_subresource(
                    vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .mip_level(level as u32)
                        .layer_count(1),
                )
                .image_extent(mip_extent(image.extent, level as u32));
            offset += length as u64;
            result
        })
        .collect()
}
fn mip_extent(extent: vk::Extent3D, level: u32) -> vk::Extent3D {
    vk::Extent3D {
        width: (extent.width >> level).max(1),
        height: (extent.height >> level).max(1),
        depth: 1,
    }
}
fn generate_sampled_mips(image: &SampledSurfaceImage, command: vk::CommandBuffer) {
    for level in 1..image.levels {
        transition(
            image,
            command,
            level - 1,
            1,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );
        let blit = mip_blit(image, level);
        unsafe {
            image.runtime.device().cmd_blit_image(
                command,
                image.handle,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image.handle,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[blit],
                vk::Filter::LINEAR,
            );
        }
        transition(
            image,
            command,
            level - 1,
            1,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
    }
    transition(
        image,
        command,
        image.levels - 1,
        1,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
}
fn mip_blit(image: &SampledSurfaceImage, level: u32) -> vk::ImageBlit {
    let before = mip_extent(image.extent, level - 1);
    let after = mip_extent(image.extent, level);
    let layers = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .layer_count(1);
    vk::ImageBlit::default()
        .src_subresource(layers.mip_level(level - 1))
        .dst_subresource(layers.mip_level(level))
        .src_offsets([
            vk::Offset3D::default(),
            vk::Offset3D {
                x: before.width as i32,
                y: before.height as i32,
                z: 1,
            },
        ])
        .dst_offsets([
            vk::Offset3D::default(),
            vk::Offset3D {
                x: after.width as i32,
                y: after.height as i32,
                z: 1,
            },
        ])
}
fn transition(
    image: &SampledSurfaceImage,
    command: vk::CommandBuffer,
    first: u32,
    count: u32,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
) {
    let shader = new == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    let barrier = vk::ImageMemoryBarrier2::default()
        .image(image.handle)
        .subresource_range(image_range(first, count))
        .old_layout(old)
        .new_layout(new)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .src_stage_mask(if old == vk::ImageLayout::UNDEFINED {
            vk::PipelineStageFlags2::NONE
        } else {
            vk::PipelineStageFlags2::TRANSFER
        })
        .src_access_mask(if old == vk::ImageLayout::UNDEFINED {
            vk::AccessFlags2::NONE
        } else {
            vk::AccessFlags2::TRANSFER_WRITE | vk::AccessFlags2::TRANSFER_READ
        })
        .dst_stage_mask(if shader {
            vk::PipelineStageFlags2::FRAGMENT_SHADER
        } else {
            vk::PipelineStageFlags2::TRANSFER
        })
        .dst_access_mask(if shader {
            vk::AccessFlags2::SHADER_SAMPLED_READ
        } else {
            vk::AccessFlags2::TRANSFER_READ | vk::AccessFlags2::TRANSFER_WRITE
        });
    unsafe {
        image.runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().image_memory_barriers(&[barrier]),
        );
    }
}
