//! `checkFont` of `Font_FontMgr`: opens a font file with FreeType and describes each face
//! (and variable-font instance) as a `Font_SystemFont`.
#![allow(unsafe_code)]
use super::freetype_ffi::{
    FT_ENCODING_UNICODE, FT_STYLE_FLAG_BOLD, FT_STYLE_FLAG_ITALIC, FtFaceRec, FtLong, freetype,
};
use super::{FontFTLibrary, FontFontAspect, FontSystemFont};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
use std::ffi::{CStr, CString, c_char};

/// An open FreeType face, released on drop.
struct FaceOwner(*mut FtFaceRec);

impl Drop for FaceOwner {
    fn drop(&mut self) {
        if let Some(api) = freetype() {
            // SAFETY: the face came from FT_New_Face and is released once, here.
            unsafe { (api.FT_Done_Face)(self.0) };
        }
    }
}

/// The C string of a FreeType name field, empty for null.
fn c_text(text: *const c_char) -> Option<AsciiString> {
    // SAFETY: FreeType keeps its name fields NUL-terminated while the face is open.
    (!text.is_null()).then(|| AsciiString::from_view(unsafe { CStr::from_ptr(text) }.to_bytes()))
}

/// `FT_New_Face` of a file face, `None` on failure.
fn open_face(library: &FontFTLibrary, path: &AsciiString, face_id: FtLong) -> Option<FaceOwner> {
    let api = freetype()?;
    let name = CString::new(path.as_bytes()).ok()?;
    let mut face: *mut FtFaceRec = std::ptr::null_mut();
    // SAFETY: a valid library handle, a NUL-terminated path and an output pointer.
    let error = unsafe { (api.FT_New_Face)(library.instance(), name.as_ptr(), face_id, &mut face) };
    (error == 0 && !face.is_null()).then_some(FaceOwner(face))
}

/// Removes the first `word` of `style` (its length is `word.len()`).
fn remove_word(style: &mut AsciiString, word: &str) -> Result<bool, Failure> {
    let at = style.search(word);
    if at == -1 {
        return Ok(false);
    }
    style.remove(at, word.len() as i32)?;
    Ok(true)
}

/// Removes `Italic`, else `Oblique`.
fn remove_italic(style: &mut AsciiString) -> Result<(), Failure> {
    if !remove_word(style, "Italic")? {
        remove_word(style, "Oblique")?;
    }
    Ok(())
}

/// The aspect of the style flags, with the style words that it implies removed.
fn aspect_of(style_flags: FtLong, style: &mut AsciiString) -> Result<FontFontAspect, Failure> {
    let both = FT_STYLE_FLAG_ITALIC | FT_STYLE_FLAG_BOLD;
    if style_flags == both {
        remove_italic(style)?;
        remove_word(style, "Bold")?;
        return Ok(FontFontAspect::BoldItalic);
    }
    if style_flags == FT_STYLE_FLAG_ITALIC {
        remove_italic(style)?;
        return Ok(FontFontAspect::Italic);
    }
    if style_flags == FT_STYLE_FLAG_BOLD {
        remove_word(style, "Bold")?;
        return Ok(FontFontAspect::Bold);
    }
    Ok(FontFontAspect::Regular)
}

/// The family name with the remaining style words (`Regular`/`Book` dropped, spaces folded).
fn family_with_style(family: AsciiString, mut style: AsciiString) -> Result<AsciiString, Failure> {
    if !remove_word(&mut style, "Regular")? {
        remove_word(&mut style, "Book")?;
    }
    style.left_adjust();
    style.right_adjust();
    while style.search("  ") != -1 {
        let at = style.search("  ");
        style.remove(at, 1)?;
    }
    if style.is_empty() {
        return Ok(family);
    }
    Ok(AsciiString::from_view(
        &[family.as_bytes(), b" ", style.as_bytes()].concat(),
    ))
}

/// `checkFont`: appends the system font of each face of `path` (a face id of -1 also visits
/// the other faces); false when the file is not a Unicode font FreeType can open.
/// Example: `check_font(&mut fonts, &library, &path, -1)?`.
pub(crate) fn check_font(
    fonts: &mut Vec<Handle<FontSystemFont>>,
    library: &FontFTLibrary,
    path: &AsciiString,
    face_id: FtLong,
) -> Result<bool, Failure> {
    let opened_id = if face_id != -1 { face_id } else { 0 };
    let Some(face) = open_face(library, path, opened_id) else {
        return Ok(false);
    };
    // SAFETY: the face is open for the rest of this function.
    let record = unsafe { &*face.0 };
    let Some(family) = c_text(record.family_name) else {
        return Ok(false);
    };
    let api = freetype().ok_or_else(|| {
        Failure::new("FreeType is not loaded; expected the library of the open face")
    })?;
    // SAFETY: an open face and the Unicode encoding tag.
    if unsafe { (api.FT_Select_Charmap)(face.0, FT_ENCODING_UNICODE) } != 0 {
        return Ok(false);
    }
    let mut style = c_text(record.style_name).unwrap_or_default();
    let aspect = aspect_of(record.style_flags, &mut style)?;
    let font = FontSystemFont::new(&family_with_style(family, style)?)?;
    // FreeType uses C long, which is 32-bit on Windows and 64-bit on LP64 hosts.
    #[allow(clippy::unnecessary_cast)]
    font.set_font_path(aspect, path, opened_id as i32)?;
    font.set_single_stroke_font(font.font_key().starts_with("olf "));
    fonts.push(Handle::new(font));
    visit_sub_faces(
        fonts,
        library,
        path,
        face_id,
        (record.num_faces, record.style_flags),
    )?;
    Ok(true)
}

/// The variable-font instances of the face, then (for a face id of -1) the other faces.
fn visit_sub_faces(
    fonts: &mut Vec<Handle<FontSystemFont>>,
    library: &FontFTLibrary,
    path: &AsciiString,
    face_id: FtLong,
    (num_faces, style_flags): (FtLong, FtLong),
) -> Result<(), Failure> {
    let opened_id = if face_id != -1 { face_id } else { 0 };
    if face_id < num_faces {
        for instance in 1..(style_flags >> 16) {
            check_font(fonts, library, path, opened_id + (instance << 16))?;
        }
    }
    if face_id == -1 {
        for other in 1..num_faces {
            check_font(fonts, library, path, other)?;
        }
    }
    Ok(())
}
