//! The minimal fontconfig binding of `Font_FontMgr` (visualization-design.md section 2):
//! `libfontconfig.so.1` is loaded at run time for the font directory list only.
#![allow(unsafe_code, non_snake_case)]
use crate::dynlib::{DynamicLibrary, bind_symbols};
use std::ffi::{CStr, c_char, c_void};
use std::sync::OnceLock;

/// The bound fontconfig entry points.
struct FontconfigApi {
    FcInitLoadConfig: unsafe extern "C" fn() -> *mut c_void,
    FcConfigGetFontDirs: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    FcStrListNext: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    FcStrListDone: unsafe extern "C" fn(*mut c_void),
    FcConfigDestroy: unsafe extern "C" fn(*mut c_void),
}

fn fontconfig() -> Option<&'static FontconfigApi> {
    static LOADED: OnceLock<Option<FontconfigApi>> = OnceLock::new();
    LOADED
        .get_or_init(|| {
            let lib = DynamicLibrary::open(&["libfontconfig.so.1", "libfontconfig.so"])?;
            bind_symbols!(
                &lib,
                FontconfigApi {
                    FcInitLoadConfig,
                    FcConfigGetFontDirs,
                    FcStrListNext,
                    FcStrListDone,
                    FcConfigDestroy,
                }
            )
        })
        .as_ref()
}

/// The font directories of the fontconfig configuration (`FcInitLoadConfig`,
/// `FcConfigGetFontDirs`), in fontconfig order; empty without fontconfig.
/// Example: `fontconfig_font_dirs()`.
pub(crate) fn fontconfig_font_dirs() -> Vec<Vec<u8>> {
    let Some(api) = fontconfig() else {
        return Vec::new();
    };
    // SAFETY: FcInitLoadConfig returns a new configuration or null.
    let config = unsafe { (api.FcInitLoadConfig)() };
    if config.is_null() {
        return Vec::new();
    }
    // SAFETY: `config` is a live configuration; the list (or null) is owned by the caller.
    let list = unsafe { (api.FcConfigGetFontDirs)(config) };
    let dirs = if list.is_null() {
        Vec::new()
    } else {
        drain(api, list)
    };
    // SAFETY: the configuration is destroyed once, after its list was released.
    unsafe { (api.FcConfigDestroy)(config) };
    dirs
}

/// Reads and releases a string list.
fn drain(api: &FontconfigApi, list: *mut c_void) -> Vec<Vec<u8>> {
    let mut dirs = Vec::new();
    loop {
        // SAFETY: `list` is a live FcStrList; each item is a NUL-terminated string owned by it.
        let item = unsafe { (api.FcStrListNext)(list) };
        if item.is_null() {
            break;
        }
        // SAFETY: a non-null item is a valid C string until the list is released.
        dirs.push(unsafe { CStr::from_ptr(item) }.to_bytes().to_vec());
    }
    // SAFETY: the list is released once, after its last read.
    unsafe { (api.FcStrListDone)(list) };
    dirs
}
