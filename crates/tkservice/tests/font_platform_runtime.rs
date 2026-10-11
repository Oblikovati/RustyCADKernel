//! Required native font dependency checks never silently skip rendering failures.
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkservice::font::{FontFTFont, FontFTFontParams, FontFTLibrary, FontFontMgr};

#[test]
fn required_freetype_loads_and_rasterizes_embedded_glyph() {
    assert!(
        FontFTLibrary::new().is_valid(),
        "expected installed native FreeType library"
    );
    let font = FontFTFont::default();
    assert!(font.init_from(
        &FontFontMgr::embed_fallback_font(),
        &AsciiString::from("Martensite embedded regression"),
        &FontFTFontParams {
            point_size: 14,
            ..Default::default()
        },
        0,
    ));
    assert!(font.render_glyph('A' as u32));
    assert!(!font.glyph_image().is_null());
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn native_platform_discovery_finds_fonts_that_freetype_can_rasterize() {
    use rustycad_tkservice::font::{FontFontAspect, FontStrictLevel};
    let registry = FontFontMgr::get_instance();
    assert!(
        !registry.get_available_fonts().is_empty(),
        "received empty font catalog; expected installed native platform fonts"
    );
    let params = FontFTFontParams {
        point_size: 14,
        ..Default::default()
    };
    let font = FontFTFont::find_and_create(
        &AsciiString::default(),
        FontFontAspect::Regular,
        &params,
        FontStrictLevel::Any,
    );
    assert!(!font.is_null(), "expected usable registered system font");
    assert!(font.render_glyph('A' as u32));
}
