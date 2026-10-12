//! CPU regressions keep encoded XOR inputs separate from ordinary linear glyph colors.
use super::tests::{identity_text_transform, minimal_glyph_layout};
use super::*;
#[test]
fn xor_glyph_submission_stays_linear_and_alpha_retains_native_cutoff_input() {
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_text_display_type(AspectTypeOfDisplayText::Blend);
    aspect.set_interior_color_rgba(ColorRgba::new(0.18, 0.4, 0.7, 0.4).unwrap());
    let draws = lower_xor_fixture(&aspect);
    assert_eq!(draws.len(), 1);
    assert!(draws[0].text_logic_xor && !draws[0].text_shadow_bias);
    let expected = [0.18, 0.4, 0.7, 0.4];
    assert_eq!(draws[0].vertices()[0].color, expected);
    assert_eq!(
        draws[0].textured_style.as_ref().unwrap().alpha_cutoff,
        0.285
    );
}
#[test]
fn live_normal_style_restores_linear_colors_and_preserves_other_families() {
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_interior_color_rgba(ColorRgba::new(0.18, 0.4, 0.7, 1.).unwrap());
    for display in ORDINARY_FAMILIES {
        aspect.set_text_display_type(display);
        let draws = lower_xor_fixture(&aspect);
        let foreground = draws.last().unwrap();
        assert!(!foreground.text_logic_xor);
        assert_eq!(foreground.vertices()[0].color, [0.18, 0.4, 0.7, 1.]);
    }
}

const ORDINARY_FAMILIES: [AspectTypeOfDisplayText; 4] = [
    AspectTypeOfDisplayText::Normal,
    AspectTypeOfDisplayText::Shadow,
    AspectTypeOfDisplayText::Subtitle,
    AspectTypeOfDisplayText::Dekale,
];
fn lower_xor_fixture(aspect: &Graphic3dAspectText3d) -> Vec<PrimitiveDraw> {
    let text = Graphic3dText::new(16.);
    lower_text_draws(
        &text,
        aspect,
        &minimal_glyph_layout(),
        &identity_text_transform(),
    )
    .unwrap()
}
