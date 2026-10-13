//! Floating OIT attachments have their own lifetime and never alias the encoded scene target.
use super::image::{AttachmentSpec, TargetImage};
use super::oit_support::{
    DeviceOitFormats, peeling_attachment_formats, weighted_attachment_formats,
};
use super::runtime::VulkanRuntime;
use crate::{RenderError, RenderTransparency};
use ash::vk;
use std::sync::Arc;

pub(super) enum OitTargets {
    Weighted {
        accumulation: TargetImage,
        coverage: TargetImage,
    },
    Peeling {
        ping: Box<[TargetImage; 3]>,
        pong: Box<[TargetImage; 3]>,
        back: TargetImage,
    },
}
impl OitTargets {
    pub fn geometry_shape(&self) -> ([vk::Format; 3], vk::SampleCountFlags) {
        match self {
            Self::Weighted {
                accumulation,
                coverage,
            } => (
                [accumulation.format, coverage.format, vk::Format::UNDEFINED],
                accumulation.samples,
            ),
            Self::Peeling { ping, .. } => (
                ping.each_ref().map(|image| image.format),
                vk::SampleCountFlags::TYPE_1,
            ),
        }
    }
    pub fn create(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        transparency: RenderTransparency,
        samples: vk::SampleCountFlags,
    ) -> Result<Option<Self>, RenderError> {
        let probe = DeviceOitFormats(runtime);
        match transparency {
            RenderTransparency::Unordered => Ok(None),
            RenderTransparency::Weighted { .. } => weighted_attachment_formats(&probe)
                .map(|formats| Self::weighted(runtime, extent, formats, samples))
                .transpose(),
            RenderTransparency::DepthPeeling { .. } => peeling_attachment_formats(&probe)
                .map(|formats| Self::peeling(runtime, extent, formats))
                .transpose(),
        }
    }
    fn weighted(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        formats: [vk::Format; 2],
        samples: vk::SampleCountFlags,
    ) -> Result<Self, RenderError> {
        Ok(Self::Weighted {
            accumulation: TargetImage::with_attachment(
                runtime,
                extent,
                oit_attachment(formats[0], samples),
            )?,
            coverage: TargetImage::with_attachment(
                runtime,
                extent,
                oit_attachment(formats[1], samples),
            )?,
        })
    }
    fn peeling(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        formats: [vk::Format; 3],
    ) -> Result<Self, RenderError> {
        Ok(Self::Peeling {
            ping: Box::new(peeling_images(runtime, extent, formats)?),
            pong: Box::new(peeling_images(runtime, extent, formats)?),
            back: TargetImage::with_attachment(
                runtime,
                extent,
                oit_attachment(formats[1], vk::SampleCountFlags::TYPE_1),
            )?,
        })
    }
}
fn peeling_images(
    runtime: &Arc<VulkanRuntime>,
    extent: vk::Extent2D,
    formats: [vk::Format; 3],
) -> Result<[TargetImage; 3], RenderError> {
    let samples = vk::SampleCountFlags::TYPE_1;
    Ok([
        TargetImage::with_attachment(runtime, extent, oit_attachment(formats[0], samples))?,
        TargetImage::with_attachment(runtime, extent, oit_attachment(formats[1], samples))?,
        TargetImage::with_attachment(runtime, extent, oit_attachment(formats[2], samples))?,
    ])
}
fn oit_attachment(format: vk::Format, samples: vk::SampleCountFlags) -> AttachmentSpec {
    AttachmentSpec {
        format,
        samples,
        aspect: vk::ImageAspectFlags::COLOR,
        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn floating_targets_are_sampleable_without_encoded_scene_aliases() {
        let spec = oit_attachment(
            vk::Format::R16G16B16A16_SFLOAT,
            vk::SampleCountFlags::TYPE_4,
        );
        assert_eq!(spec.samples, vk::SampleCountFlags::TYPE_4);
        assert_eq!(spec.aspect, vk::ImageAspectFlags::COLOR);
        assert!(
            spec.usage
                .contains(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::COLOR_ATTACHMENT)
        );
        assert_eq!(spec.format, vk::Format::R16G16B16A16_SFLOAT);
    }
}
