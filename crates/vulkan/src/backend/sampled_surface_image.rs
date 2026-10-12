//! Device-local sampled images retain all mip levels until no frame/cache references them.
use super::{
    runtime::{VulkanRuntime, vk_error},
    surface_image_format::*,
};
use crate::{RenderError, error::rejected, surface_texture::SurfaceTextureImage};
use ash::vk;
use std::sync::Arc;
pub(super) struct SampledSurfaceImage {
    pub runtime: Arc<VulkanRuntime>,
    pub handle: vk::Image,
    pub format: vk::Format,
    pub levels: u32,
    pub extent: vk::Extent3D,
    memory: vk::DeviceMemory,
}
impl SampledSurfaceImage {
    pub fn new(
        runtime: &Arc<VulkanRuntime>,
        source: &SurfaceTextureImage,
        format: vk::Format,
    ) -> Result<Arc<Self>, RenderError> {
        let levels = sampled_levels(source);
        validate_sampled_image(runtime, source, format, levels)?;
        let extent = sampled_surface_extent(source);
        let handle = create_sampled_image(runtime, extent, format, levels)?;
        let mut image = Self {
            runtime: runtime.clone(),
            handle,
            format,
            levels,
            extent,
            memory: vk::DeviceMemory::null(),
        };
        image.allocate()?;
        super::sampled_surface_upload::upload_sampled_image(&image, source)?;
        Ok(Arc::new(image))
    }
    fn allocate(&mut self) -> Result<(), RenderError> {
        self.memory = allocate_sampled_memory(&self.runtime, self.handle)?;
        unsafe {
            self.runtime
                .device()
                .bind_image_memory(self.handle, self.memory, 0)
        }
        .map_err(|e| vk_error("bind sampled image", e))
    }
    pub fn view(&self, first: u32, count: u32) -> Result<vk::ImageView, RenderError> {
        let info = vk::ImageViewCreateInfo::default()
            .image(self.handle)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(self.format)
            .subresource_range(image_range(first, count));
        unsafe { self.runtime.device().create_image_view(&info, None) }
            .map_err(|e| vk_error("create sampled image view", e))
    }
}
impl Drop for SampledSurfaceImage {
    fn drop(&mut self) {
        unsafe {
            self.runtime.device().destroy_image(self.handle, None);
            if self.memory != vk::DeviceMemory::null() {
                self.runtime.device().free_memory(self.memory, None);
            }
        }
    }
}
fn create_sampled_image(
    runtime: &VulkanRuntime,
    extent: vk::Extent3D,
    format: vk::Format,
    levels: u32,
) -> Result<vk::Image, RenderError> {
    let info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(extent)
        .mip_levels(levels)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::TRANSFER_SRC,
        );
    unsafe { runtime.device().create_image(&info, None) }
        .map_err(|e| vk_error("create sampled image", e))
}
fn validate_sampled_image(
    runtime: &VulkanRuntime,
    source: &SurfaceTextureImage,
    format: vk::Format,
    levels: u32,
) -> Result<(), RenderError> {
    validate_sampled_extent(source.extent, runtime.limits.max_image_dimension2_d)?;
    let support = unsafe {
        runtime
            .instance
            .get_physical_device_format_properties(runtime.physical, format)
    }
    .optimal_tiling_features;
    let required = required_sampled_features(source.supplied_levels.is_empty() && levels > 1);
    if !support.contains(required) {
        return Err(rejected(
            (format, support, required),
            "device support for sampled image and mip generation",
        ));
    }
    Ok(())
}

fn sampled_surface_extent(source: &SurfaceTextureImage) -> vk::Extent3D {
    vk::Extent3D {
        width: source.extent[0],
        height: source.extent[1],
        depth: 1,
    }
}
fn allocate_sampled_memory(
    runtime: &VulkanRuntime,
    image: vk::Image,
) -> Result<vk::DeviceMemory, RenderError> {
    let needs = unsafe { runtime.device().get_image_memory_requirements(image) };
    let kind = runtime.memory_type(
        needs.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let info = vk::MemoryAllocateInfo::default()
        .allocation_size(needs.size)
        .memory_type_index(kind);
    unsafe { runtime.device().allocate_memory(&info, None) }
        .map_err(|e| vk_error("allocate sampled image", e))
}
fn validate_sampled_extent(extent: [u32; 2], maximum: u32) -> Result<(), RenderError> {
    if extent.contains(&0) || extent.iter().any(|value| *value > maximum) {
        return Err(rejected(
            (extent, maximum),
            "positive sampled dimensions within maxImageDimension2D",
        ));
    }
    Ok(())
}
fn required_sampled_features(generate_mips: bool) -> vk::FormatFeatureFlags {
    let mut features = vk::FormatFeatureFlags::SAMPLED_IMAGE
        | vk::FormatFeatureFlags::TRANSFER_DST
        | vk::FormatFeatureFlags::TRANSFER_SRC;
    if generate_mips {
        features |= vk::FormatFeatureFlags::BLIT_SRC
            | vk::FormatFeatureFlags::BLIT_DST
            | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR;
    }
    features
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sampled_extent_and_mip_feature_requirements_are_explicit_before_allocating() {
        assert!(validate_sampled_extent([0, 4], 1024).is_err());
        assert!(validate_sampled_extent([4, 2048], 1024).is_err());
        assert!(validate_sampled_extent([4, 4], 1024).is_ok());
        assert!(required_sampled_features(true).contains(
            vk::FormatFeatureFlags::BLIT_SRC | vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
        ));
        assert!(!required_sampled_features(false).contains(vk::FormatFeatureFlags::BLIT_SRC));
    }
}
