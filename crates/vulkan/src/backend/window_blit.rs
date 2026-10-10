//! Copy the rendered image entirely on the GPU, preserving sRGB and top-left orientation.
use super::{frame::FrameResources, runtime::VulkanRuntime, window_chain::WindowChain};
use ash::vk;
pub(super) fn record(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    resources: &FrameResources,
    chain: &WindowChain,
    index: u32,
) {
    let image = chain.images[index as usize];
    transition(runtime, command, image, false);
    let layers = vk::ImageSubresourceLayers::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .layer_count(1);
    let region = vk::ImageBlit::default()
        .src_subresource(layers)
        .dst_subresource(layers)
        .src_offsets(offsets(resources.extent))
        .dst_offsets(offsets(chain.config.extent));
    unsafe {
        runtime.device().cmd_blit_image(
            command,
            resources.color.handle,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
            vk::Filter::NEAREST,
        )
    };
    transition(runtime, command, image, true);
}
fn offsets(extent: vk::Extent2D) -> [vk::Offset3D; 2] {
    [
        vk::Offset3D::default(),
        vk::Offset3D {
            x: extent.width as i32,
            y: extent.height as i32,
            z: 1,
        },
    ]
}
fn transition(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    image: vk::Image,
    present: bool,
) {
    let range = vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .level_count(1)
        .layer_count(1);
    let barrier = vk::ImageMemoryBarrier2::default()
        .image(image)
        .subresource_range(range)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED);
    let barrier = if present {
        barrier
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .src_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
    } else {
        barrier
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
            .dst_access_mask(vk::AccessFlags2::TRANSFER_WRITE)
    };
    unsafe {
        runtime.device().cmd_pipeline_barrier2(
            command,
            &vk::DependencyInfo::default().image_memory_barriers(&[barrier]),
        )
    };
}
#[cfg(test)]
mod tests {
    #[test]
    fn blit_covers_the_top_left_extent() {
        let result = super::offsets(ash::vk::Extent2D {
            width: 32,
            height: 16,
        });
        assert_eq!(
            (
                result[0].x,
                result[0].y,
                result[1].x,
                result[1].y,
                result[1].z
            ),
            (0, 0, 32, 16, 1)
        );
    }
}
