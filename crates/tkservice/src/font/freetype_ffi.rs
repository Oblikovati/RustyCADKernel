//! The minimal FreeType binding of the Font package (visualization-design.md section 2):
//! `libfreetype.so.6` is loaded at run time and only the entry points that `Font_FTLibrary`,
//! `Font_FTFont` and `Font_FontMgr` call are bound. The record layouts mirror the public
//! FreeType 2 headers (`freetype.h`, `ftimage.h`); only their leading, public fields are read.
#![allow(unsafe_code, non_snake_case, non_camel_case_types, dead_code)]
use crate::dynlib::{DynamicLibrary, bind_symbols};
use std::ffi::{c_char, c_int, c_long, c_short, c_uint, c_ulong, c_ushort, c_void};
use std::sync::OnceLock;

/// `FT_Error`.
pub(crate) type FtError = c_int;
/// `FT_Pos`, `FT_Fixed`, `FT_F26Dot6`, `FT_Long`.
pub(crate) type FtLong = c_long;

/// `FT_Err_Ok`.
pub(crate) const FT_ERR_OK: FtError = 0x00;
/// `FT_Err_Invalid_Argument`.
pub(crate) const FT_ERR_INVALID_ARGUMENT: FtError = 0x06;
/// `FT_Err_Array_Too_Large`.
pub(crate) const FT_ERR_ARRAY_TOO_LARGE: FtError = 0x0A;
/// `FT_Err_Invalid_Outline`.
pub(crate) const FT_ERR_INVALID_OUTLINE: FtError = 0x14;

/// `FT_LOAD_NO_SCALE`.
pub(crate) const FT_LOAD_NO_SCALE: i32 = 1 << 0;
/// `FT_LOAD_NO_HINTING`.
pub(crate) const FT_LOAD_NO_HINTING: i32 = 1 << 1;
/// `FT_LOAD_RENDER`.
pub(crate) const FT_LOAD_RENDER: i32 = 1 << 2;
/// `FT_LOAD_NO_BITMAP`.
pub(crate) const FT_LOAD_NO_BITMAP: i32 = 1 << 3;
/// `FT_LOAD_FORCE_AUTOHINT`.
pub(crate) const FT_LOAD_FORCE_AUTOHINT: i32 = 1 << 5;
/// `FT_LOAD_NO_AUTOHINT`.
pub(crate) const FT_LOAD_NO_AUTOHINT: i32 = 1 << 15;
/// `FT_LOAD_TARGET_NORMAL` (`FT_RENDER_MODE_NORMAL << 16`).
pub(crate) const FT_LOAD_TARGET_NORMAL: i32 = 0;
/// `FT_LOAD_TARGET_LIGHT` (`FT_RENDER_MODE_LIGHT << 16`).
pub(crate) const FT_LOAD_TARGET_LIGHT: i32 = 1 << 16;

/// `FT_FACE_FLAG_SCALABLE`.
pub(crate) const FT_FACE_FLAG_SCALABLE: FtLong = 1 << 0;
/// `FT_FACE_FLAG_KERNING`.
pub(crate) const FT_FACE_FLAG_KERNING: FtLong = 1 << 6;
/// `FT_FACE_FLAG_TRICKY`.
pub(crate) const FT_FACE_FLAG_TRICKY: FtLong = 1 << 13;
/// `FT_STYLE_FLAG_ITALIC`.
pub(crate) const FT_STYLE_FLAG_ITALIC: FtLong = 1 << 0;
/// `FT_STYLE_FLAG_BOLD`.
pub(crate) const FT_STYLE_FLAG_BOLD: FtLong = 1 << 1;

/// The four-character tag of `FT_IMAGE_TAG` and `FT_ENC_TAG`.
const fn tag(bytes: &[u8; 4]) -> u32 {
    ((bytes[0] as u32) << 24)
        | ((bytes[1] as u32) << 16)
        | ((bytes[2] as u32) << 8)
        | bytes[3] as u32
}
/// `FT_GLYPH_FORMAT_BITMAP`.
pub(crate) const FT_GLYPH_FORMAT_BITMAP: u32 = tag(b"bits");
/// `FT_GLYPH_FORMAT_OUTLINE`.
pub(crate) const FT_GLYPH_FORMAT_OUTLINE: u32 = tag(b"outl");
/// `FT_ENCODING_UNICODE` (`ft_encoding_unicode`).
pub(crate) const FT_ENCODING_UNICODE: u32 = tag(b"unic");
/// `FT_PIXEL_MODE_MONO`.
pub(crate) const FT_PIXEL_MODE_MONO: u8 = 1;
/// `FT_PIXEL_MODE_GRAY`.
pub(crate) const FT_PIXEL_MODE_GRAY: u8 = 2;
/// `FT_OUTLINE_EVEN_ODD_FILL`.
pub(crate) const FT_OUTLINE_EVEN_ODD_FILL: c_int = 0x2;
/// `FT_OUTLINE_OVERLAP`.
pub(crate) const FT_OUTLINE_OVERLAP: c_int = 0x40;
/// `FT_KERNING_UNFITTED`.
pub(crate) const FT_KERNING_UNFITTED: c_uint = 1;

/// `FT_Vector`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FtVector {
    pub(crate) x: FtLong,
    pub(crate) y: FtLong,
}
/// `FT_Matrix`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FtMatrix {
    pub(crate) xx: FtLong,
    pub(crate) xy: FtLong,
    pub(crate) yx: FtLong,
    pub(crate) yy: FtLong,
}
/// `FT_BBox`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct FtBBox {
    pub(crate) x_min: FtLong,
    pub(crate) y_min: FtLong,
    pub(crate) x_max: FtLong,
    pub(crate) y_max: FtLong,
}
/// `FT_Generic`.
#[repr(C)]
pub(crate) struct FtGeneric {
    data: *mut c_void,
    finalizer: *mut c_void,
}
/// `FT_Bitmap`.
#[repr(C)]
pub(crate) struct FtBitmap {
    pub(crate) rows: c_uint,
    pub(crate) width: c_uint,
    pub(crate) pitch: c_int,
    pub(crate) buffer: *mut u8,
    pub(crate) num_grays: c_ushort,
    pub(crate) pixel_mode: u8,
    pub(crate) palette_mode: u8,
    pub(crate) palette: *mut c_void,
}
/// `FT_Outline`.
#[repr(C)]
pub(crate) struct FtOutline {
    pub(crate) n_contours: c_short,
    pub(crate) n_points: c_short,
    pub(crate) points: *mut FtVector,
    pub(crate) tags: *mut c_char,
    pub(crate) contours: *mut c_short,
    pub(crate) flags: c_int,
}
/// `FT_Glyph_Metrics`.
#[repr(C)]
pub(crate) struct FtGlyphMetrics {
    values: [FtLong; 8],
}
/// `FT_GlyphSlotRec` (public part).
#[repr(C)]
pub(crate) struct FtGlyphSlotRec {
    library: *mut c_void,
    face: *mut c_void,
    next: *mut c_void,
    /// `glyph_index` (FreeType 2.10 and later; reserved before, same layout).
    glyph_index: c_uint,
    generic: FtGeneric,
    metrics: FtGlyphMetrics,
    linear_hori_advance: FtLong,
    linear_vert_advance: FtLong,
    pub(crate) advance: FtVector,
    pub(crate) format: u32,
    pub(crate) bitmap: FtBitmap,
    pub(crate) bitmap_left: c_int,
    pub(crate) bitmap_top: c_int,
    pub(crate) outline: FtOutline,
    num_subglyphs: c_uint,
    subglyphs: *mut c_void,
    control_data: *mut c_void,
    control_len: c_long,
    pub(crate) lsb_delta: FtLong,
    pub(crate) rsb_delta: FtLong,
}
/// `FT_Size_Metrics`.
#[repr(C)]
pub(crate) struct FtSizeMetrics {
    pub(crate) x_ppem: c_ushort,
    pub(crate) y_ppem: c_ushort,
    pub(crate) x_scale: FtLong,
    pub(crate) y_scale: FtLong,
    pub(crate) ascender: FtLong,
    pub(crate) descender: FtLong,
    pub(crate) height: FtLong,
    pub(crate) max_advance: FtLong,
}
/// `FT_SizeRec` (public part).
#[repr(C)]
pub(crate) struct FtSizeRec {
    face: *mut c_void,
    generic: FtGeneric,
    pub(crate) metrics: FtSizeMetrics,
}
/// `FT_FaceRec` (public part).
#[repr(C)]
pub(crate) struct FtFaceRec {
    pub(crate) num_faces: FtLong,
    pub(crate) face_index: FtLong,
    pub(crate) face_flags: FtLong,
    pub(crate) style_flags: FtLong,
    pub(crate) num_glyphs: FtLong,
    pub(crate) family_name: *const c_char,
    pub(crate) style_name: *const c_char,
    num_fixed_sizes: c_int,
    available_sizes: *mut c_void,
    num_charmaps: c_int,
    charmaps: *mut c_void,
    generic: FtGeneric,
    pub(crate) bbox: FtBBox,
    pub(crate) units_per_em: c_ushort,
    pub(crate) ascender: c_short,
    pub(crate) descender: c_short,
    pub(crate) height: c_short,
    max_advance_width: c_short,
    max_advance_height: c_short,
    underline_position: c_short,
    underline_thickness: c_short,
    pub(crate) glyph: *mut FtGlyphSlotRec,
    pub(crate) size: *mut FtSizeRec,
}

/// `FT_Outline_MoveToFunc` / `FT_Outline_LineToFunc`.
pub(crate) type FtOutlinePointFunc = unsafe extern "C" fn(*const FtVector, *mut c_void) -> c_int;
/// `FT_Outline_ConicToFunc`.
pub(crate) type FtOutlineConicFunc =
    unsafe extern "C" fn(*const FtVector, *const FtVector, *mut c_void) -> c_int;
/// `FT_Outline_CubicToFunc`.
pub(crate) type FtOutlineCubicFunc =
    unsafe extern "C" fn(*const FtVector, *const FtVector, *const FtVector, *mut c_void) -> c_int;
/// `FT_Outline_Funcs`.
#[repr(C)]
pub(crate) struct FtOutlineFuncs {
    pub(crate) move_to: FtOutlinePointFunc,
    pub(crate) line_to: FtOutlinePointFunc,
    pub(crate) conic_to: FtOutlineConicFunc,
    pub(crate) cubic_to: FtOutlineCubicFunc,
    pub(crate) shift: c_int,
    pub(crate) delta: FtLong,
}

/// The bound FreeType entry points.
pub(crate) struct FreeTypeApi {
    pub(crate) FT_Init_FreeType: unsafe extern "C" fn(*mut *mut c_void) -> FtError,
    pub(crate) FT_Done_FreeType: unsafe extern "C" fn(*mut c_void) -> FtError,
    pub(crate) FT_New_Face:
        unsafe extern "C" fn(*mut c_void, *const c_char, FtLong, *mut *mut FtFaceRec) -> FtError,
    pub(crate) FT_New_Memory_Face: unsafe extern "C" fn(
        *mut c_void,
        *const u8,
        FtLong,
        FtLong,
        *mut *mut FtFaceRec,
    ) -> FtError,
    pub(crate) FT_Done_Face: unsafe extern "C" fn(*mut FtFaceRec) -> FtError,
    pub(crate) FT_Select_Charmap: unsafe extern "C" fn(*mut FtFaceRec, u32) -> FtError,
    pub(crate) FT_Set_Char_Size:
        unsafe extern "C" fn(*mut FtFaceRec, FtLong, FtLong, c_uint, c_uint) -> FtError,
    pub(crate) FT_MulFix: unsafe extern "C" fn(FtLong, FtLong) -> FtLong,
    pub(crate) FT_Set_Transform: unsafe extern "C" fn(*mut FtFaceRec, *mut FtMatrix, *mut FtVector),
    pub(crate) FT_Get_Char_Index: unsafe extern "C" fn(*mut FtFaceRec, c_ulong) -> c_uint,
    pub(crate) FT_Load_Glyph: unsafe extern "C" fn(*mut FtFaceRec, c_uint, i32) -> FtError,
    pub(crate) FT_Outline_Check: unsafe extern "C" fn(*mut FtOutline) -> FtError,
    pub(crate) FT_Outline_Decompose:
        unsafe extern "C" fn(*mut FtOutline, *const FtOutlineFuncs, *mut c_void) -> FtError,
    pub(crate) FT_Get_Kerning:
        unsafe extern "C" fn(*mut FtFaceRec, c_uint, c_uint, c_uint, *mut FtVector) -> FtError,
    pub(crate) FT_Library_Version:
        unsafe extern "C" fn(*mut c_void, *mut c_int, *mut c_int, *mut c_int),
}

/// The loaded FreeType, or `None` when `libfreetype.so.6` (with every entry point above)
/// cannot be loaded; `freetype().is_some()`.
pub(crate) fn freetype() -> Option<&'static FreeTypeApi> {
    static LOADED: OnceLock<Option<FreeTypeApi>> = OnceLock::new();
    LOADED.get_or_init(load).as_ref()
}

fn load() -> Option<FreeTypeApi> {
    let lib = DynamicLibrary::open(&[
        "libfreetype.so.6",
        "libfreetype.so",
        "libfreetype.6.dylib",
        "freetype.dll",
        "libfreetype.dll",
        "freetype6.dll",
        "libfreetype-6.dll",
    ])?;
    bind_symbols!(
        &lib,
        FreeTypeApi {
            FT_Init_FreeType,
            FT_Done_FreeType,
            FT_New_Face,
            FT_New_Memory_Face,
            FT_Done_Face,
            FT_Select_Charmap,
            FT_Set_Char_Size,
            FT_MulFix,
            FT_Set_Transform,
            FT_Get_Char_Index,
            FT_Load_Glyph,
            FT_Outline_Check,
            FT_Outline_Decompose,
            FT_Get_Kerning,
            FT_Library_Version,
        }
    )
}
