//! True2D contracts differ deliberately from retained Graphic3d text/persistence.
use super::*;
use crate::text_drawing::tests::minimal_glyph_layout;
use rustycad_tkernel::quantity::ColorRgba;
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkservice::aspect::*;
const PIXEL_FAMILIES: [AspectTypeOfDisplayText; 4] = [
    AspectTypeOfDisplayText::Blend,
    AspectTypeOfDisplayText::Shadow,
    AspectTypeOfDisplayText::Subtitle,
    AspectTypeOfDisplayText::Dekale,
];
#[test]
fn pixel_positions_ignore_z_and_zoom_and_force_depth_disabled_for_every_family() {
    let text = Graphic3dText::new(16.);
    text.set_position(&Point3::new(12., 8., 99.));
    let aspect = Graphic3dAspectText3d::default();
    let layout = minimal_glyph_layout();
    for display in PIXEL_FAMILIES {
        aspect.set_text_display_type(display);
        let draws = lower_pixel_text(&text, &aspect, &layout, &PixelTextView::default()).unwrap();
        assert!(draws.iter().all(|d| d.annotation && !d.text_shadow_bias));
        let clip = draws.last().unwrap().clip_transform();
        assert_eq!([clip[12], clip[13], clip[14]], [12., 8., 0.]);
        assert_eq!(clip[5], -1.);
        aspect.set_text_zoomable(true);
        let zoomed = lower_pixel_text(&text, &aspect, &layout, &PixelTextView::default()).unwrap();
        assert_eq!(zoomed.last().unwrap().clip_transform(), clip);
    }
}
#[test]
fn pixel_local_plane_consumes_current_world_view_while_own_plane_retains_projection() {
    let text = Graphic3dText::new(16.);
    text.set_orientation(&Axis2::new(Point3::new(12., 8., 5.), Direction3::Z).unwrap());
    let aspect = Graphic3dAspectText3d::default();
    let view = nonidentity_pixel_fixture();
    let layout = minimal_glyph_layout();
    text.set_own_anchor_point(false);
    let local = lower_pixel_text(&text, &aspect, &layout, &view).unwrap();
    assert_eq!(local[0].clip_transform()[12], 28.);
    assert_eq!(local[0].clip_transform()[13], 30.);
    text.set_own_anchor_point(true);
    let own = lower_pixel_text(&text, &aspect, &layout, &view).unwrap();
    assert_eq!(own[0].clip_transform()[12], 12.);
    assert_eq!(own[0].clip_transform()[13], 8.);
}
#[test]
fn pixel_layout_reuse_observes_live_angle_colors_culling_and_mirroring() {
    let text = Graphic3dText::new(16.);
    let aspect = Graphic3dAspectText3d::default();
    let layout = minimal_glyph_layout();
    let view = mirrored_pixel_fixture();
    let first = lower_pixel_text(&text, &aspect, &layout, &view).unwrap();
    aspect.set_text_angle(90.);
    aspect.set_interior_color_rgba(ColorRgba::new(0.18, 0.4, 0.7, 0.4).unwrap());
    aspect.set_text_display_type(AspectTypeOfDisplayText::Blend);
    let changed = lower_pixel_text(&text, &aspect, &layout, &view).unwrap();
    assert_ne!(changed[0].clip_transform(), first[0].clip_transform());
    assert!(changed[0].text_logic_xor && changed[0].mirrored);
    assert_eq!(changed[0].face_culling, RenderFaceCulling::Back);
    assert_eq!(changed[0].vertices()[0].color[3], 0.4);
}
#[test]
fn unsupported_pixel_styles_and_nonfinite_transforms_return_typed_errors() {
    let text = Graphic3dText::new(16.);
    let aspect = Graphic3dAspectText3d::default();
    aspect.set_text_display_type(AspectTypeOfDisplayText::Dimension);
    let layout = minimal_glyph_layout();
    assert!(lower_pixel_text(&text, &aspect, &layout, &PixelTextView::default()).is_err());
    aspect.set_text_display_type(AspectTypeOfDisplayText::Normal);
    let view = PixelTextView {
        projection: Mat4::from_data([f64::NAN; 16]),
        ..Default::default()
    };
    let error = lower_pixel_text(&text, &aspect, &layout, &view)
        .unwrap_err()
        .to_string();
    assert!(error.contains("NaN") && error.contains("finite"));
}

fn mirrored_pixel_fixture() -> PixelTextView {
    PixelTextView {
        mirrored: true,
        face_culling: RenderFaceCulling::Back,
        ..Default::default()
    }
}

fn nonidentity_pixel_fixture() -> PixelTextView {
    let world_view = Mat4::from_data([
        2., 0., 0., 0., 0., 3., 0., 0., 0., 0., 1., 0., 4., 6., -20., 1.,
    ]);
    PixelTextView {
        world_view,
        ..Default::default()
    }
}
