//! Ping-pong images keep explicit layout history across transparent depth-clear batches.
use super::{image::TargetImage, runtime::VulkanRuntime};
use ash::{vk, vk::Handle};
use std::collections::BTreeMap;

#[derive(Default)]
pub(super) struct OitAttachmentLayouts(BTreeMap<u64, vk::ImageLayout>);
impl OitAttachmentLayouts {
    pub fn transition(
        &mut self,
        runtime: &VulkanRuntime,
        command: vk::CommandBuffer,
        image: &TargetImage,
        layout: vk::ImageLayout,
    ) {
        let old = self.capture_transition(image.handle.as_raw(), layout);
        let barrier = oit_image_barrier(image, old, layout);
        unsafe {
            runtime.device().cmd_pipeline_barrier2(
                command,
                &vk::DependencyInfo::default().image_memory_barriers(&[barrier]),
            );
        }
    }
    fn capture_transition(&mut self, image: u64, layout: vk::ImageLayout) -> vk::ImageLayout {
        self.0
            .insert(image, layout)
            .unwrap_or(vk::ImageLayout::UNDEFINED)
    }
}
fn oit_image_barrier(
    image: &TargetImage,
    old: vk::ImageLayout,
    new: vk::ImageLayout,
) -> vk::ImageMemoryBarrier2<'static> {
    let initialized = old != vk::ImageLayout::UNDEFINED;
    vk::ImageMemoryBarrier2::default()
        .image(image.handle)
        .subresource_range(image.range())
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .old_layout(old)
        .new_layout(new)
        .src_stage_mask(if initialized {
            vk::PipelineStageFlags2::ALL_GRAPHICS
        } else {
            vk::PipelineStageFlags2::NONE
        })
        .src_access_mask(if initialized {
            vk::AccessFlags2::COLOR_ATTACHMENT_WRITE | vk::AccessFlags2::SHADER_SAMPLED_READ
        } else {
            vk::AccessFlags2::NONE
        })
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        .dst_access_mask(if new == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
            vk::AccessFlags2::SHADER_SAMPLED_READ
        } else {
            vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_batches_preserve_old_sampled_layout_instead_of_discarding_history() {
        let mut layouts = OitAttachmentLayouts::default();
        assert_eq!(
            layouts.capture_transition(1, vk::ImageLayout::ATTACHMENT_OPTIMAL),
            vk::ImageLayout::UNDEFINED
        );
        assert_eq!(
            layouts.capture_transition(1, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
            vk::ImageLayout::ATTACHMENT_OPTIMAL
        );
        assert_eq!(
            layouts.capture_transition(2, vk::ImageLayout::ATTACHMENT_OPTIMAL),
            vk::ImageLayout::UNDEFINED
        );
        assert_eq!(
            layouts.capture_transition(1, vk::ImageLayout::ATTACHMENT_OPTIMAL),
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        );
    }
}
