//! `Font_FTFont`: a FreeType face at one size, with Unicode-subset fallback faces, glyph
//! bitmaps, outlines and metrics.
//!
//! The FreeType face is a raw handle owned by the font (released by `Release` or drop); the
//! font state is behind the handle's shared lock, so every member takes `&self`.
#![allow(unsafe_code)]
use super::freetype_ffi::{
    FT_ENCODING_UNICODE, FT_LOAD_FORCE_AUTOHINT, FT_LOAD_NO_AUTOHINT, FT_LOAD_NO_HINTING,
    FT_LOAD_TARGET_LIGHT, FT_LOAD_TARGET_NORMAL, FtFaceRec, FtLong, FtMatrix, freetype,
};
use super::glyph_outline::FontGlyphOutlineGlyphIdentity;
use super::{FontFTLibrary, FontFontAspect, FontFontMgr, FontHinting, FontUnicodeSubset};
use crate::image::ImagePixMap;
use rustycad_tkernel::message::Message;
use rustycad_tkernel::ncollection::Buffer;
use rustycad_tkernel::standard::{Handle, SharedState, StandardTransient};
use rustycad_tkernel::tcollection::AsciiString;
use std::ffi::CString;
use std::sync::Arc;

/// `Font_FTFont::Params`: the face size, resolution, hinting and style options.
/// Example: `FontFTFontParams { point_size: 12, ..Default::default() }.is_valid()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontFTFontParams {
    /// `PointSize`: the face size in points (1/72 inch).
    pub point_size: u32,
    /// `Resolution`: the device resolution in dpi.
    pub resolution: u32,
    /// `FontHinting`: the FreeType hinting policy.
    pub font_hinting: FontHinting,
    /// `ToSynthesizeItalic`: shear the face when it has no italic style.
    pub to_synthesize_italic: bool,
    /// `IsSingleStrokeFont`: read the outlines as one-line strokes.
    pub is_single_stroke_font: bool,
}

impl Default for FontFTFontParams {
    fn default() -> Self {
        Self {
            point_size: 0,
            resolution: 72,
            font_hinting: FontHinting::OFF,
            to_synthesize_italic: false,
            is_single_stroke_font: false,
        }
    }
}

impl FontFTFontParams {
    /// `IsValid`: positive size and resolution, no conflicting hinting flags.
    pub fn is_valid(&self) -> bool {
        let has = |flag: FontHinting| self.font_hinting.0 & flag.0 != 0;
        let conflicting = has(FontHinting::LIGHT) && has(FontHinting::NORMAL);
        let conflicting_auto = has(FontHinting::FORCE_AUTOHINT) && has(FontHinting::NO_AUTOHINT);
        self.point_size > 0 && self.resolution > 0 && !conflicting && !conflicting_auto
    }
}

/// `LoadedGlyph`: the FreeType slot currently loaded and what it exposes.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LoadedGlyph {
    /// The `FT_Face` address of the loaded slot, 0 for none.
    pub(crate) face: usize,
    pub(crate) character: u32,
    pub(crate) has_bitmap: bool,
}

/// The font fields, shared by every handle.
pub(crate) struct FtFontState {
    pub(crate) ft_lib: Handle<FontFTLibrary>,
    pub(crate) buffer: Handle<Buffer>,
    pub(crate) fallback_faces: [Handle<FontFTFont>; FontUnicodeSubset::Nb as usize],
    /// The `FT_Face` address, 0 for none.
    pub(crate) face: usize,
    pub(crate) font_path: AsciiString,
    pub(crate) params: FontFTFontParams,
    pub(crate) font_aspect: FontFontAspect,
    pub(crate) width_scaling: f32,
    pub(crate) load_flags: i32,
    pub(crate) identity: Arc<FontGlyphOutlineGlyphIdentity>,
    pub(crate) revision: u64,
    pub(crate) loaded_glyph: LoadedGlyph,
    pub(crate) to_use_unicode_subset_fallback: bool,
}

/// A FreeType font; `FontFTFont::find_and_create(&"sans-serif".into(), aspect, &params, strict)`.
pub struct FontFTFont {
    transient: StandardTransient,
    pub(crate) state: SharedState<FtFontState>,
    pub(crate) glyph_img: Handle<ImagePixMap>,
}
rustycad_tkernel::impl_transient!(FontFTFont, "Font_FTFont", StandardTransient, transient);

/// The `FT_FaceRec` behind an address.
pub(crate) fn face_rec<'a>(face: usize) -> Option<&'a FtFaceRec> {
    // SAFETY: a non-zero address is an `FT_Face` this crate opened and has not released;
    // the reference does not outlive the font state lock that keeps it alive.
    (face != 0).then(|| unsafe { &*(face as *const FtFaceRec) })
}

/// `FT_Done_Face` of an owned face address.
fn done_face(face: usize) {
    if let (true, Some(api)) = (face != 0, freetype()) {
        // SAFETY: the face was opened by this crate and is released once, here.
        unsafe { (api.FT_Done_Face)(face as *mut FtFaceRec) };
    }
}

impl Default for FontFTFont {
    fn default() -> Self {
        Self::new(Handle::default())
    }
}

impl FontFTFont {
    /// A font over a FreeType library (a new one when null); `FontFTFont::new(Handle::default())`.
    pub fn new(ft_lib: Handle<FontFTLibrary>) -> Self {
        let ft_lib = if ft_lib.is_null() {
            Handle::new(FontFTLibrary::new())
        } else {
            ft_lib
        };
        let state = FtFontState {
            ft_lib,
            buffer: Handle::default(),
            fallback_faces: Default::default(),
            face: 0,
            font_path: AsciiString::default(),
            params: FontFTFontParams::default(),
            font_aspect: FontFontAspect::Regular,
            width_scaling: 1.0,
            load_flags: FT_LOAD_NO_HINTING | FT_LOAD_TARGET_NORMAL,
            identity: Arc::new(FontGlyphOutlineGlyphIdentity),
            revision: 1,
            loaded_glyph: LoadedGlyph::default(),
            to_use_unicode_subset_fallback: FontFontMgr::to_use_unicode_subset_fallback(),
        };
        Self {
            transient: StandardTransient::default(),
            state: SharedState::new(state),
            glyph_img: Handle::new(ImagePixMap::default()),
        }
    }
    /// Whether a face is open; `font.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.state.read().face != 0
    }
    /// The bitmap of the last rendered glyph; `font.glyph_image()`.
    pub fn glyph_image(&self) -> &Handle<ImagePixMap> {
        &self.glyph_img
    }
    /// `invalidateGlyphState`: a new geometry revision and no loaded glyph.
    pub(crate) fn invalidate_glyph_state(&self, state: &mut FtFontState) {
        state.revision = state.revision.wrapping_add(1);
        self.glyph_img.clear();
        state.loaded_glyph = LoadedGlyph::default();
    }
    /// `Release`: closes the face and forgets the fallbacks, path and memory buffer.
    /// Example: `font.release()`.
    pub fn release(&self) {
        let mut state = self.state.write();
        self.invalidate_glyph_state(&mut state);
        state.fallback_faces = Default::default();
        state.font_path.clear();
        done_face(std::mem::replace(&mut state.face, 0));
        state.buffer = Handle::default();
    }
}

impl Drop for FontFTFont {
    fn drop(&mut self) {
        let face = std::mem::replace(&mut self.state.write().face, 0);
        done_face(face);
    }
}

/// The FreeType load flags of a hinting policy.
fn load_flags(hinting: FontHinting) -> i32 {
    let has = |flag: FontHinting| hinting.0 & flag.0 != 0;
    let mut flags = FT_LOAD_TARGET_NORMAL;
    if has(FontHinting::LIGHT) {
        flags = FT_LOAD_TARGET_LIGHT;
    } else if !has(FontHinting::NORMAL) {
        flags |= FT_LOAD_NO_HINTING;
    }
    if has(FontHinting::FORCE_AUTOHINT) {
        flags |= FT_LOAD_FORCE_AUTOHINT;
    }
    if has(FontHinting::NO_AUTOHINT) {
        flags |= FT_LOAD_NO_AUTOHINT;
    }
    flags
}

fn trace(parts: &[&[u8]]) {
    Message::send_trace_text(&AsciiString::from_view(&parts.concat()));
}

/// An owned new face, released on drop unless taken.
struct NewFace(usize);
impl NewFace {
    fn take(mut self) -> usize {
        std::mem::replace(&mut self.0, 0)
    }
}
impl Drop for NewFace {
    fn drop(&mut self) {
        done_face(self.0);
    }
}

/// `FT_New_Memory_Face` or `FT_New_Face`.
fn open_face(
    library: &FontFTLibrary,
    data: &Handle<Buffer>,
    path: &AsciiString,
    face_id: i32,
) -> Option<NewFace> {
    let api = freetype()?;
    let mut face: *mut FtFaceRec = std::ptr::null_mut();
    let error = match data.get() {
        Some(buffer) => {
            let size = buffer.size() as FtLong;
            // SAFETY: the buffer outlives the face (the font keeps it as `myBuffer`).
            unsafe {
                (api.FT_New_Memory_Face)(
                    library.instance(),
                    buffer.data_ptr(),
                    size,
                    face_id as FtLong,
                    &mut face,
                )
            }
        }
        None => {
            let name = CString::new(path.as_bytes()).ok()?;
            // SAFETY: a valid library and a NUL-terminated path.
            unsafe {
                (api.FT_New_Face)(
                    library.instance(),
                    name.as_ptr(),
                    face_id as FtLong,
                    &mut face,
                )
            }
        }
    };
    if error != 0 || face.is_null() {
        if data.is_some() {
            trace(&[b"Font '", path.as_bytes(), b"' failed to load from memory"]);
        }
        return None;
    }
    Some(NewFace(face as usize))
}

/// The charmap, size and italic shear of a new face; false (with a trace) on failure.
fn configure_face(face: usize, path: &AsciiString, params: &FontFTFontParams) -> bool {
    let Some(api) = freetype() else { return false };
    let raw = face as *mut FtFaceRec;
    // SAFETY: an open face and the Unicode encoding tag.
    if unsafe { (api.FT_Select_Charmap)(raw, FT_ENCODING_UNICODE) } != 0 {
        trace(&[
            b"Font '",
            path.as_bytes(),
            b"' doesn't contains Unicode charmap",
        ]);
        return false;
    }
    let point_size = FtLong::from(params.point_size) * 64;
    // SAFETY: an open face; FT_Set_Char_Size only reads the arguments.
    let error =
        unsafe { (api.FT_Set_Char_Size)(raw, 0, point_size, params.resolution, params.resolution) };
    if error != 0 {
        trace(&[
            b"Font '",
            path.as_bytes(),
            b"' failed to set requested character size",
        ]);
        return false;
    }
    if params.to_synthesize_italic {
        shear_italic(face);
    }
    true
}

/// The 10-degree italic shear transform of `ToSynthesizeItalic`.
fn shear_italic(face: usize) {
    let Some(api) = freetype() else { return };
    let angle = 10.0 * std::f64::consts::PI / 180.0;
    let xx = ((-angle).cos() * f64::from(1 << 16)) as FtLong;
    let factor = (angle.tan() * f64::from(1 << 16)) as FtLong;
    // SAFETY: FT_MulFix is a pure arithmetic call.
    let xy = unsafe { (api.FT_MulFix)(factor, xx) };
    let mut matrix = FtMatrix {
        xx,
        xy,
        yx: 0,
        yy: xx,
    };
    // SAFETY: an open face; the matrix is copied by FreeType.
    unsafe { (api.FT_Set_Transform)(face as *mut FtFaceRec, &mut matrix, std::ptr::null_mut()) };
}

impl FontFTFont {
    /// `Init(theFontPath, theParams, theFaceId)`: opens a font file; `font.init(&path, &params, 0)`.
    pub fn init(&self, path: &AsciiString, params: &FontFTFontParams, face_id: i32) -> bool {
        self.init_from(&Handle::default(), path, params, face_id)
    }
    /// `Init(theData, theFileName, theParams, theFaceId)`: opens a font from memory (or the
    /// file when `data` is null); invalid parameters or library fail before anything changes.
    /// Example: `font.init_from(&data, &"embedded".into(), &params, 0)`.
    pub fn init_from(
        &self,
        data: &Handle<Buffer>,
        path: &AsciiString,
        params: &FontFTFontParams,
        face_id: i32,
    ) -> bool {
        let library = self.state.read().ft_lib.clone();
        let library_valid = library.get().is_some_and(FontFTLibrary::is_valid);
        if !params.is_valid() || !library_valid {
            if !library_valid {
                trace(&[b"FreeType library is unavailable"]);
            }
            return false;
        }
        let Some(library) = library.get() else {
            return false;
        };
        let Some(face) = open_face(library, data, path, face_id) else {
            return false;
        };
        if !configure_face(face.0, path, params) {
            return false;
        }
        self.release();
        let mut state = self.state.write();
        (state.buffer, state.font_path, state.params) = (data.clone(), path.clone(), *params);
        state.load_flags = load_flags(params.font_hinting);
        state.face = face.take();
        true
    }
}
