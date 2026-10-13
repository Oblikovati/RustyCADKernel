//! Scene resolve targets keep single-sample capture buffers separate from geometry samples.
use super::{image::TargetImage, runtime::VulkanRuntime};
use crate::RenderError;
use ash::vk;
use std::sync::Arc;

pub(super) struct MultisampleTargets {
    pub color: TargetImage,
    pub depth: TargetImage,
}
impl MultisampleTargets {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        stencil: bool,
        samples: vk::SampleCountFlags,
    ) -> Result<Option<Self>, RenderError> {
        if samples == vk::SampleCountFlags::TYPE_1 {
            return Ok(None);
        }
        Ok(Some(Self {
            color: TargetImage::new_multisample(runtime, extent, false, false, samples)?,
            depth: TargetImage::new_multisample(runtime, extent, true, stencil, samples)?,
        }))
    }
}
pub(super) fn resolve_attachment<'a>(
    mut attachment: vk::RenderingAttachmentInfo<'a>,
    output: vk::ImageView,
    mode: vk::ResolveModeFlags,
    multisampled: bool,
) -> vk::RenderingAttachmentInfo<'a> {
    if !multisampled {
        return attachment;
    }
    attachment = attachment
        .resolve_mode(mode)
        .resolve_image_view(output)
        .resolve_image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL);
    attachment
}
#[cfg(test)]
mod tests {
    use super::*;
    use ash::vk::Handle;
    #[test]
    fn single_sample_does_not_resolve_and_multisample_keeps_explicit_attachment_alias() {
        let view = vk::ImageView::from_raw(1);
        let output = vk::ImageView::from_raw(2);
        let source = vk::RenderingAttachmentInfo::default().image_view(view);
        assert_eq!(
            resolve_attachment(source, output, vk::ResolveModeFlags::AVERAGE, false).resolve_mode,
            vk::ResolveModeFlags::NONE
        );
        let resolved = resolve_attachment(source, output, vk::ResolveModeFlags::SAMPLE_ZERO, true);
        assert_eq!(resolved.image_view, view);
        assert_eq!(resolved.resolve_image_view, output);
        assert_eq!(resolved.resolve_mode, vk::ResolveModeFlags::SAMPLE_ZERO);
    }
}
