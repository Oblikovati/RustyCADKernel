//! Background bounds include advance whitespace and preserve native triangle-strip winding.
use super::*;
use crate::RasterImage;
use rustycad_tkernel::{ncollection::Mat4, quantity::ColorRgba};
use rustycad_tkservice::{aspect::*, font::FontRect};
fn subtitle_fixture_rectangle(right: f32) -> FontRect {
    FontRect {
        left: 0.,
        right,
        top: 10.,
        bottom: -15.,
    }
}
fn subtitle_fixture_layout(rectangle: FontRect) -> GlyphLayout {
    GlyphLayout {
        positions: vec![[0., 0.]; 3],
        uv: vec![[0.; 2]; 3],
        indices: vec![0, 1, 2],
        atlas: RasterImage::new([1, 1], vec![[1.; 4]]).unwrap(),
        formatter_bounds: rectangle,
        point_size: 16,
    }
}
fn subtitle_fixture_transform() -> TextTransform {
    TextTransform {
        projection: Mat4::identity(),
        model_view: Mat4::identity(),
        camera_rotation: Mat4::identity(),
        viewport: [0, 0, 128, 128],
    }
}
fn subtitle_fixture_draws(aspect: &Graphic3dAspectText3d) -> Vec<PrimitiveDraw> {
    aspect.set_text_display_type(AspectTypeOfDisplayText::Subtitle);
    let layout = subtitle_fixture_layout(subtitle_fixture_rectangle(32.));
    crate::text_drawing::lower_text_draws(
        &Graphic3dText::new(16.),
        aspect,
        &layout,
        &subtitle_fixture_transform(),
    )
    .unwrap()
}
#[test]
fn rectangle_retains_native_formatter_winding_and_transparent_color() {
    let layout = subtitle_fixture_layout(subtitle_fixture_rectangle(-32.));
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_edge_color_rgba(ColorRgba::new(1., 0., 0., 0.).unwrap());
    let vertices = subtitle_vertices(&layout, &aspect);
    let positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.position).collect();
    let expected = [
        [-32., -15., 0.],
        [-32., 10., 0.],
        [0., -15., 0.],
        [0., 10., 0.],
    ];
    assert_eq!(positions, expected);
    assert!(vertices.iter().all(|v| v.color == [1., 0., 0., 0.]));
}
#[test]
fn subtitle_pass_precedes_glyphs_and_keeps_ordered_depth_bias_and_blend() {
    let draws = subtitle_fixture_draws(&Graphic3dAspectText3d::default());
    assert_eq!(draws.len(), 2);
    assert_eq!(draws[0].indices(), [0, 1, 2, 1, 3, 2]);
    assert!(draws[0].textured_style().is_none());
    assert!(draws[1].textured_style().is_some());
    assert!(!draws[0].surface_blend && draws[0].text_background_blend && draws[0].text_shadow_bias);
    assert!(!draws[1].text_shadow_bias);
}
#[test]
fn annotation_and_opaque_subtitle_disable_blending_and_temporary_bias() {
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_text_style(AspectTypeOfStyleText::Annotation);
    aspect.set_alpha_mode(Graphic3dAlphaMode::Opaque, 0.285);
    aspect.set_edge_color_rgba(ColorRgba::new(0., 0., 1., 0.).unwrap());
    let draws = subtitle_fixture_draws(&aspect);
    assert!(draws[0].annotation && !draws[0].text_shadow_bias);
    assert!(!draws[0].surface_blend && !draws[0].text_background_blend);
    assert_eq!(draws[0].vertices()[0].color, [0., 0., 1., 0.]);
}
