//! The face lookup of `Font_FTFont`: `FindAndInit`, the Unicode-subset fallback faces, the
//! character subsets and the glyph selection.
#![allow(unsafe_code)]
use super::freetype_ffi::{FtFaceRec, freetype};
use super::ft_font::{FontFTFont, FontFTFontParams, face_rec};
use super::glyph_outline::FontGlyphOutlineGlyph;
use super::{FontFontAspect, FontFontMgr, FontStrictLevel, FontUnicodeSubset};
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcollection::AsciiString;
use std::ffi::CStr;

/// `GlyphSelection`: the face that provides a glyph (0 for this font, else the fallback
/// font), the face number of the glyph identity and the glyph index.
#[derive(Clone)]
pub(crate) struct GlyphSelection {
    pub(crate) provider: Option<Handle<FontFTFont>>,
    pub(crate) face: u32,
    pub(crate) index: u32,
}

/// `FT_Get_Char_Index` of a face address (0 for none).
pub(crate) fn char_index(face: usize, character: u32) -> u32 {
    let Some(api) = freetype().filter(|_| face != 0) else {
        return 0;
    };
    // SAFETY: an open face; the call only reads it.
    unsafe { (api.FT_Get_Char_Index)(face as *mut FtFaceRec, std::ffi::c_ulong::from(character)) }
}

impl FontFTFont {
    /// `IsCharFromCJK`; `FontFTFont::is_char_from_cjk(0x4E00)`.
    pub fn is_char_from_cjk(c: u32) -> bool {
        let ranges = [
            (0x03400, 0x04DFF),
            (0x04E00, 0x09FFF),
            (0x0F900, 0x0FAFF),
            (0x20000, 0x2A6DF),
            (0x2F800, 0x2FA1F),
        ];
        ranges.iter().any(|&(lo, hi)| (lo..=hi).contains(&c))
            || Self::is_char_from_hiragana(c)
            || Self::is_char_from_katakana(c)
    }
    /// `IsCharFromHiragana`; `FontFTFont::is_char_from_hiragana(0x3042)`.
    pub fn is_char_from_hiragana(c: u32) -> bool {
        (0x03040..=0x0309F).contains(&c)
    }
    /// `IsCharFromKatakana`; `FontFTFont::is_char_from_katakana(0x30A2)`.
    pub fn is_char_from_katakana(c: u32) -> bool {
        (0x030A0..=0x030FF).contains(&c)
    }
    /// `IsCharFromKorean`; `FontFTFont::is_char_from_korean(0xAC00)`.
    pub fn is_char_from_korean(c: u32) -> bool {
        [(0x01100, 0x011FF), (0x03130, 0x0318F), (0x0AC00, 0x0D7A3)]
            .iter()
            .any(|&(lo, hi)| (lo..=hi).contains(&c))
    }
    /// `IsCharFromArabic`; `FontFTFont::is_char_from_arabic(0x0627)`.
    pub fn is_char_from_arabic(c: u32) -> bool {
        (0x00600..=0x006FF).contains(&c)
    }
    /// `IsCharRightToLeft`: Arabic; `FontFTFont::is_char_right_to_left(0x0627)`.
    pub fn is_char_right_to_left(c: u32) -> bool {
        Self::is_char_from_arabic(c)
    }
    /// `CharSubset`; `FontFTFont::char_subset('A' as u32) == FontUnicodeSubset::Western`.
    pub fn char_subset(c: u32) -> FontUnicodeSubset {
        if Self::is_char_from_cjk(c) {
            FontUnicodeSubset::Cjk
        } else if Self::is_char_from_korean(c) {
            FontUnicodeSubset::Korean
        } else if Self::is_char_from_arabic(c) {
            FontUnicodeSubset::Arabic
        } else {
            FontUnicodeSubset::Western
        }
    }
}

impl FontFTFont {
    /// `FindAndCreate`: a new font found through the font manager, or null.
    /// Example: `FontFTFont::find_and_create(&"sans-serif".into(), aspect, &params, FontStrictLevel::Any)`.
    pub fn find_and_create(
        name: &AsciiString,
        aspect: FontFontAspect,
        params: &FontFTFontParams,
        strict: FontStrictLevel,
    ) -> Handle<FontFTFont> {
        let font = FontFTFont::default();
        if font.find_and_init(name, aspect, params, strict) {
            Handle::new(font)
        } else {
            Handle::default()
        }
    }
    /// `FindAndInit`: the file of the font the manager finds (with its single-stroke flag and
    /// italic synthesis), else (level `Any`) the embedded fallback font.
    /// Example: `font.find_and_init(&"monospace".into(), aspect, &params, FontStrictLevel::Any)`.
    pub fn find_and_init(
        &self,
        name: &AsciiString,
        aspect: FontFontAspect,
        params: &FontFTFontParams,
        strict: FontStrictLevel,
    ) -> bool {
        let mut params = *params;
        let mut found_aspect = aspect;
        let manager = FontFontMgr::get_instance();
        let system_font = manager.find_font(name, strict, &mut found_aspect, true);
        if let Some(system_font) = system_font.get() {
            params.is_single_stroke_font =
                params.is_single_stroke_font || system_font.is_single_stroke_font();
            let mut face_id = 0;
            let path = system_font.font_path_any(
                found_aspect,
                &mut params.to_synthesize_italic,
                &mut face_id,
            );
            if path.is_empty() || !self.init(&path, &params, face_id) {
                return false;
            }
            self.state.write().font_aspect = found_aspect;
            return true;
        }
        if strict != FontStrictLevel::Any || freetype().is_none() {
            return false;
        }
        self.init_embedded(aspect, params)
    }
    /// The embedded fallback face of `FindAndInit` (italic aspects synthesize italic).
    fn init_embedded(&self, aspect: FontFontAspect, mut params: FontFTFontParams) -> bool {
        let resolved = match aspect {
            FontFontAspect::Italic | FontFontAspect::BoldItalic => {
                params.to_synthesize_italic = true;
                FontFontAspect::Italic
            }
            _ => FontFontAspect::Regular,
        };
        let data = FontFontMgr::embed_fallback_font();
        if !self.init_from(&data, &AsciiString::from("Embed Fallback Font"), &params, 0) {
            return false;
        }
        self.state.write().font_aspect = resolved;
        true
    }
    /// `findAndInitFallback`: the fallback face of a subset, created once (without its own
    /// fallbacks); whether it is valid.
    pub(crate) fn find_and_init_fallback(&self, subset: FontUnicodeSubset) -> bool {
        let index = subset as usize;
        let (existing, library, params, aspect) = {
            let state = self.state.read();
            let Some(existing) = state.fallback_faces.get(index) else {
                return false;
            };
            (
                existing.clone(),
                state.ft_lib.clone(),
                state.params,
                state.font_aspect,
            )
        };
        if let Some(existing) = existing.get() {
            return existing.is_valid();
        }
        if freetype().is_none() {
            return false;
        }
        let fallback = FontFTFont::new(library);
        fallback.state.write().to_use_unicode_subset_fallback = false;
        let fallback = Handle::new(fallback);
        self.state.write().fallback_faces[index] = fallback.clone();
        self.init_fallback_face(&fallback, subset, params, aspect);
        fallback.is_valid()
    }
    fn init_fallback_face(
        &self,
        fallback: &FontFTFont,
        subset: FontUnicodeSubset,
        mut params: FontFTFontParams,
        aspect: FontFontAspect,
    ) {
        let system_font = FontFontMgr::get_instance().find_fallback_font(subset, aspect);
        let Some(system_font) = system_font.get() else {
            return;
        };
        let mut face_id = 0;
        params.is_single_stroke_font =
            params.is_single_stroke_font || system_font.is_single_stroke_font();
        let path =
            system_font.font_path_any(aspect, &mut params.to_synthesize_italic, &mut face_id);
        if path.is_empty() || !fallback.init(&path, &params, face_id) {
            return;
        }
        fallback.state.write().font_aspect = aspect;
        let primary = self.primary_name();
        let text = [
            &b"Font_FTFont, using fallback font '"[..],
            system_font.font_name().as_bytes(),
            b"' for symbols unsupported by '",
            &primary,
            b"'",
        ]
        .concat();
        Message::send_trace_text(&AsciiString::from_view(&text));
    }
    /// The family name of the face, else the font path.
    fn primary_name(&self) -> Vec<u8> {
        let state = self.state.read();
        match face_rec(state.face).filter(|f| !f.family_name.is_null()) {
            // SAFETY: an open face keeps its family name NUL-terminated.
            Some(face) => unsafe { CStr::from_ptr(face.family_name) }
                .to_bytes()
                .to_vec(),
            None => state.font_path.as_bytes().to_vec(),
        }
    }
}

impl FontFTFont {
    /// `HasSymbol`: whether the face maps the character; `font.has_symbol('A' as u32)`.
    pub fn has_symbol(&self, character: u32) -> bool {
        char_index(self.state.read().face, character) != 0
    }
    /// `selectGlyph`: this face's glyph, else the fallback face's, else the missing glyph of
    /// this face; `None` without a face or for the NUL character.
    pub(crate) fn select_glyph(&self, character: u32) -> Option<GlyphSelection> {
        let (face, to_fallback) = {
            let state = self.state.read();
            (state.face, state.to_use_unicode_subset_fallback)
        };
        if face == 0 || character == 0 || freetype().is_none() {
            return None;
        }
        let primary = char_index(face, character);
        if primary != 0 {
            return Some(GlyphSelection {
                provider: None,
                face: 0,
                index: primary,
            });
        }
        if let Some(selection) = to_fallback
            .then(|| self.fallback_selection(character))
            .flatten()
        {
            return Some(selection);
        }
        Some(GlyphSelection {
            provider: None,
            face: 0,
            index: 0,
        })
    }
    fn fallback_selection(&self, character: u32) -> Option<GlyphSelection> {
        let subset = Self::char_subset(character);
        if !self.find_and_init_fallback(subset) {
            return None;
        }
        let fallback = self.state.read().fallback_faces[subset as usize].clone();
        let font = fallback.get().filter(|f| f.has_symbol(character))?;
        let index = char_index(font.state.read().face, character);
        Some(GlyphSelection {
            provider: Some(fallback.clone()),
            face: subset as u32 + 1,
            index,
        })
    }
    /// `ResolveGlyph`: the identity of the glyph that would draw the character.
    /// Example: `font.resolve_glyph('A' as u32)`.
    pub fn resolve_glyph(&self, character: u32) -> Option<FontGlyphOutlineGlyph> {
        let selection = self.select_glyph(character)?;
        let state = self.state.read();
        Some(FontGlyphOutlineGlyph::new(
            state.identity.clone(),
            state.revision,
            selection.face,
            selection.index,
        ))
    }
    /// `fontForGlyph`: the font (this or a fallback) of a glyph of the current revision.
    pub(crate) fn font_for_glyph(&self, glyph: &FontGlyphOutlineGlyph) -> Option<FontForGlyph> {
        let state = self.state.read();
        if !std::sync::Arc::ptr_eq(glyph.owner(), &state.identity)
            || glyph.revision() != state.revision
        {
            return None;
        }
        if glyph.face() == 0 {
            return Some(FontForGlyph::This);
        }
        let fallback = state.fallback_faces.get((glyph.face() - 1) as usize)?;
        fallback
            .is_some()
            .then(|| FontForGlyph::Fallback(fallback.clone()))
    }
    /// `IsGlyphValid`: a glyph of this font's revision whose index exists in its face.
    /// Example: `font.is_glyph_valid(&glyph)`.
    pub fn is_glyph_valid(&self, glyph: &FontGlyphOutlineGlyph) -> bool {
        let Some(font) = self.font_for_glyph(glyph) else {
            return false;
        };
        let face = font.face(self);
        face_rec(face).is_some_and(|f| i64::from(glyph.index()) < f.num_glyphs)
    }
}

/// The font a glyph belongs to.
pub(crate) enum FontForGlyph {
    This,
    Fallback(Handle<FontFTFont>),
}

impl FontForGlyph {
    /// The `FT_Face` address of the font.
    pub(crate) fn face(&self, this: &FontFTFont) -> usize {
        match self {
            Self::This => this.state.read().face,
            Self::Fallback(font) => font.get().map_or(0, |f| f.state.read().face),
        }
    }
    /// The single-stroke flag and load flags of the font.
    pub(crate) fn outline_flags(&self, this: &FontFTFont) -> (bool, i32) {
        let read = |font: &FontFTFont| {
            let state = font.state.read();
            (state.params.is_single_stroke_font, state.load_flags)
        };
        match self {
            Self::This => read(this),
            Self::Fallback(font) => font.get().map_or((false, 0), read),
        }
    }
}

impl FontFTFont {
    /// Deprecated `Init(theFontPath, thePointSize, theResolution)`: a file at a size.
    /// Example: `font.init_with_size(&path, 12, 72)`.
    #[deprecated(
        note = "Deprecated method, Font_FTFont::Params should be used for passing parameters"
    )]
    pub fn init_with_size(&self, path: &String8, point_size: u32, resolution: u32) -> bool {
        let params = FontFTFontParams {
            point_size,
            resolution,
            ..Default::default()
        };
        self.init(&AsciiString::from_view(utf8_bytes(path)), &params, 0)
    }
    /// Deprecated `Init(theFontName, theFontAspect, thePointSize, theResolution)`: a font found
    /// by name at a size. Example: `font.init_by_name(&name, FontFontAspect::Regular, 12, 72)`.
    #[deprecated(
        note = "Deprecated method, Font_FTFont::Params should be used for passing parameters"
    )]
    pub fn init_by_name(
        &self,
        name: &String8,
        aspect: FontFontAspect,
        point_size: u32,
        resolution: u32,
    ) -> bool {
        let params = FontFTFontParams {
            point_size,
            resolution,
            ..Default::default()
        };
        let name = AsciiString::from_view(utf8_bytes(name));
        self.find_and_init(&name, aspect, &params, FontStrictLevel::Any)
    }
}

/// The UTF-8 bytes of a string without its terminator (`ToCString`).
fn utf8_bytes(text: &String8) -> &[u8] {
    let units = text.to_c_string();
    let end = units.iter().position(|&b| b == 0).unwrap_or(units.len());
    &units[..end]
}
