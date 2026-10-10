//! Native cap masks suppress color/depth writes and invert the stencil parity bit.
use crate::RenderCappingPass;
use ash::vk;
pub(super) const MASK_BLEND: [vk::PipelineColorBlendAttachmentState; 1] =
    [vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::FALSE,
        src_color_blend_factor: vk::BlendFactor::ZERO,
        dst_color_blend_factor: vk::BlendFactor::ZERO,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ZERO,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::empty(),
    }];
pub(super) fn depth_state(
    pass: RenderCappingPass,
) -> vk::PipelineDepthStencilStateCreateInfo<'static> {
    let stencil = stencil_state(pass.is_mask());
    vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(if pass == RenderCappingPass::Surface {
            vk::CompareOp::LESS
        } else {
            vk::CompareOp::LESS_OR_EQUAL
        })
        .stencil_test_enable(pass != RenderCappingPass::None)
        .front(stencil)
        .back(stencil)
}
fn stencil_state(mask: bool) -> vk::StencilOpState {
    vk::StencilOpState::default()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(if mask {
            vk::StencilOp::INVERT
        } else {
            vk::StencilOp::KEEP
        })
        .depth_fail_op(if mask {
            vk::StencilOp::INVERT
        } else {
            vk::StencilOp::KEEP
        })
        .compare_op(if mask {
            vk::CompareOp::ALWAYS
        } else {
            vk::CompareOp::EQUAL
        })
        .compare_mask(1)
        .write_mask(if mask { 0xff } else { 0 })
        .reference(1)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_and_surface_follow_native_stencil_and_depth_policy() {
        let mask = depth_state(RenderCappingPass::MaskStart);
        assert_eq!(mask.front.pass_op, vk::StencilOp::INVERT);
        assert_eq!(mask.back.depth_fail_op, vk::StencilOp::INVERT);
        assert!(MASK_BLEND[0].color_write_mask.is_empty());
        let cap = depth_state(RenderCappingPass::Surface);
        assert_eq!(cap.depth_compare_op, vk::CompareOp::LESS);
        assert_eq!(cap.front.compare_op, vk::CompareOp::EQUAL);
        assert_eq!(cap.front.compare_mask, 1);
        assert_eq!(cap.front.write_mask, 0);
        assert_eq!(
            depth_state(RenderCappingPass::None).stencil_test_enable,
            vk::FALSE
        );
    }
}
