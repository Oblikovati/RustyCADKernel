//! Sampler filtering matches native mip-aware minification and device anisotropy limits.
use super::{
    runtime::{VulkanRuntime, vk_error},
    sampled_surface_image::SampledSurfaceImage,
};
use crate::{RenderError, error::rejected, surface_texture::SurfaceSamplerPolicy};
use ash::vk;
use rustycad_tkservice::graphic3d::*;
pub(super) fn create_surface_sampler(
    image: &SampledSurfaceImage,
    policy: &SurfaceSamplerPolicy,
    count: u32,
) -> Result<vk::Sampler, RenderError> {
    let linear = policy.filter != Graphic3dTypeOfTextureFilter::Nearest;
    let filter = if linear {
        vk::Filter::LINEAR
    } else {
        vk::Filter::NEAREST
    };
    let mip = if policy.filter == Graphic3dTypeOfTextureFilter::Trilinear {
        vk::SamplerMipmapMode::LINEAR
    } else {
        vk::SamplerMipmapMode::NEAREST
    };
    let wrap = if policy.repeat {
        vk::SamplerAddressMode::REPEAT
    } else {
        vk::SamplerAddressMode::CLAMP_TO_EDGE
    };
    let anisotropy = surface_anisotropy(&image.runtime, policy.anisotropy);
    let info = vk::SamplerCreateInfo::default()
        .min_filter(filter)
        .mag_filter(filter)
        .mipmap_mode(mip)
        .address_mode_u(wrap)
        .address_mode_v(wrap)
        .address_mode_w(wrap)
        .min_lod(0.)
        .max_lod((count - 1) as f32)
        .anisotropy_enable(anisotropy > 1.)
        .max_anisotropy(anisotropy);
    unsafe { image.runtime.device().create_sampler(&info, None) }
        .map_err(|e| vk_error("create surface sampler", e))
}
fn surface_anisotropy(runtime: &VulkanRuntime, level: Graphic3dLevelOfTextureAnisotropy) -> f32 {
    if !runtime.sampler_anisotropy {
        return 1.;
    }
    let maximum = runtime.limits.max_sampler_anisotropy;
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
pub(super) fn surface_view_levels(
    levels: u32,
    policy: &SurfaceSamplerPolicy,
) -> Result<(u32, u32), RenderError> {
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
        policy.first_level = 2;
        policy.last_level = 3;
        assert_eq!(surface_view_levels(4, &policy).unwrap(), (2, 2));
        policy.first_level = 4;
        assert!(surface_view_levels(4, &policy).is_err());
        policy.first_level = -1;
        assert!(surface_view_levels(4, &policy).is_err());
    }
}
