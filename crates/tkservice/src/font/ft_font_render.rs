//! Glyph loading and rendering of `Font_FTFont`: the loaded slot, the glyph bitmap and the
//! glyph-dependent options (width scaling, single stroke, subset fallback).
#![allow(unsafe_code)]
use super::freetype_ffi::{
    FT_GLYPH_FORMAT_BITMAP, FT_LOAD_RENDER, FT_PIXEL_MODE_GRAY, FT_PIXEL_MODE_MONO, FtBitmap,
    FtFaceRec, freetype,
};
use super::ft_font::{FontFTFont, LoadedGlyph, face_rec};
use super::ft_font_select::GlyphSelection;
use crate::image::ImageFormat;

impl GlyphSelection {
    /// The `FT_Face` address and load flags of the font that provides the glyph.
    fn face_and_flags(&self, this: &FontFTFont) -> (usize, i32) {
        let font = self.provider.as_ref().and_then(|p| p.get()).unwrap_or(this);
        let state = font.state.read();
        (state.face, state.load_flags)
    }
}

/// `FT_Load_Glyph` with extra flags; whether the slot holds the glyph.
fn load_glyph_slot(face: usize, index: u32, flags: i32) -> bool {
    let Some(api) = freetype() else { return false };
    // SAFETY: an open face; FT_Load_Glyph fills its glyph slot.
    let error = unsafe { (api.FT_Load_Glyph)(face as *mut FtFaceRec, index, flags) };
    error == 0 && face_rec(face).is_some_and(|f| !f.glyph.is_null())
}

impl FontFTFont {
    /// `loadGlyph`: loads the character's glyph slot (kept while the character is the same).
    pub(crate) fn load_glyph(&self, character: u32) -> bool {
        {
            let state = self.state.read();
            if state.loaded_glyph.character == character {
                return state.loaded_glyph.face != 0;
            }
        }
        self.glyph_img.clear();
        self.state.write().loaded_glyph = LoadedGlyph::default();
        let Some(selection) = self.select_glyph(character) else {
            return false;
        };
        let (face, flags) = selection.face_and_flags(self);
        if !load_glyph_slot(face, selection.index, flags) {
            return false;
        }
        self.state.write().loaded_glyph = LoadedGlyph {
            face,
            character,
            has_bitmap: false,
        };
        true
    }
    /// `RenderGlyph`: renders the character's glyph into the glyph image (gray bitmaps as
    /// alpha, mono bitmaps expanded to 0/255 gray); false when nothing can be rendered.
    /// Example: `font.render_glyph('A' as u32)`.
    pub fn render_glyph(&self, character: u32) -> bool {
        self.glyph_img.clear();
        self.state.write().loaded_glyph = LoadedGlyph::default();
        let Some(selection) = self.select_glyph(character) else {
            return false;
        };
        let (face, flags) = selection.face_and_flags(self);
        if !load_glyph_slot(face, selection.index, flags | FT_LOAD_RENDER) {
            return false;
        }
        // SAFETY: the slot of the open face was just loaded.
        let Some(slot) = face_rec(face).and_then(|f| unsafe { f.glyph.as_ref() }) else {
            return false;
        };
        if slot.format != FT_GLYPH_FORMAT_BITMAP || !self.store_bitmap(&slot.bitmap) {
            return false;
        }
        self.state.write().loaded_glyph = LoadedGlyph {
            face,
            character,
            has_bitmap: true,
        };
        true
    }
    /// Copies a FreeType bitmap into the glyph image.
    fn store_bitmap(&self, bitmap: &FtBitmap) -> bool {
        if bitmap.buffer.is_null() || bitmap.width == 0 || bitmap.rows == 0 {
            return false;
        }
        let (width, rows) = (bitmap.width as usize, bitmap.rows as usize);
        let pitch = bitmap.pitch.unsigned_abs() as usize;
        // SAFETY: FreeType's bitmap holds `rows * |pitch|` bytes.
        let bytes = unsafe { std::slice::from_raw_parts(bitmap.buffer, rows * pitch) };
        let stored = match bitmap.pixel_mode {
            FT_PIXEL_MODE_GRAY => self.glyph_img.init_wrapper(
                ImageFormat::Alpha,
                Some(bytes.to_vec()),
                width,
                rows,
                pitch,
            ),
            FT_PIXEL_MODE_MONO => self.glyph_img.init_wrapper(
                ImageFormat::Gray,
                Some(expand_mono(bytes, width, rows)),
                width,
                rows,
                width,
            ),
            _ => return false,
        };
        if stored {
            self.glyph_img.set_top_down(bitmap.pitch > 0);
        }
        stored
    }
}

/// The 0/255 gray bytes of a 1-bit bitmap (rows of `ceil(width / 8)` bytes).
fn expand_mono(bytes: &[u8], width: usize, rows: usize) -> Vec<u8> {
    let row_bytes = width.div_ceil(8);
    let mut gray = vec![0u8; width * rows];
    for row in 0..rows {
        for col in 0..width {
            let bit = bytes[row_bytes * row + col / 8] & (0x80 >> (col % 8));
            gray[row * width + col] = if bit != 0 { 255 } else { 0 };
        }
    }
    gray
}

impl FontFTFont {
    /// Whether missing characters use the subset fallback faces; `font.to_use_unicode_subset_fallback()`.
    pub fn to_use_unicode_subset_fallback(&self) -> bool {
        self.state.read().to_use_unicode_subset_fallback
    }
    /// Sets the subset fallback (a change invalidates the glyph state).
    /// Example: `font.set_use_unicode_subset_fallback(false)`.
    pub fn set_use_unicode_subset_fallback(&self, to_fallback: bool) {
        let mut state = self.state.write();
        if state.to_use_unicode_subset_fallback == to_fallback {
            return;
        }
        state.to_use_unicode_subset_fallback = to_fallback;
        self.invalidate_glyph_state(&mut state);
    }
    /// Whether the outlines are single strokes; `font.is_single_stroke_font()`.
    pub fn is_single_stroke_font(&self) -> bool {
        self.state.read().params.is_single_stroke_font
    }
    /// Sets the single-stroke flag (a change drops the fallbacks and invalidates the glyphs).
    /// Example: `font.set_single_stroke_font(true)`.
    pub fn set_single_stroke_font(&self, is_single_line: bool) {
        let mut state = self.state.write();
        if state.params.is_single_stroke_font == is_single_line {
            return;
        }
        state.fallback_faces = Default::default();
        state.params.is_single_stroke_font = is_single_line;
        self.invalidate_glyph_state(&mut state);
    }
    /// Whether italic is synthesized; `font.to_synthesize_italic()`.
    pub fn to_synthesize_italic(&self) -> bool {
        self.state.read().params.to_synthesize_italic
    }
    /// The face size in points; `font.point_size()`.
    pub fn point_size(&self) -> u32 {
        self.state.read().params.point_size
    }
    /// The X scaling of the glyphs; `font.width_scaling()`.
    pub fn width_scaling(&self) -> f32 {
        self.state.read().width_scaling
    }
    /// Sets a finite positive X scaling (a change invalidates the glyph state); false for
    /// another value. Example: `font.set_width_scaling(0.5)`.
    pub fn set_width_scaling(&self, factor: f32) -> bool {
        if !factor.is_finite() || factor <= 0.0 {
            return false;
        }
        let mut state = self.state.write();
        if state.width_scaling == factor {
            return true;
        }
        state.width_scaling = factor;
        self.invalidate_glyph_state(&mut state);
        true
    }
}
