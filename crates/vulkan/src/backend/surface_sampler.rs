//! Sampler filtering matches native mip-aware minification and device anisotropy limits.
use super::{
    runtime::{VulkanRuntime, vk_error},
    sampled_surface_image::SampledSurfaceImage,
};
use crate::{RenderError, error::rejected, surface_texture::SurfaceSamplerPolicy};
use ash::vk;
use rustycad_tkservice::graphic3d::*;
/// Create actual VkSampler state; `create_surface_sampler(image, policy, levels)?`.
pub(super) fn create_surface_sampler(
    image: &SampledSurfaceImage,
    policy: &SurfaceSamplerPolicy,
    count: u32,
) -> Result<vk::Sampler, RenderError> {
    let info = surface_sampler_info(&image.runtime, policy, count);
    unsafe { image.runtime.device().create_sampler(&info, None) }
        .map_err(|e| vk_error("create surface sampler", e))
}
fn surface_sampler_info(
    runtime: &VulkanRuntime,
    policy: &SurfaceSamplerPolicy,
    count: u32,
) -> vk::SamplerCreateInfo<'static> {
    let (filter, mip) = surface_filter_modes(policy.filter);
    let wrap = surface_sampler_wrap(policy.repeat);
    let anisotropy = runtime_surface_anisotropy(runtime, policy.anisotropy);
    vk::SamplerCreateInfo::default()
        .min_filter(filter)
        .mag_filter(filter)
        .mipmap_mode(mip)
        .address_mode_u(wrap)
        .address_mode_v(wrap)
        .address_mode_w(wrap)
        .min_lod(0.)
        .max_lod((count - 1) as f32)
        .anisotropy_enable(anisotropy > 1.)
        .max_anisotropy(anisotropy)
}
fn surface_sampler_wrap(repeat: bool) -> vk::SamplerAddressMode {
    if repeat {
        vk::SamplerAddressMode::REPEAT
    } else {
        vk::SamplerAddressMode::CLAMP_TO_EDGE
    }
}
fn runtime_surface_anisotropy(
    runtime: &VulkanRuntime,
    level: Graphic3dLevelOfTextureAnisotropy,
) -> f32 {
    if !runtime.sampler_anisotropy {
        return 1.;
    }
    surface_anisotropy_limit(runtime.limits.max_sampler_anisotropy, level)
}
fn surface_filter_modes(
    filter: Graphic3dTypeOfTextureFilter,
) -> (vk::Filter, vk::SamplerMipmapMode) {
    let texel = if filter == Graphic3dTypeOfTextureFilter::Nearest {
        vk::Filter::NEAREST
    } else {
        vk::Filter::LINEAR
    };
    let mip = if filter == Graphic3dTypeOfTextureFilter::Trilinear {
        vk::SamplerMipmapMode::LINEAR
    } else {
        vk::SamplerMipmapMode::NEAREST
    };
    (texel, mip)
}
fn surface_anisotropy_limit(maximum: f32, level: Graphic3dLevelOfTextureAnisotropy) -> f32 {
    match level {
        Graphic3dLevelOfTextureAnisotropy::Off => 1.,
        Graphic3dLevelOfTextureAnisotropy::Fast => 2_f32.min(maximum),
        Graphic3dLevelOfTextureAnisotropy::Middle => {
            if maximum <= 4. {
                2_f32.min(maximum)
            } else {
                (maximum / 2.).floor()
            }
        }
        _ => maximum,
    }
}
/// Validate view-relative native mip range; `surface_view_levels(4, policy)?`.
pub(super) fn surface_view_levels(
    levels: u32,
    policy: &SurfaceSamplerPolicy,
) -> Result<(u32, u32), RenderError> {
    if levels == 0 {
        return Err(rejected(levels, "positive uploaded mip count"));
    }
    let first = u32::try_from(policy.first_level)
        .map_err(|_| rejected(policy.first_level, "nonnegative texture base level"))?;
    let last = u32::try_from(policy.last_level)
        .map_err(|_| rejected(policy.last_level, "nonnegative texture maximum level"))?
        .min(levels - 1);
    if first > last {
        return Err(rejected(
            (first, last, levels),
            "texture level range within uploaded mip levels",
        ));
    }
    Ok((first, last - first + 1))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn level_range_truncates_default_maximum_and_rejects_invalid_ranges() {
        let parameters = Graphic3dTextureParams::default();
        let mut policy = SurfaceSamplerPolicy::read(&parameters);
        assert_eq!(surface_view_levels(4, &policy).unwrap(), (0, 4));
        assert!(surface_view_levels(0, &policy).is_err());
        policy.first_level = 2;
        policy.last_level = 3;
        assert_eq!(surface_view_levels(4, &policy).unwrap(), (2, 2));
        policy.first_level = 4;
        assert!(surface_view_levels(4, &policy).is_err());
        policy.first_level = -1;
        assert!(surface_view_levels(4, &policy).is_err());
    }
    #[test]
    fn native_filter_pairs_are_distinct() {
        assert_eq!(
            surface_filter_modes(Graphic3dTypeOfTextureFilter::Nearest),
            (vk::Filter::NEAREST, vk::SamplerMipmapMode::NEAREST)
        );
        assert_eq!(
            surface_filter_modes(Graphic3dTypeOfTextureFilter::Bilinear),
            (vk::Filter::LINEAR, vk::SamplerMipmapMode::NEAREST)
        );
        assert_eq!(
            surface_filter_modes(Graphic3dTypeOfTextureFilter::Trilinear),
            (vk::Filter::LINEAR, vk::SamplerMipmapMode::LINEAR)
        );
    }
    #[test]
    fn native_anisotropy_limits_and_wrap_modes_follow_parameters() {
        assert_eq!(surface_sampler_wrap(true), vk::SamplerAddressMode::REPEAT);
        assert_eq!(
            surface_sampler_wrap(false),
            vk::SamplerAddressMode::CLAMP_TO_EDGE
        );
        assert_eq!(
            surface_anisotropy_limit(16., Graphic3dLevelOfTextureAnisotropy::Middle),
            8.
        );
        assert_eq!(
            surface_anisotropy_limit(4., Graphic3dLevelOfTextureAnisotropy::Middle),
            2.
        );
        assert_eq!(
            surface_anisotropy_limit(1., Graphic3dLevelOfTextureAnisotropy::Fast),
            1.
        );
    }
}
