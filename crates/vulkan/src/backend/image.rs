//! Color and depth images own their views and memory in Vulkan destruction order.
use super::runtime::{VulkanRuntime, vk_error};
use crate::error::{RenderError, rejected};
use ash::vk;
use std::sync::Arc;
pub(super) struct TargetImage {
    runtime: Arc<VulkanRuntime>,
    pub handle: vk::Image,
    pub view: vk::ImageView,
    pub xor_view: vk::ImageView,
    memory: vk::DeviceMemory,
    pub aspect: vk::ImageAspectFlags,
    pub format: vk::Format,
    pub samples: vk::SampleCountFlags,
}
#[derive(Clone, Copy)]
pub(super) struct AttachmentSpec {
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub aspect: vk::ImageAspectFlags,
    pub samples: vk::SampleCountFlags,
}
impl TargetImage {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        depth: bool,
        stencil: bool,
    ) -> Result<Self, RenderError> {
        Self::new_multisample(
            runtime,
            extent,
            depth,
            stencil,
            vk::SampleCountFlags::TYPE_1,
        )
    }
    pub fn new_multisample(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        depth: bool,
        stencil: bool,
        samples: vk::SampleCountFlags,
    ) -> Result<Self, RenderError> {
        let (format, usage, aspect) = attachment_properties(depth, stencil);
        Self::with_attachment(
            runtime,
            extent,
            AttachmentSpec {
                format,
                usage,
                aspect,
                samples,
            },
        )
    }
    pub fn with_attachment(
        runtime: &Arc<VulkanRuntime>,
        extent: vk::Extent2D,
        spec: AttachmentSpec,
    ) -> Result<Self, RenderError> {
        check_format(
            runtime,
            spec.format,
            spec.aspect.contains(vk::ImageAspectFlags::DEPTH),
        )?;
        let info = image_description(extent, spec.format, spec.usage).samples(spec.samples);
        let handle = unsafe { runtime.device().create_image(&info, None) }
            .map_err(|error| vk_error("create target image", error))?;
        let mut image = Self {
            runtime: runtime.clone(),
            handle,
            view: vk::ImageView::null(),
            xor_view: vk::ImageView::null(),
            memory: vk::DeviceMemory::null(),
            aspect: spec.aspect,
            format: spec.format,
            samples: spec.samples,
        };
        image.allocate()?;
        image.view = image.create_view(spec.format)?;
        image.create_encoded_alias(spec.format)?;
        Ok(image)
    }
    fn create_encoded_alias(&mut self, format: vk::Format) -> Result<(), RenderError> {
        if format != vk::Format::R8G8B8A8_SRGB {
            return Ok(());
        }
        check_format(&self.runtime, vk::Format::R8G8B8A8_UNORM, false)?;
        self.xor_view = self.create_view(vk::Format::R8G8B8A8_UNORM)?;
        Ok(())
    }
    fn allocate(&mut self) -> Result<(), RenderError> {
        let requirements = unsafe {
            self.runtime
                .device()
                .get_image_memory_requirements(self.handle)
        };
        let index = self.runtime.memory_type(
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;
        let info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(index);
        self.memory = unsafe { self.runtime.device().allocate_memory(&info, None) }
            .map_err(|e| vk_error("allocate target image", e))?;
        unsafe {
            self.runtime
                .device()
                .bind_image_memory(self.handle, self.memory, 0)
        }
        .map_err(|e| vk_error("bind image memory", e))
    }
    fn create_view(&self, format: vk::Format) -> Result<vk::ImageView, RenderError> {
        let info = vk::ImageViewCreateInfo::default()
            .image(self.handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(self.range());
        unsafe { self.runtime.device().create_image_view(&info, None) }
            .map_err(|e| vk_error("create target view", e))
    }
    pub fn range(&self) -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange::default()
            .aspect_mask(self.aspect)
            .level_count(1)
            .layer_count(1)
    }
}
impl Drop for TargetImage {
    fn drop(&mut self) {
        unsafe {
            if self.view != vk::ImageView::null() {
                self.runtime.device().destroy_image_view(self.view, None);
            }
            if self.xor_view != vk::ImageView::null() {
                self.runtime
                    .device()
                    .destroy_image_view(self.xor_view, None);
            }
            self.runtime.device().destroy_image(self.handle, None);
            if self.memory != vk::DeviceMemory::null() {
                self.runtime.device().free_memory(self.memory, None);
            }
        }
    }
}
pub(super) fn check_format(
    runtime: &VulkanRuntime,
    format: vk::Format,
    depth: bool,
) -> Result<(), RenderError> {
    let features = unsafe {
        runtime
            .instance
            .get_physical_device_format_properties(runtime.physical, format)
    }
    .optimal_tiling_features;
    let required = vk::FormatFeatureFlags::TRANSFER_SRC
        | if depth {
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
        } else {
            vk::FormatFeatureFlags::COLOR_ATTACHMENT
        };
    if !features.contains(required) {
        return Err(rejected(
            (format, features),
            "attachment and transfer-source format support",
        ));
    }
    Ok(())
}

fn attachment_properties(
    depth: bool,
    stencil: bool,
) -> (vk::Format, vk::ImageUsageFlags, vk::ImageAspectFlags) {
    if depth {
        return (
            if stencil {
                vk::Format::D32_SFLOAT_S8_UINT
            } else {
                vk::Format::D32_SFLOAT
            },
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            if stencil {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            },
        );
    }
    // OCCT 3d097a0 OpenGl_View uses GL_SRGB8_ALPHA8: inputs stay linear, captures encode sRGB.
    (
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageUsageFlags::COLOR_ATTACHMENT,
        vk::ImageAspectFlags::COLOR,
    )
}
fn image_description(
    extent: vk::Extent2D,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
) -> vk::ImageCreateInfo<'static> {
    vk::ImageCreateInfo::default()
        // Vulkan logic ops ignore sRGB targets; compatible UNORM views preserve encoded bytes.
        .flags(if format == vk::Format::R8G8B8A8_SRGB {
            vk::ImageCreateFlags::MUTABLE_FORMAT
        } else {
            vk::ImageCreateFlags::empty()
        })
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage | vk::ImageUsageFlags::TRANSFER_SRC)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_capped_depth_targets_allocate_a_combined_depth_stencil_attachment() {
        let plain = attachment_properties(true, false);
        let cap = attachment_properties(true, true);
        assert_eq!(plain.0, vk::Format::D32_SFLOAT);
        assert_eq!(plain.2, vk::ImageAspectFlags::DEPTH);
        assert_eq!(cap.0, vk::Format::D32_SFLOAT_S8_UINT);
        assert_eq!(
            cap.2,
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        );
        assert_eq!(
            attachment_properties(false, true),
            attachment_properties(false, false)
        );
        let info = image_description(
            vk::Extent2D {
                width: 64,
                height: 48,
            },
            cap.0,
            cap.1,
        );
        assert!(info.usage.contains(vk::ImageUsageFlags::TRANSFER_SRC));
        assert_eq!(info.extent.height, 48);
    }
}

#[cfg(test)]
mod xor_tests {
    use super::*;
    #[test]
    fn only_color_attachments_allow_compatible_encoded_byte_views() {
        let extent = vk::Extent2D {
            width: 8,
            height: 8,
        };
        let color = image_description(
            extent,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageUsageFlags::COLOR_ATTACHMENT,
        );
        let depth = image_description(
            extent,
            vk::Format::D32_SFLOAT,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        );
        assert_eq!(color.flags, vk::ImageCreateFlags::MUTABLE_FORMAT);
        assert!(depth.flags.is_empty());
        assert_eq!(color.samples, vk::SampleCountFlags::TYPE_1);
        assert_eq!(color.array_layers, 1);
    }
}
