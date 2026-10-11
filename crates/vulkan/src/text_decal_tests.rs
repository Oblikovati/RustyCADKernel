//! Native Decal order shares the existing atlas, offset matrix and temporary bias contract.
use super::tests::{identity_text_transform, minimal_glyph_layout};
use super::*;
#[test]
fn decal_passes_keep_native_order_outline_color_and_foreground_bias() {
    let style = Graphic3dAspectText3d::default();
    style.set_text_display_type(AspectTypeOfDisplayText::Dekale);
    style.set_edge_color_rgba(ColorRgba::new(1., 0., 0., 0.4).unwrap());
    let text = Graphic3dText::new(16.);
    let layout = minimal_glyph_layout();
    let transform = identity_text_transform();
    let draws = lower_text_draws(&text, &style, &layout, &transform).unwrap();
    assert_eq!(draws.len(), 5);
    assert_decal_outline_sequence(&draws, &text, &style, &transform);
    assert!(!draws[4].text_shadow_bias);
}
fn assert_decal_outline_sequence(
    draws: &[PrimitiveDraw],
    text: &Graphic3dText,
    style: &Graphic3dAspectText3d,
    transform: &TextTransform,
) {
    for (draw, offset) in draws.iter().zip(DECAL_PIXEL_OFFSETS) {
        assert_eq!(
            *draw.clip_transform(),
            transform.clip_offset(text, style, 16, offset).unwrap()
        );
        assert_eq!(draw.vertices()[0].color, [1., 0., 0., 0.4]);
        assert!(draw.text_shadow_bias && !draw.surface_blend);
    }
}
#[test]
fn annotation_decal_keeps_all_passes_without_temporary_depth_bias() {
    let style = Graphic3dAspectText3d::default();
    style.set_text_display_type(AspectTypeOfDisplayText::Dekale);
    style.set_text_style(AspectTypeOfStyleText::Annotation);
    let draws = lower_text_draws(
        &Graphic3dText::new(16.),
        &style,
        &minimal_glyph_layout(),
        &identity_text_transform(),
    )
    .unwrap();
    assert_eq!(draws.len(), 5);
    assert!(
        draws
            .iter()
            .all(|draw| draw.annotation && !draw.text_shadow_bias)
    );
}
