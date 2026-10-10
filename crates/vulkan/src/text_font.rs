//! Reuse Martensite's OCCT font adapter and formatter under one per-source font-cache lock.
use crate::{GlyphLayout, RenderError, TextLayoutSource, error::rejected};
use rustycad_tkernel::{
    standard::{Handle, SharedState},
    tcollection::AsciiString,
};
use rustycad_tkservice::{font::*, graphic3d::*};
use std::collections::BTreeMap;
type FontKey = (Vec<u8>, i32, u32, u32, i32);
/// Native FreeType/layout provider; `FreeTypeTextSource::default()` can be shared across groups.
#[derive(Default)]
pub struct FreeTypeTextSource {
    fonts: SharedState<BTreeMap<FontKey, Handle<FontFTFont>>>,
}
impl TextLayoutSource for FreeTypeTextSource {
    fn layout(
        &self,
        text: &Graphic3dText,
        aspects: &dyn Graphic3dAspectsSource,
        params: &Graphic3dRenderingParams,
    ) -> Result<Option<GlyphLayout>, RenderError> {
        if text.text().is_empty() || text.height() < 2. {
            return Ok(None);
        }
        let key = font_key(text, aspects, params);
        // A glyph bitmap borrows the mutable FreeType slot, so serialize formatting and copying.
        let mut fonts = self.fonts.write();
        let font = fonts.entry(key.clone()).or_insert_with(|| load_font(&key));
        if font.is_null() {
            return Ok(None);
        }
        let formatter = prepare_formatter(text, font)?;
        crate::text_atlas::build_glyph_layout(font, &formatter)
    }
}
fn font_key(
    text: &Graphic3dText,
    aspects: &dyn Graphic3dAspectsSource,
    params: &Graphic3dRenderingParams,
) -> FontKey {
    let name = aspects.text_font();
    let name = name
        .get()
        .map_or_else(|| b"Courier".to_vec(), |s| s.string().as_bytes().to_vec());
    let aspect = match aspects.text_font_aspect() {
        FontFontAspect::Undefined => FontFontAspect::Regular,
        other => other,
    };
    (
        name,
        aspect as i32,
        text.height() as u32,
        params.resolution,
        params.font_hinting.0,
    )
}
fn load_font(key: &FontKey) -> Handle<FontFTFont> {
    let aspect = match key.1 {
        1 => FontFontAspect::Bold,
        2 => FontFontAspect::Italic,
        3 => FontFontAspect::BoldItalic,
        _ => FontFontAspect::Regular,
    };
    let params = FontFTFontParams {
        point_size: key.2,
        resolution: key.3,
        font_hinting: FontHinting(key.4),
        ..Default::default()
    };
    FontFTFont::find_and_create(
        &AsciiString::from_view(&key.0),
        aspect,
        &params,
        FontStrictLevel::Any,
    )
}
fn prepare_formatter(
    text: &Graphic3dText,
    font: &FontFTFont,
) -> Result<Handle<FontTextFormatter>, RenderError> {
    let formatter = text.text_formatter();
    let formatter = if formatter.is_null() {
        Handle::new(FontTextFormatter::default())
    } else {
        formatter
    };
    formatter.setup_alignment(text.horizontal_alignment(), text.vertical_alignment());
    formatter.get().unwrap().reset();
    formatter
        .append(&text.text(), font)
        .map_err(|e| rejected(e, "a formattable native text string"))?;
    formatter
        .format()
        .map_err(|e| rejected(e, "a valid native text layout"))?;
    Ok(formatter)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn font_cache_key_distinguishes_resolution_height_aspect_and_hinting() {
        let text = Graphic3dText::new(16.9);
        let style = Graphic3dAspectText3d::default();
        let mut params = Graphic3dRenderingParams::new().unwrap();
        let initial = font_key(&text, &style, &params);
        assert_eq!(initial.2, 16);
        params.resolution = 144;
        assert_ne!(initial, font_key(&text, &style, &params));
        let updated = font_key(&text, &style, &params);
        text.set_height(24.);
        assert_ne!(updated, font_key(&text, &style, &params));
        let source = FreeTypeTextSource::default();
        assert!(source.layout(&text, &style, &params).unwrap().is_none());
        assert!(source.fonts.read().is_empty());
    }
}
