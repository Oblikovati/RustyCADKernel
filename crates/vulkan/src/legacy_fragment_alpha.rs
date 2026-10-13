//! Native drawMarkers restores compatibility alpha state even when the next program is GLSL.
use crate::{PrimitiveDraw, point_style::PointColorPolicy};
use rustycad_tkservice::{aspect::AspectInteriorStyle, graphic3d::*};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LegacyFragmentAlpha(pub Option<f32>);
impl LegacyFragmentAlpha {
    pub fn advance(&mut self, draw: &PrimitiveDraw) -> Option<f32> {
        if draw.fixed_function_lighting {
            self.0 = restored_material_alpha(draw.native_material_cutoff);
        }
        if draw
            .point_style
            .as_ref()
            .is_some_and(|style| style.color_policy != PointColorPolicy::Plain)
        {
            self.0 = restored_material_alpha(draw.native_material_cutoff);
            return Some(0.1);
        }
        self.0
    }
}
fn restored_material_alpha(cutoff: Option<f32>) -> Option<f32> {
    // Native ShortRealLast disables the compatibility test even for an explicit mask mode.
    cutoff
        .filter(|reference| *reference < f32::MAX)
        .map(|reference| reference.clamp(0., 1.))
}
pub(crate) fn capture_material_alpha(
    draw: &mut PrimitiveDraw,
    aspects: &dyn Graphic3dAspectsSource,
) {
    draw.native_material_cutoff = native_material_alpha_cutoff(aspects);
}
fn native_material_alpha_cutoff(aspects: &dyn Graphic3dAspectsSource) -> Option<f32> {
    let interior = aspects.interior_style();
    if aspects.to_draw_edges()
        && (interior == AspectInteriorStyle::Empty
            || (interior == AspectInteriorStyle::Solid && aspects.edge_color_rgba().alpha() < 1.))
    {
        return Some(0.285);
    }
    matches!(
        aspects.alpha_mode(),
        Graphic3dAlphaMode::Mask | Graphic3dAlphaMode::MaskBlend
    )
    .then(|| aspects.alpha_cutoff())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_sprite_restores_material_reference_while_programmable_fill_keeps_it() {
        let aspects = Graphic3dAspects::default();
        aspects.set_alpha_mode(Graphic3dAlphaMode::Mask, 0.5);
        let mut marker = alpha_sprite();
        capture_material_alpha(&mut marker, &aspects);
        let mut state = LegacyFragmentAlpha::default();
        assert_eq!(state.advance(&marker), Some(0.1));
        assert_eq!(state.0, Some(0.5));
        let mut fill = alpha_triangle();
        assert_eq!(state.advance(&fill), Some(0.5));
        fill.fixed_function_lighting = true;
        assert_eq!(state.advance(&fill), None);
    }
    #[test]
    fn opaque_sprite_clears_previous_state_and_native_reference_is_clamped() {
        let mut state = LegacyFragmentAlpha(Some(0.75));
        let mut marker = alpha_sprite();
        assert_eq!(state.advance(&marker), Some(0.1));
        assert_eq!(state.0, None);
        marker.native_material_cutoff = Some(2.);
        state.advance(&marker);
        assert_eq!(state.0, Some(1.));
    }
    #[test]
    fn edge_background_material_threshold_overrides_explicit_mask_reference() {
        let aspects = Graphic3dAspects::default();
        aspects.set_alpha_mode(Graphic3dAlphaMode::MaskBlend, 0.75);
        assert_eq!(native_material_alpha_cutoff(&aspects), Some(0.75));
        aspects.set_draw_edges(true);
        aspects.set_interior_style(AspectInteriorStyle::Empty);
        assert_eq!(native_material_alpha_cutoff(&aspects), Some(0.285));
    }
    #[test]
    fn native_short_real_last_restoration_disables_the_context_alpha_test() {
        let mut marker = alpha_sprite();
        marker.native_material_cutoff = Some(f32::MAX);
        let mut state = LegacyFragmentAlpha(Some(0.5));
        assert_eq!(state.advance(&marker), Some(0.1));
        assert_eq!(state.0, None);
        assert_eq!(restored_material_alpha(Some(-2.)), Some(0.));
    }
    fn alpha_triangle() -> PrimitiveDraw {
        PrimitiveDraw::new(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
        )
        .unwrap()
    }
    fn alpha_sprite() -> PrimitiveDraw {
        let style = crate::PointStyle::new(1., [1, 1], vec![[1.; 4]]).unwrap();
        PrimitiveDraw::points(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4])],
            vec![0],
            crate::identity_clip(),
            style,
        )
        .unwrap()
    }
}
