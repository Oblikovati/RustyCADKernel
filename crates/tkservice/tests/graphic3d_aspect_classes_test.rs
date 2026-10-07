//! Graphic3d_Aspects members and the four primitive aspects classes; the dump text is
//! compared with the native classes by the sv_aspects_dump probe.
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::aspect::{
    AspectHatchStyle, AspectInteriorStyle, AspectTypeOfDisplayText, AspectTypeOfLine,
    AspectTypeOfMarker, AspectTypeOfStyleText,
};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkservice::image::{ImageFormat, ImagePixMap};

#[test]
fn line_types_and_patterns_follow_each_other() {
    let aspects = Graphic3dAspects::default();
    aspects.set_line_type(AspectTypeOfLine::Dash);
    assert_eq!(aspects.line_pattern(), 0xFFC0);
    aspects.set_line_pattern(0x1234);
    assert_eq!(aspects.line_type(), AspectTypeOfLine::Userdefined);
    assert_eq!(
        graphic3d_default_line_type_for_pattern(0xCCCC),
        AspectTypeOfLine::Dot
    );
    assert!(aspects.set_line_stipple_factor(0).is_err());
    assert!(aspects.set_line_stipple_factor(257).is_err());
    aspects.set_line_stipple_factor(256).unwrap();
    assert_eq!(aspects.line_stipple_factor(), 256);
    assert_eq!(
        aspects.set_line_width(0.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn edges_turn_an_empty_line_solid() {
    let aspects = Graphic3dAspects::default();
    aspects.set_line_type(AspectTypeOfLine::Empty);
    aspects.set_draw_edges(true);
    assert_eq!(aspects.line_type(), AspectTypeOfLine::Solid);
    assert!(aspects.to_draw_edges());
}

#[test]
fn polygon_offsets_mask_the_mode() {
    let aspects = Graphic3dAspects::default();
    aspects.set_polygon_offsets(0x7F, 2.0, 3.0);
    assert_eq!(aspects.polygon_offsets(), (0x7F & 0x0F, 2.0, 3.0));
}

#[test]
fn texture_maps_wrap_into_a_set() {
    let aspects = Graphic3dAspects::default();
    assert!(aspects.texture_map().is_null());
    let pixmap = ImagePixMap::default();
    pixmap.init_zero(ImageFormat::Rgb, 2, 2, 0, 0);
    let texture: Handle<dyn Graphic3dTextureMap> =
        Handle::new(Graphic3dTexture2D::from_pix_map(Handle::new(pixmap))).cast();
    aspects.set_texture_map(texture.clone());
    assert!(aspects.texture_map() == texture);
    aspects.set_texture_map(Handle::default());
    assert!(aspects.texture_set().is_null());
}

#[test]
fn markers_are_sprites_unless_points_or_empty_user_markers() {
    let aspects = Graphic3dAspectMarker3d::default();
    assert_eq!(aspects.marker_kind(), AspectTypeOfMarker::X);
    assert!(aspects.is_marker_sprite());
    aspects.set_marker_kind(AspectTypeOfMarker::Userdefined);
    assert!(!aspects.is_marker_sprite());
    assert_eq!(aspects.get_texture_size(), (0, 0));
    assert!(
        Graphic3dAspectMarker3d::new(
            AspectTypeOfMarker::Plus,
            Color::named(NamedColor::RED),
            -1.0
        )
        .is_err()
    );
}

#[test]
fn primitive_aspects_start_from_their_defaults() {
    let fill = Graphic3dAspectFillArea3d::default();
    assert_eq!(fill.interior_style(), AspectInteriorStyle::Empty);
    let line =
        Graphic3dAspectLine3d::new(Color::named(NamedColor::RED), AspectTypeOfLine::Dot, 2.0)
            .unwrap();
    assert_eq!(
        (line.line_kind(), line.width(), line.line_pattern()),
        (AspectTypeOfLine::Dot, 2.0, 0xCCCC)
    );
    assert_eq!(line.shading_model(), Graphic3dTypeOfShadingModel::Unlit);
    let text = Graphic3dAspectText3d::new(
        Color::named(NamedColor::RED),
        Some("Arial"),
        1.0,
        0.0,
        AspectTypeOfStyleText::Normal,
        AspectTypeOfDisplayText::Subtitle,
    );
    assert_eq!(text.font().as_bytes(), b"Arial");
    assert_eq!(
        (text.alpha_mode(), text.alpha_cutoff()),
        (Graphic3dAlphaMode::MaskBlend, 0.285)
    );
    text.set_font(&AsciiString::default());
    assert!(text.text_font().is_null());
}

#[test]
fn hatch_styles_and_equality_use_identity() {
    let (a, b) = (Graphic3dAspects::default(), Graphic3dAspects::default());
    a.set_hatch_style_type(AspectHatchStyle::Grid);
    assert!(!a.is_equal(&b));
    b.set_hatch_style(a.hatch_style());
    assert!(a.is_equal(&b));
    a.set_hatch_style_type(AspectHatchStyle::Solid);
    assert!(a.hatch_style().is_null());
}
