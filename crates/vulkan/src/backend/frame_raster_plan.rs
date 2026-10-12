//! Resolve capability and stencil sample limits participate before allocating any frame target.
use super::{
    oit_plan::{OitCapabilities, OitResourcePlan, choose_resource_plan},
    runtime::VulkanRuntime,
};
use crate::{RenderFrame, RenderTransparency};
use ash::vk;

pub(super) fn frame_raster_plan(
    runtime: &VulkanRuntime,
    frame: &RenderFrame,
    stencil: bool,
) -> OitResourcePlan {
    if frame.raster.requested_samples <= 0 && frame.transparency() == RenderTransparency::Unordered
    {
        return OitResourcePlan {
            transparency: RenderTransparency::Unordered,
            samples: vk::SampleCountFlags::TYPE_1,
            disable_feedback: 0,
        };
    }
    let mut caps = OitCapabilities::query(runtime, frame.transparency());
    if stencil {
        caps.attachment_samples &= runtime.limits.framebuffer_stencil_sample_counts;
    }
    let mut plan = choose_resource_plan(
        frame.raster,
        caps,
        frame
            .draws
            .iter()
            .any(crate::PrimitiveDraw::is_surface_blended),
    );
    if plan.samples != vk::SampleCountFlags::TYPE_1
        && !supports_scene_resolve(
            runtime.depth_resolve_modes,
            runtime.stencil_resolve_modes,
            stencil,
        )
    {
        plan.samples = vk::SampleCountFlags::TYPE_1;
        plan.disable_feedback |= 1;
    }
    if plan.disable_feedback & 1 != 0 {
        frame.raster_feedback.disable_multisampling();
    }
    if plan.disable_feedback & 2 != 0 {
        frame.raster_feedback.disable_oit();
    }
    plan
}
fn supports_scene_resolve(
    depth: vk::ResolveModeFlags,
    stencil: vk::ResolveModeFlags,
    capped: bool,
) -> bool {
    depth.contains(vk::ResolveModeFlags::SAMPLE_ZERO)
        && (!capped || stencil.contains(vk::ResolveModeFlags::SAMPLE_ZERO))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cap_stencil_needs_its_actual_resolve_mode_without_affecting_uncapped_depth() {
        let zero = vk::ResolveModeFlags::SAMPLE_ZERO;
        assert!(supports_scene_resolve(
            zero,
            vk::ResolveModeFlags::NONE,
            false
        ));
        assert!(!supports_scene_resolve(
            zero,
            vk::ResolveModeFlags::NONE,
            true
        ));
        assert!(supports_scene_resolve(zero, zero, true));
        assert!(!supports_scene_resolve(
            vk::ResolveModeFlags::NONE,
            zero,
            false
        ));
    }
}
