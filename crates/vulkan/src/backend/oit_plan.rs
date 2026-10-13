//! Native view capability fallback is distinct from allocation failure in one frame.
use super::raster_samples::select_attachment_samples;
use crate::transparency::{RenderRasterSettings, RenderTransparency};
use ash::vk;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OitCapabilities {
    pub weighted_formats: bool,
    pub peeling_formats: bool,
    pub max_color_attachments: u32,
    pub sample_rate_shading: bool,
    pub attachment_samples: vk::SampleCountFlags,
}
impl OitCapabilities {
    pub fn query(
        runtime: &super::runtime::VulkanRuntime,
        transparency: RenderTransparency,
    ) -> Self {
        let probe = super::oit_support::DeviceOitFormats(runtime);
        let weighted = super::oit_support::weighted_attachment_formats(&probe);
        let peeling = super::oit_support::peeling_attachment_formats(&probe);
        let samples = compatible_attachment_samples(runtime, transparency, weighted);
        Self {
            weighted_formats: weighted.is_some(),
            peeling_formats: peeling.is_some(),
            max_color_attachments: runtime.limits.max_color_attachments,
            sample_rate_shading: runtime.sample_rate_shading,
            attachment_samples: samples,
        }
    }
}
fn compatible_attachment_samples(
    runtime: &super::runtime::VulkanRuntime,
    transparency: RenderTransparency,
    weighted: Option<[vk::Format; 2]>,
) -> vk::SampleCountFlags {
    let mut samples = runtime.limits.framebuffer_color_sample_counts
        & runtime.limits.framebuffer_depth_sample_counts;
    if matches!(transparency, RenderTransparency::Weighted { .. })
        && let Some(formats) = weighted
    {
        for format in formats {
            samples &= super::oit_support::floating_sample_counts(runtime, format);
        }
    }
    samples
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct OitResourcePlan {
    pub transparency: RenderTransparency,
    pub samples: vk::SampleCountFlags,
    pub disable_feedback: u8,
}
pub(super) fn choose_resource_plan(
    raster: RenderRasterSettings,
    caps: OitCapabilities,
    has_transparent_fill: bool,
) -> OitResourcePlan {
    let mut plan = initial_resource_plan(raster, caps);
    if plan.transparency == RenderTransparency::Unordered {
        return plan;
    }
    if !check_weighted_compatibility(&mut plan, caps) {
        return plan;
    }
    apply_oit_sample_fallback(&mut plan, has_transparent_fill);
    if matches!(plan.transparency, RenderTransparency::DepthPeeling { .. })
        && (!caps.peeling_formats || caps.max_color_attachments < 3)
    {
        plan.transparency = RenderTransparency::Unordered;
    }
    plan
}
fn initial_resource_plan(raster: RenderRasterSettings, caps: OitCapabilities) -> OitResourcePlan {
    let samples = requested_samples(raster, caps);
    let transparency =
        if raster.disabled_capabilities & 2 != 0 && samples == vk::SampleCountFlags::TYPE_1 {
            RenderTransparency::Unordered
        } else {
            raster.transparency
        };
    OitResourcePlan {
        transparency,
        samples,
        disable_feedback: 0,
    }
}
fn check_weighted_compatibility(plan: &mut OitResourcePlan, caps: OitCapabilities) -> bool {
    let incompatible = !caps.weighted_formats
        || caps.max_color_attachments < 2
        || (plan.samples != vk::SampleCountFlags::TYPE_1 && !caps.sample_rate_shading);
    if !incompatible {
        return true;
    }
    plan.transparency = RenderTransparency::Unordered;
    // Native checkOitCompatibility writes DisableMSAA for this already-selected MSAA frame.
    plan.disable_feedback |= if plan.samples == vk::SampleCountFlags::TYPE_1 {
        2
    } else {
        1
    };
    false
}

fn requested_samples(raster: RenderRasterSettings, caps: OitCapabilities) -> vk::SampleCountFlags {
    if raster.disabled_capabilities & 1 != 0 {
        return vk::SampleCountFlags::TYPE_1;
    }
    select_attachment_samples(raster.requested_samples, caps.attachment_samples)
}
fn apply_oit_sample_fallback(plan: &mut OitResourcePlan, has_transparent_fill: bool) {
    if plan.samples == vk::SampleCountFlags::TYPE_1 {
        return;
    }
    // OCCT 3d097a0 initializes peeling buffers single-sample and disables MSAA after failed blit.
    if has_transparent_fill && matches!(plan.transparency, RenderTransparency::DepthPeeling { .. })
    {
        plan.samples = vk::SampleCountFlags::TYPE_1;
        plan.disable_feedback |= 1;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn masked_only_peeling_keeps_native_msaa_without_triggering_a_peel_blit() {
        let mut raster = weighted_request();
        raster.transparency = RenderTransparency::DepthPeeling { layers: 1 };
        let opaque = choose_resource_plan(raster, named_full_device(), false);
        assert_eq!(opaque.samples, vk::SampleCountFlags::TYPE_4);
        assert_eq!(opaque.disable_feedback, 0);
        let transparent = choose_resource_plan(raster, named_full_device(), true);
        assert_eq!(transparent.samples, vk::SampleCountFlags::TYPE_1);
        assert_eq!(transparent.disable_feedback, 1);
    }
    fn named_full_device() -> OitCapabilities {
        OitCapabilities {
            weighted_formats: true,
            peeling_formats: true,
            max_color_attachments: 4,
            sample_rate_shading: true,
            attachment_samples: vk::SampleCountFlags::TYPE_1 | vk::SampleCountFlags::TYPE_4,
        }
    }
    fn weighted_request() -> RenderRasterSettings {
        RenderRasterSettings {
            transparency: RenderTransparency::Weighted { depth_factor: 0.5 },
            requested_samples: 3,
            ..Default::default()
        }
    }
    #[test]
    fn sample_failure_disables_only_view_msaa_and_preserves_requested_algorithm() {
        let mut raster = weighted_request();
        let mut caps = named_full_device();
        caps.sample_rate_shading = false;
        let weighted = choose_resource_plan(raster, caps, true);
        assert_eq!(weighted.transparency, RenderTransparency::Unordered);
        assert_eq!(weighted.samples, vk::SampleCountFlags::TYPE_4);
        assert_eq!(weighted.disable_feedback, 1);
        raster.disabled_capabilities = 1;
        assert_eq!(
            choose_resource_plan(raster, caps, true).transparency,
            raster.transparency
        );
        raster.disabled_capabilities = 0;
        raster.transparency = RenderTransparency::DepthPeeling { layers: 4 };
        assert_eq!(
            choose_resource_plan(raster, named_full_device(), true).disable_feedback,
            1
        );
    }
    #[test]
    fn format_failure_distinguishes_global_oit_and_peeling_only_failure() {
        let mut caps = named_full_device();
        caps.weighted_formats = false;
        assert_eq!(
            choose_resource_plan(weighted_request(), caps, true).disable_feedback,
            1
        );
        let mut raster = weighted_request();
        raster.requested_samples = 0;
        raster.transparency = RenderTransparency::DepthPeeling { layers: 4 };
        caps = named_full_device();
        caps.peeling_formats = false;
        let plan = choose_resource_plan(raster, caps, true);
        assert_eq!(plan.transparency, RenderTransparency::Unordered);
        assert_eq!(plan.disable_feedback, 0);
    }
    #[test]
    fn captured_view_failure_keeps_later_weighted_request_single_sample() {
        let mut raster = weighted_request();
        raster.disabled_capabilities = 1;
        assert_eq!(
            choose_resource_plan(raster, named_full_device(), true).samples,
            vk::SampleCountFlags::TYPE_1
        );
        raster.disabled_capabilities = 2;
        let plan = choose_resource_plan(raster, named_full_device(), true);
        assert_eq!(plan.transparency, raster.transparency);
        assert_eq!(plan.samples, vk::SampleCountFlags::TYPE_4);
        assert_eq!(plan.disable_feedback, 0);
        raster.requested_samples = 0;
        assert_eq!(
            choose_resource_plan(raster, named_full_device(), true).transparency,
            RenderTransparency::Unordered
        );
    }
    #[test]
    fn missing_float_feedback_matches_native_restricted_then_restored_view() {
        let mut raster = weighted_request();
        let mut caps = named_full_device();
        caps.weighted_formats = false;
        let first = choose_resource_plan(raster, caps, true);
        assert_eq!(first.samples, vk::SampleCountFlags::TYPE_4);
        assert_eq!(first.disable_feedback, 1);
        raster.disabled_capabilities |= first.disable_feedback;
        let second = choose_resource_plan(raster, caps, true);
        assert_eq!(second.samples, vk::SampleCountFlags::TYPE_1);
        assert_eq!(second.disable_feedback, 2);
        raster.disabled_capabilities |= second.disable_feedback;
        assert_eq!(
            choose_resource_plan(raster, named_full_device(), true).transparency,
            RenderTransparency::Unordered
        );
    }
}
