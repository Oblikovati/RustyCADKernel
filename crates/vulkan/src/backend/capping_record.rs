//! Cap masks reset stencil inside the active render area and temporarily suppress layer depth.
use super::{frame::UploadedDraw, runtime::VulkanRuntime};
use crate::RenderLayerSpan;
use ash::vk;
pub(super) fn depth_policy(draw: &UploadedDraw, layer: &RenderLayerSpan) -> (bool, bool) {
    let mask = draw.capping.is_mask();
    (
        layer.depth_test && !draw.annotation && !mask,
        layer.depth_write && !mask,
    )
}
pub(super) fn set_depth_policy(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    policy: (bool, bool),
) {
    unsafe {
        runtime
            .device()
            .cmd_set_depth_test_enable(command, policy.0);
        runtime
            .device()
            .cmd_set_depth_write_enable(command, policy.1);
    }
}
pub(super) fn clear_stencil(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    extent: vk::Extent2D,
) {
    let clear = vk::ClearAttachment::default()
        .aspect_mask(vk::ImageAspectFlags::STENCIL)
        .clear_value(vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.,
                stencil: 0,
            },
        });
    let rect = vk::ClearRect::default()
        .rect(vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent,
        })
        .layer_count(1);
    unsafe {
        runtime
            .device()
            .cmd_clear_attachments(command, &[clear], &[rect]);
    }
}

pub(super) fn set_culling(
    runtime: &VulkanRuntime,
    command: vk::CommandBuffer,
    culling: (crate::RenderFaceCulling, bool),
) {
    let mode = match culling.0 {
        crate::RenderFaceCulling::None => vk::CullModeFlags::NONE,
        crate::RenderFaceCulling::Back => vk::CullModeFlags::BACK,
        crate::RenderFaceCulling::Front => vk::CullModeFlags::FRONT,
    };
    unsafe {
        runtime.device().cmd_set_cull_mode(command, mode);
        runtime.device().cmd_set_front_face(
            command,
            if culling.1 {
                vk::FrontFace::CLOCKWISE
            } else {
                vk::FrontFace::COUNTER_CLOCKWISE
            },
        );
    }
}
