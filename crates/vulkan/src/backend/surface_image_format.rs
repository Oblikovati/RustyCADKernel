//! Native source-image encodings select actual sampled Vulkan formats and block geometry.
use crate::surface_texture::SurfaceImageEncoding;
use ash::vk;
pub(super) fn sampled_format(encoding: SurfaceImageEncoding) -> vk::Format {
    use SurfaceImageEncoding::*;
    use vk::Format as F;
    match encoding {
        Srgb8 => F::R8G8B8A8_SRGB,
        Linear8 => F::R8G8B8A8_UNORM,
        Linear16 => F::R16G16B16A16_UNORM,
        Float16 => F::R16G16B16A16_SFLOAT,
        Float32 => F::R32G32B32A32_SFLOAT,
        Bc1Rgb(true) => F::BC1_RGB_SRGB_BLOCK,
        Bc1Rgb(false) => F::BC1_RGB_UNORM_BLOCK,
        Bc1Rgba(true) => F::BC1_RGBA_SRGB_BLOCK,
        Bc1Rgba(false) => F::BC1_RGBA_UNORM_BLOCK,
        Bc2(true) => F::BC2_SRGB_BLOCK,
        Bc2(false) => F::BC2_UNORM_BLOCK,
        Bc3(true) => F::BC3_SRGB_BLOCK,
        Bc3(false) => F::BC3_UNORM_BLOCK,
    }
}
pub(super) fn sampled_levels(image: &crate::surface_texture::SurfaceTextureImage) -> u32 {
    if !image.supplied_levels.is_empty() {
        return image.supplied_levels.len() as u32;
    }
    if !image.mipmaps {
        return 1;
    }
    u32::BITS - image.extent[0].max(image.extent[1]).leading_zeros()
}
pub(super) fn image_range(first: u32, count: u32) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(first)
        .level_count(count)
        .layer_count(1)
}
