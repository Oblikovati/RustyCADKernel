//! Native FFP disables framebuffer sRGB for font programs without switching font material state.
use crate::RenderFrame;

pub(crate) fn capture_text_color_target(frame: &mut RenderFrame) {
    if !frame.raster.fixed_function {
        return;
    }
    for draw in &mut frame.draws {
        if draw.textured_style.is_some() {
            draw.compatibility_color_target = true;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn font_target_changes_without_claiming_fixed_lighting_or_resetting_inherited_alpha() {
        let image = crate::RasterImage::new([1, 1], vec![[1.; 4]]).unwrap();
        let style = crate::TexturedStyle::new(image, vec![[0.; 2]; 3], 0.1).unwrap();
        let draw = crate::PrimitiveDraw::textured(
            vec![crate::RenderVertex::new([0.; 3], [1.; 4]); 3],
            vec![0, 1, 2],
            crate::identity_clip(),
            style,
        )
        .unwrap();
        let mut frame = RenderFrame::new(2, 2, [0.; 4], vec![draw]).unwrap();
        capture_text_color_target(&mut frame);
        assert!(!frame.draws[0].compatibility_color_target);
        frame.raster.fixed_function = true;
        capture_text_color_target(&mut frame);
        let draw = &frame.draws[0];
        assert!(draw.compatibility_color_target);
        assert!(!draw.fixed_function_lighting);
        let mut state = crate::legacy_fragment_alpha::LegacyFragmentAlpha(Some(0.5));
        assert_eq!(state.advance(draw), Some(0.5));
    }
}
