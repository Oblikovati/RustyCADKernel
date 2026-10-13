//! Existing driver limit queries report actual supported Vulkan raster capabilities.
use super::{oit_plan::OitCapabilities, raster_samples::select_attachment_samples};
use crate::RenderTransparency;
use rustycad_tkservice::graphic3d::Graphic3dTypeOfLimit;

impl super::VulkanRenderer {
    pub(crate) fn raster_limit(&self, limit: Graphic3dTypeOfLimit) -> i32 {
        let transparency = limit_query_transparency(limit);
        let caps = OitCapabilities::query(&self.runtime, transparency);
        raster_capability_limit(caps, limit)
    }
}
fn limit_query_transparency(limit: Graphic3dTypeOfLimit) -> RenderTransparency {
    if limit == Graphic3dTypeOfLimit::MaxMsaa {
        return RenderTransparency::Unordered;
    }
    RenderTransparency::Weighted { depth_factor: 0. }
}
fn raster_capability_limit(caps: OitCapabilities, limit: Graphic3dTypeOfLimit) -> i32 {
    let oit = caps.weighted_formats && caps.max_color_attachments >= 2;
    match limit {
        Graphic3dTypeOfLimit::MaxMsaa => {
            let samples = select_attachment_samples(i32::MAX, caps.attachment_samples).as_raw();
            if samples == 1 { 0 } else { samples as i32 }
        }
        Graphic3dTypeOfLimit::HasBlendedOit => i32::from(oit),
        Graphic3dTypeOfLimit::HasBlendedOitMsaa => i32::from(oit && caps.sample_rate_shading),
        _ => 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ash::vk;
    #[test]
    fn ordinary_msaa_limit_does_not_depend_on_float_oit_sample_counts() {
        assert_eq!(
            limit_query_transparency(Graphic3dTypeOfLimit::MaxMsaa),
            RenderTransparency::Unordered
        );
        assert_eq!(
            limit_query_transparency(Graphic3dTypeOfLimit::HasBlendedOitMsaa),
            RenderTransparency::Weighted { depth_factor: 0. }
        );
    }
    #[test]
    fn native_oit_limit_flags_distinguish_sample_variables_from_float_buffers() {
        let mut caps = OitCapabilities {
            weighted_formats: true,
            peeling_formats: true,
            max_color_attachments: 4,
            sample_rate_shading: false,
            attachment_samples: vk::SampleCountFlags::TYPE_1 | vk::SampleCountFlags::TYPE_4,
        };
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::MaxMsaa),
            4
        );
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::HasBlendedOit),
            1
        );
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::HasBlendedOitMsaa),
            0
        );
        caps.sample_rate_shading = true;
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::HasBlendedOitMsaa),
            1
        );
        caps.weighted_formats = false;
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::HasBlendedOit),
            0
        );
        assert_eq!(
            raster_capability_limit(caps, Graphic3dTypeOfLimit::HasMeshEdges),
            0
        );
    }
}
