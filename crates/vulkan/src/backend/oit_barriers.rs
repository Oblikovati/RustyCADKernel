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
    let (stage, access) = prior_attachment_access(old);
    vk::ImageMemoryBarrier2::default()
        .image(image.handle)
        .subresource_range(image.range())
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .old_layout(old)
        .new_layout(new)
        .src_stage_mask(stage)
        .src_access_mask(access)
        .dst_stage_mask(vk::PipelineStageFlags2::ALL_GRAPHICS)
        .dst_access_mask(next_attachment_access(new))
}
fn prior_attachment_access(layout: vk::ImageLayout) -> (vk::PipelineStageFlags2, vk::AccessFlags2) {
    if layout == vk::ImageLayout::UNDEFINED {
        return (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE);
    }
    (
        vk::PipelineStageFlags2::ALL_GRAPHICS,
        vk::AccessFlags2::COLOR_ATTACHMENT_WRITE | vk::AccessFlags2::SHADER_SAMPLED_READ,
    )
}
fn next_attachment_access(layout: vk::ImageLayout) -> vk::AccessFlags2 {
    if layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL {
        return vk::AccessFlags2::SHADER_SAMPLED_READ;
    }
    vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layout_access_preserves_initialized_write_to_sampled_visibility() {
        assert_eq!(
            prior_attachment_access(vk::ImageLayout::UNDEFINED),
            (vk::PipelineStageFlags2::NONE, vk::AccessFlags2::NONE)
        );
        assert!(
            prior_attachment_access(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .1
                .contains(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
        );
        assert_eq!(
            next_attachment_access(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL),
            vk::AccessFlags2::SHADER_SAMPLED_READ
        );
        assert!(
            next_attachment_access(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .contains(vk::AccessFlags2::COLOR_ATTACHMENT_READ)
        );
    }

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
