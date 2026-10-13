//! OCCT 3d097a0 uses identical blend equations on every geometry MRT attachment.
use ash::vk;

pub(super) fn weighted_blend() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ZERO)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
}
pub(super) fn peeling_blend() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE)
        .dst_color_blend_factor(vk::BlendFactor::ONE)
        .color_blend_op(vk::BlendOp::MAX)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::MAX)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
}
pub(super) fn peeling_back_blend() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
}
pub(super) fn weighted_composite_blend() -> vk::PipelineColorBlendAttachmentState {
    vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ZERO)
        .dst_alpha_blend_factor(vk::BlendFactor::ONE)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::from_raw(0x7))
}
pub(super) fn peeling_composite_blend() -> vk::PipelineColorBlendAttachmentState {
    let mut blend = weighted_composite_blend();
    blend.src_color_blend_factor = vk::BlendFactor::ONE;
    blend.dst_color_blend_factor = vk::BlendFactor::ONE_MINUS_SRC_ALPHA;
    blend
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn geometry_preserves_revealage_or_componentwise_maximum() {
        let weighted = weighted_blend();
        assert_eq!(weighted.dst_color_blend_factor, vk::BlendFactor::ONE);
        assert_eq!(weighted.src_alpha_blend_factor, vk::BlendFactor::ZERO);
        assert_eq!(
            weighted.dst_alpha_blend_factor,
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
        );
        assert_eq!(weighted.color_write_mask, vk::ColorComponentFlags::RGBA);
        let peeling = peeling_blend();
        assert_eq!(peeling.color_blend_op, vk::BlendOp::MAX);
        assert_eq!(peeling.alpha_blend_op, vk::BlendOp::MAX);
    }
    #[test]
    fn composites_preserve_scene_alpha_and_back_blending_uses_source_alpha() {
        let weighted = weighted_composite_blend();
        let peeling = peeling_composite_blend();
        assert_eq!(
            weighted.src_color_blend_factor,
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
        );
        assert_eq!(weighted.dst_color_blend_factor, vk::BlendFactor::SRC_ALPHA);
        assert_eq!(peeling.src_color_blend_factor, vk::BlendFactor::ONE);
        assert_eq!(
            peeling.dst_color_blend_factor,
            vk::BlendFactor::ONE_MINUS_SRC_ALPHA
        );
        assert_eq!(
            peeling.color_write_mask,
            vk::ColorComponentFlags::from_raw(0x7)
        );
    }
    #[test]
    fn back_composite_preserves_accumulated_alpha_and_all_color_channels() {
        let back = peeling_back_blend();
        assert_eq!(back.src_color_blend_factor, vk::BlendFactor::SRC_ALPHA);
        assert_eq!(back.src_alpha_blend_factor, vk::BlendFactor::ONE);
        assert_eq!(back.color_write_mask, vk::ColorComponentFlags::RGBA);
    }
}
