//! The metrics of `Font_FTFont`: glyph box sizes, ascender, descender, line spacing,
//! advances with kerning, glyph counts and the rendered glyph rectangle.
#![allow(unsafe_code)]
use super::freetype_ffi::{
    FT_FACE_FLAG_KERNING, FT_FACE_FLAG_SCALABLE, FT_KERNING_UNFITTED, FtFaceRec, FtLong,
    FtSizeMetrics, FtVector, freetype,
};
use super::ft_font::{FontFTFont, face_rec};
use super::ft_font_select::char_index;
use super::{FontRect, FontTextFormatter};
use crate::graphic3d::{Graphic3dHorizontalTextAlignment, Graphic3dVerticalTextAlignment};
use rustycad_tkernel::ncollection::String8;
use rustycad_tkernel::standard::Failure;

/// `fromFTPoints<float>`: 26.6 fixed point to float.
fn from_ft_points(units: FtLong) -> f32 {
    units as f32 / 64.0
}

/// The size metrics of a face with a size.
fn size_metrics(face: &FtFaceRec) -> Option<&FtSizeMetrics> {
    // SAFETY: the size of an open face is null or valid.
    unsafe { face.size.as_ref() }.map(|size| &size.metrics)
}

/// A font-unit value scaled to pixels by `ppem / units_per_EM`.
fn scaled(value: f32, ppem: u16, units_per_em: u16) -> f32 {
    value * (f32::from(ppem) / f32::from(units_per_em))
}

impl FontFTFont {
    /// The face with its size metrics.
    fn sized_face<R>(&self, read: impl FnOnce(&FtFaceRec, &FtSizeMetrics) -> R) -> Option<R> {
        let face = face_rec(self.state.read().face)?;
        Some(read(face, size_metrics(face)?))
    }
    /// `GlyphMaxSizeX`: the face box width in pixels (rounded), with the fallbacks' maximum
    /// when asked. Example: `font.glyph_max_size_x(false)`.
    pub fn glyph_max_size_x(&self, include_fallback: bool) -> u32 {
        let own = self.sized_face(|face, metrics| {
            let width = if face.face_flags & FT_FACE_FLAG_SCALABLE != 0 && face.units_per_em != 0 {
                scaled(
                    (face.bbox.x_max - face.bbox.x_min) as f32,
                    metrics.x_ppem,
                    face.units_per_em,
                )
            } else {
                from_ft_points(metrics.max_advance)
            };
            (width + 0.5) as u32
        });
        self.with_fallback_max(own, include_fallback, |f| f.glyph_max_size_x(false))
    }
    /// `GlyphMaxSizeY`: the face box height in pixels (rounded), with the fallbacks' maximum
    /// when asked. Example: `font.glyph_max_size_y(false)`.
    pub fn glyph_max_size_y(&self, include_fallback: bool) -> u32 {
        let own = self.sized_face(|face, metrics| {
            let height = if face.face_flags & FT_FACE_FLAG_SCALABLE != 0 && face.units_per_em != 0 {
                scaled(
                    (face.bbox.y_max - face.bbox.y_min) as f32,
                    metrics.y_ppem,
                    face.units_per_em,
                )
            } else {
                from_ft_points(metrics.height)
            };
            (height + 0.5) as u32
        });
        self.with_fallback_max(own, include_fallback, |f| f.glyph_max_size_y(false))
    }
    fn with_fallback_max(
        &self,
        own: Option<u32>,
        include_fallback: bool,
        size: impl Fn(&FontFTFont) -> u32,
    ) -> u32 {
        let Some(own) = own else { return 0 };
        if !include_fallback {
            return own;
        }
        let fallbacks = self.state.read().fallback_faces.clone();
        fallbacks
            .iter()
            .filter_map(|f| f.get())
            .filter(|f| f.is_valid())
            .fold(own, |acc, f| acc.max(size(f)))
    }
    /// A vertical metric: the face value scaled by `y_ppem`, or the size metric for a face
    /// without units per EM.
    fn vertical_metric(
        &self,
        face_value: impl Fn(&FtFaceRec) -> i16,
        size_value: impl Fn(&FtSizeMetrics) -> FtLong,
    ) -> f32 {
        self.sized_face(|face, metrics| {
            if face.units_per_em == 0 {
                return from_ft_points(size_value(metrics));
            }
            scaled(
                f32::from(face_value(face)),
                metrics.y_ppem,
                face.units_per_em,
            )
        })
        .unwrap_or(0.0)
    }
    /// `Ascender` in pixels; `font.ascender()`.
    pub fn ascender(&self) -> f32 {
        self.vertical_metric(|f| f.ascender, |m| m.ascender)
    }
    /// `Descender` in pixels (negative); `font.descender()`.
    pub fn descender(&self) -> f32 {
        self.vertical_metric(|f| f.descender, |m| m.descender)
    }
    /// `LineSpacing` in pixels; `font.line_spacing()`.
    pub fn line_spacing(&self) -> f32 {
        self.vertical_metric(|f| f.height, |m| m.height)
    }
}

impl FontFTFont {
    /// The face of the next character for kerning: this face or its fallback face.
    fn next_face(&self, next: u32) -> usize {
        let state = self.state.read();
        if char_index(state.face, next) != 0 {
            return state.face;
        }
        if !state.to_use_unicode_subset_fallback {
            return 0;
        }
        let fallback = &state.fallback_faces[Self::char_subset(next) as usize];
        let face = fallback.get().map_or(0, |f| f.state.read().face);
        if char_index(face, next) != 0 { face } else { 0 }
    }
    /// `getKerning`: the unfitted kerning between the loaded character and the next one when
    /// both glyphs come from the loaded face; zero otherwise.
    fn kerning(&self, current: u32, next: u32) -> FtVector {
        let loaded = self.state.read().loaded_glyph.face;
        let kerned = face_rec(loaded).is_some_and(|f| f.face_flags & FT_FACE_FLAG_KERNING != 0);
        if next == 0 || !kerned || self.next_face(next) != loaded {
            return FtVector::default();
        }
        let (left, right) = (char_index(loaded, current), char_index(loaded, next));
        let Some(api) = freetype().filter(|_| left != 0 && right != 0) else {
            return FtVector::default();
        };
        let mut kern = FtVector::default();
        // SAFETY: the loaded face is open; FT_Get_Kerning writes `kern`.
        let error = unsafe {
            (api.FT_Get_Kerning)(
                loaded as *mut FtFaceRec,
                left,
                right,
                FT_KERNING_UNFITTED,
                &mut kern,
            )
        };
        if error != 0 {
            FtVector::default()
        } else {
            kern
        }
    }
    /// The loaded slot's advance and side-bearing deltas, `None` without a loaded glyph.
    fn loaded_advance(&self) -> Option<(FtVector, FtLong, FtLong, u32)> {
        let loaded = self.state.read().loaded_glyph;
        if loaded.character == 0 {
            return None;
        }
        // SAFETY: the slot of the loaded face is null or valid.
        let slot = face_rec(loaded.face).and_then(|f| unsafe { f.glyph.as_ref() })?;
        Some((
            slot.advance,
            slot.lsb_delta,
            slot.rsb_delta,
            loaded.character,
        ))
    }
    /// `AdvanceX(next)`: the X advance of the loaded glyph with kerning and side-bearing
    /// deltas, times the width scaling. Example: `font.advance_x('B' as u32)`.
    pub fn advance_x(&self, next: u32) -> f32 {
        let Some((advance, lsb, rsb, current)) = self.loaded_advance() else {
            return 0.0;
        };
        let kern = self.kerning(current, next);
        self.width_scaling() * from_ft_points(advance.x + kern.x + lsb - rsb)
    }
    /// `AdvanceX(char, next)`: loads the glyph first; `font.advance_x_of('A' as u32, 'B' as u32)`.
    pub fn advance_x_of(&self, character: u32, next: u32) -> f32 {
        self.load_glyph(character);
        self.advance_x(next)
    }
    /// `AdvanceY(next)`: the Y advance of the loaded glyph with kerning.
    /// Example: `font.advance_y('B' as u32)`.
    pub fn advance_y(&self, next: u32) -> f32 {
        let Some((advance, _, _, current)) = self.loaded_advance() else {
            return 0.0;
        };
        let kern = self.kerning(current, next);
        from_ft_points(advance.y + kern.y)
    }
    /// `AdvanceY(char, next)`: loads the glyph first; `font.advance_y_of('A' as u32, 'B' as u32)`.
    pub fn advance_y_of(&self, character: u32, next: u32) -> f32 {
        self.load_glyph(character);
        self.advance_y(next)
    }
    /// `GlyphsNumber`: the glyphs of the face, plus the valid fallbacks' when asked.
    /// Example: `font.glyphs_number(false)`.
    pub fn glyphs_number(&self, include_fallback: bool) -> i32 {
        let Some(face) = face_rec(self.state.read().face) else {
            return 0;
        };
        // FreeType uses C long, which is 32-bit on Windows and 64-bit on LP64 hosts.
        #[allow(clippy::unnecessary_cast)]
        let own = face.num_glyphs as i32;
        if !include_fallback {
            return own;
        }
        let fallbacks = self.state.read().fallback_faces.clone();
        fallbacks
            .iter()
            .filter_map(|f| f.get())
            .filter(|f| f.is_valid())
            .fold(own, |acc, f| acc + f.glyphs_number(false))
    }
    /// `BoundingBox`: the box of a string formatted with this font and the alignments.
    /// Example: `font.bounding_box(&text, Graphic3dHorizontalTextAlignment::Left, Graphic3dVerticalTextAlignment::Bottom)?`.
    pub fn bounding_box(
        &self,
        text: &String8,
        align_x: Graphic3dHorizontalTextAlignment,
        align_y: Graphic3dVerticalTextAlignment,
    ) -> Result<FontRect, Failure> {
        let formatter = FontTextFormatter::default();
        formatter.setup_alignment(align_x, align_y);
        formatter.reset();
        formatter.append(text, self)?;
        formatter.format()?;
        let mut bounds = FontRect::default();
        formatter.bnd_box(&mut bounds);
        Ok(bounds)
    }
    /// `GlyphRect`: the pixel rectangle of the rendered glyph (zero without a bitmap).
    /// Example: `font.glyph_rect(&mut rect)`.
    pub fn glyph_rect(&self, rect: &mut FontRect) {
        (rect.left, rect.right, rect.top, rect.bottom) = (0.0, 0.0, 0.0, 0.0);
        let loaded = self.state.read().loaded_glyph;
        if !loaded.has_bitmap {
            return;
        }
        // SAFETY: the slot of the loaded face is null or valid.
        let Some(slot) = face_rec(loaded.face).and_then(|f| unsafe { f.glyph.as_ref() }) else {
            return;
        };
        rect.left = slot.bitmap_left as f32;
        rect.top = slot.bitmap_top as f32;
        rect.right = (slot.bitmap_left + slot.bitmap.width as i32) as f32;
        rect.bottom = (slot.bitmap_top - slot.bitmap.rows as i32) as f32;
    }
}
