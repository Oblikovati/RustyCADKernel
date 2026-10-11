//! Cap masks reset stencil inside the active render area and temporarily suppress layer depth.
use super::{frame::UploadedDraw, runtime::VulkanRuntime};
use crate::RenderLayerSpan;
use ash::vk;
pub(super) fn depth_policy(draw: &UploadedDraw, layer: &RenderLayerSpan) -> (bool, bool) {
    draw_depth_flags(
        layer.depth_policy(),
        draw.annotation,
        draw.capping.is_mask(),
        draw.surface_blend,
    )
}
fn draw_depth_flags(
    layer: (bool, bool, bool),
    annotation: bool,
    mask: bool,
    blend: bool,
) -> (bool, bool) {
    (layer.0 && !annotation && !mask, layer.1 && !mask && !blend)
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

#[cfg(test)]
mod tests {
    use super::draw_depth_flags;
    #[test]
    fn transparent_surfaces_test_depth_without_writing_it() {
        assert_eq!(
            draw_depth_flags((true, true, false), false, false, true),
            (true, false)
        );
        assert_eq!(
            draw_depth_flags((false, true, false), false, false, true),
            (false, false)
        );
        assert_eq!(
            draw_depth_flags((true, true, false), false, false, false),
            (true, true)
        );
        assert_eq!(
            draw_depth_flags((true, true, false), true, false, false),
            (false, true)
        );
        assert_eq!(
            draw_depth_flags((true, true, false), false, true, false),
            (false, false)
        );
    }
}
