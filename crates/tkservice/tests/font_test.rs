//! Direct translation of Font_UnicodeSubset_Test.cxx, Font_GlyphOutline_Test.cxx,
//! Font_FontMgr_Test.cxx and Font_FTFont_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! `GTEST_SKIP` returns early (the test passes), as the skipped GoogleTest does not fail.
#![allow(non_snake_case)]
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::{
    FontFTFont, FontFTFontParams, FontFontAspect, FontFontMgr, FontGlyphOutline,
    FontGlyphOutlineContour, FontGlyphOutlineFillRule, FontGlyphOutlineSegment,
    FontGlyphOutlineSegmentKind, FontHinting, FontRect, FontStrictLevel, FontUnicodeSubset,
};
use std::sync::Arc;

fn params(point_size: u32, resolution: u32) -> FontFTFontParams {
    FontFTFontParams {
        point_size,
        resolution,
        ..Default::default()
    }
}

/// `createEmbeddedFont`.
fn create_embedded_font(point_size: u32, resolution: u32) -> Option<Handle<FontFTFont>> {
    let font = FontFTFont::default();
    let name = AsciiString::from("Embedded fallback");
    font.init_from(
        &FontFontMgr::embed_fallback_font(),
        &name,
        &params(point_size, resolution),
        0,
    )
    .then(|| Handle::new(font))
}

fn embedded() -> Option<Handle<FontFTFont>> {
    create_embedded_font(72, 4800)
}

#[test]
fn Font_UnicodeSubsetTest__CountIncludesEveryDeclaredSubset() {
    assert_eq!(4, FontUnicodeSubset::Nb as i32);
    for subset in [
        FontUnicodeSubset::Western,
        FontUnicodeSubset::Korean,
        FontUnicodeSubset::Cjk,
        FontUnicodeSubset::Arabic,
    ] {
        assert!((subset as i32) < FontUnicodeSubset::Nb as i32);
    }
}

#[test]
fn Font_GlyphOutlineTest__CreatesOutline() {
    let segments = vec![
        FontGlyphOutlineSegment::line(Xy::new(0.0, 0.0), Xy::new(10.0, 0.0)),
        FontGlyphOutlineSegment::quadratic(
            Xy::new(10.0, 0.0),
            Xy::new(12.0, 5.0),
            Xy::new(0.0, 0.0),
        ),
    ];
    let contours = vec![FontGlyphOutlineContour::new(0, 2)];
    let outline = FontGlyphOutline::create(
        2048.0,
        FontGlyphOutlineFillRule::EvenOdd,
        true,
        false,
        segments,
        contours,
    );
    assert_eq!(2048.0, outline.units_per_em());
    assert_eq!(FontGlyphOutlineFillRule::EvenOdd, outline.rule());
    assert!(outline.has_possible_overlaps());
    assert!(!outline.is_single_stroke());
    assert_eq!(2, outline.segments().len());
    assert_eq!(1, outline.contours().len());
    assert_eq!(
        FontGlyphOutlineSegmentKind::QuadraticBezier,
        outline.segments()[1].kind()
    );
    assert_eq!(0, outline.contours()[0].first_segment());
    assert_eq!(2, outline.contours()[0].nb_segments());
}

#[test]
fn Font_FontMgrTest__EmbeddedFallbackDataIsAvailable() {
    let buffer = FontFontMgr::embed_fallback_font();
    assert!(buffer.is_some());
    assert!(buffer.size() > 0);
}

#[test]
fn Font_FontMgrTest__StrictMissingFamilyIsNotResolved() {
    let manager = FontFontMgr::get_instance();
    let mut aspect = FontFontAspect::Regular;
    let name = AsciiString::from("OCCT-Definitely-Missing-Font-Family-4A72C3");
    assert!(
        manager
            .find_font(&name, FontStrictLevel::Strict, &mut aspect, false)
            .is_null()
    );
}

#[test]
fn Font_FontMgrTest__ResolvedFontProvidesUsablePath() {
    let manager = FontFontMgr::get_instance();
    let mut aspect = FontFontAspect::Regular;
    let font = manager.find_font(
        &AsciiString::default(),
        FontStrictLevel::Any,
        &mut aspect,
        false,
    );
    let Some(font) = font.get() else { return };
    let (mut italic, mut face) = (false, 0);
    assert!(
        !font
            .font_path_any(aspect, &mut italic, &mut face)
            .is_empty()
    );
}

#[test]
fn Font_FTFontTest__UninitializedAccessorsReturnEmptyValues() {
    let font = FontFTFont::default();
    assert!(!font.is_valid());
    assert!(!font.has_symbol('A' as u32));
    assert!(!font.render_glyph('A' as u32));
    assert_eq!(0, font.glyph_max_size_x(false));
    assert_eq!(0, font.glyph_max_size_y(false));
    assert_eq!(0.0, font.ascender());
    assert_eq!(0.0, font.descender());
    assert_eq!(0.0, font.line_spacing());
    assert_eq!(0.0, font.advance_x_of('A' as u32, 'B' as u32));
    assert_eq!(0.0, font.advance_y_of('A' as u32, 'B' as u32));
    assert_eq!(0, font.glyphs_number(false));
    let mut rect = FontRect {
        left: 1.0,
        right: 2.0,
        top: 3.0,
        bottom: 4.0,
    };
    font.glyph_rect(&mut rect);
    assert_eq!(
        (rect.left, rect.right, rect.top, rect.bottom),
        (0.0, 0.0, 0.0, 0.0)
    );
}

#[test]
fn Font_FTFontTest__LoadsResolvedGlyphOutline() {
    let Some(font) = embedded() else { return };
    let glyph = font.resolve_glyph('A' as u32).unwrap();
    assert!(font.is_glyph_valid(&glyph));
    let outline = font.load_glyph_outline(&glyph).unwrap();
    assert!(outline.units_per_em() > 0.0);
    assert!(!outline.is_empty());
    assert!(!outline.segments().is_empty());
    assert!(!outline.contours().is_empty());
}

#[test]
fn Font_FTFontTest__WidthScalingInvalidatesResolvedGlyph() {
    let Some(font) = embedded() else { return };
    let old = font.resolve_glyph('I' as u32).unwrap();
    assert!(font.set_width_scaling(1.25));
    assert!(!font.is_glyph_valid(&old));
    assert!(font.load_glyph_outline(&old).is_none());
    let new = font.resolve_glyph('I' as u32).unwrap();
    assert_ne!(old, new);
    assert!(font.is_glyph_valid(&new));
    assert!(font.load_glyph_outline(&new).is_some());
}

#[test]
fn Font_FTFontTest__IndependentFontsRejectForeignGlyph() {
    let (Some(first), Some(second)) = (embedded(), embedded()) else {
        return;
    };
    let first_glyph = first.resolve_glyph('I' as u32).unwrap();
    let second_glyph = second.resolve_glyph('I' as u32).unwrap();
    assert!(!Arc::ptr_eq(first_glyph.owner(), second_glyph.owner()));
    assert!(!second.is_glyph_valid(&first_glyph));
    assert!(second.load_glyph_outline(&first_glyph).is_none());
    assert!(second.is_glyph_valid(&second_glyph));
}

#[test]
fn Font_FTFontTest__InvalidWidthScalingPreservesResolvedGlyph() {
    let Some(font) = embedded() else { return };
    let glyph = font.resolve_glyph('I' as u32).unwrap();
    assert!(!font.set_width_scaling(0.0));
    assert!(!font.set_width_scaling(-1.0));
    assert_eq!(1.0, font.width_scaling());
    assert!(font.is_glyph_valid(&glyph));
}

#[test]
fn Font_FTFontTest__SingleStrokeModeInvalidatesResolvedGlyph() {
    let Some(font) = embedded() else { return };
    let old = font.resolve_glyph('I' as u32).unwrap();
    font.set_single_stroke_font(true);
    assert!(!font.is_glyph_valid(&old));
    let new = font.resolve_glyph('I' as u32).unwrap();
    let outline = font.load_glyph_outline(&new).unwrap();
    assert!(outline.is_single_stroke());
}

#[test]
fn Font_FTFontTest__FindAndInitPreservesExplicitSingleStrokeMode() {
    let manager = FontFontMgr::get_instance();
    let mut aspect = FontFontAspect::Regular;
    let system_font = manager.find_font(
        &AsciiString::default(),
        FontStrictLevel::Any,
        &mut aspect,
        false,
    );
    if system_font.get().is_none_or(|f| f.is_single_stroke_font()) {
        return;
    }
    let mut p = params(72, 4800);
    p.is_single_stroke_font = true;
    let font = FontFTFont::default();
    assert!(font.find_and_init(&AsciiString::default(), aspect, &p, FontStrictLevel::Any));
    assert!(font.is_single_stroke_font());
    let outline = font.load_glyph_outline_char('A' as u32).unwrap();
    assert!(outline.is_single_stroke());
}

#[test]
fn Font_FTFontTest__FallbackPolicyInvalidatesResolvedGlyph() {
    let Some(font) = embedded() else { return };
    let old = font.resolve_glyph('A' as u32).unwrap();
    font.set_use_unicode_subset_fallback(!font.to_use_unicode_subset_fallback());
    assert!(!font.is_glyph_valid(&old));
    let new = font.resolve_glyph('A' as u32).unwrap();
    assert_ne!(old, new);
    assert!(font.is_glyph_valid(&new));
}

#[test]
fn Font_FTFontTest__OutlineLoadingClearsBitmapState() {
    let Some(font) = create_embedded_font(12, 72).filter(|f| f.render_glyph('A' as u32)) else {
        return;
    };
    let mut rendered = FontRect::default();
    font.glyph_rect(&mut rendered);
    assert!(rendered.right - rendered.left > 0.0);
    assert!(rendered.top - rendered.bottom > 0.0);
    assert!(font.load_glyph_outline_char('A' as u32).is_some());
    let mut rect = FontRect {
        left: 1.0,
        right: 2.0,
        top: 3.0,
        bottom: 4.0,
    };
    font.glyph_rect(&mut rect);
    assert_eq!(
        (rect.left, rect.right, rect.top, rect.bottom),
        (0.0, 0.0, 0.0, 0.0)
    );
}

#[test]
fn Font_FTFontTest__ReleaseInvalidatesResolvedGlyph() {
    let Some(font) = embedded() else { return };
    let glyph = font.resolve_glyph('A' as u32).unwrap();
    font.release();
    assert!(!font.is_glyph_valid(&glyph));
    assert!(!font.is_valid());
    assert!(font.resolve_glyph('A' as u32).is_none());
}

#[test]
fn Font_FTFontTest__FailedReinitializationPreservesLoadedFace() {
    let Some(font) = embedded() else { return };
    let glyph = font.resolve_glyph('A' as u32).unwrap();
    assert!(!font.init(
        &AsciiString::from("OCCT-Missing-Font-4F9E2C.ttf"),
        &params(72, 4800),
        0
    ));
    assert!(font.is_valid() && font.is_glyph_valid(&glyph));
    let name = AsciiString::from("Embedded fallback");
    assert!(!font.init_from(
        &FontFontMgr::embed_fallback_font(),
        &name,
        &params(0, 4800),
        0
    ));
    assert!(font.is_valid() && font.is_glyph_valid(&glyph));
    let mut conflicting = params(72, 4800);
    conflicting.font_hinting = FontHinting(FontHinting::LIGHT.0 | FontHinting::NORMAL.0);
    assert!(!font.init_from(&FontFontMgr::embed_fallback_font(), &name, &conflicting, 0));
    assert!(font.is_valid() && font.is_glyph_valid(&glyph));
    assert!(font.load_glyph_outline(&glyph).is_some());
}

#[test]
fn Font_FTFontTest__ZeroCharacterHasNoResolvedGlyph() {
    let Some(font) = embedded() else { return };
    assert!(font.resolve_glyph(0).is_none());
}

#[test]
fn Font_FTFontTest__ParamsValidationIsBackendIndependent() {
    let p = FontFTFontParams {
        point_size: u32::MAX,
        ..Default::default()
    };
    assert!(p.is_valid());
}
