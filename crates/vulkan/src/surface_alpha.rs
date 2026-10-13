//! Surface alpha classification is shared by ordinary geometry and section batches.
use crate::{PrimitiveDraw, PrimitiveTopology};
use rustycad_tkservice::graphic3d::*;
pub(crate) fn apply_surface_alpha(draw: &mut PrimitiveDraw, aspects: &dyn Graphic3dAspectsSource) {
    crate::legacy_fragment_alpha::capture_material_alpha(draw, aspects);
    if draw.topology() != PrimitiveTopology::Triangles || draw.textured_style.is_some() {
        return;
    }
    draw.surface_blend = crate::style::surface_uses_blending(aspects);
    draw.surface_forced_blend = aspects.alpha_mode() == Graphic3dAlphaMode::MaskBlend;
    draw.surface_cutoff = matches!(
        aspects.alpha_mode(),
        Graphic3dAlphaMode::Mask | Graphic3dAlphaMode::MaskBlend
    )
    .then_some(aspects.alpha_cutoff());
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_alpha_modes_keep_mask_blend_ordered_and_blend_deferred() {
        let aspects = Graphic3dAspects::default();
        let mut draw = alpha_triangle_fixture();
        for (mode, deferred, forced, cutoff) in [
            (Graphic3dAlphaMode::Opaque, false, false, None),
            (Graphic3dAlphaMode::Mask, false, false, Some(0.25)),
            (Graphic3dAlphaMode::MaskBlend, false, true, Some(0.25)),
            (Graphic3dAlphaMode::Blend, true, false, None),
            (Graphic3dAlphaMode::BlendAuto, false, false, None),
        ] {
            aspects.set_alpha_mode(mode, 0.25);
            apply_surface_alpha(&mut draw, &aspects);
            assert_eq!(
                (
                    draw.surface_blend,
                    draw.surface_forced_blend,
                    draw.surface_cutoff
                ),
                (deferred, forced, cutoff),
                "{mode:?}"
            );
        }
    }
    fn alpha_triangle_fixture() -> PrimitiveDraw {
        PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
        )
        .unwrap()
    }
}
