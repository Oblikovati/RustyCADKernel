//! `Font_FTLibrary`: one FreeType library instance, released with the object.
#![allow(unsafe_code)]
use super::freetype_ffi::{FT_ERR_OK, freetype};
use rustycad_tkernel::standard::StandardTransient;
use std::ffi::c_void;

/// A FreeType library; invalid when FreeType is missing or fails to initialise.
/// Example: `FontFTLibrary::new().is_valid()`.
pub struct FontFTLibrary {
    transient: StandardTransient,
    /// The `FT_Library` address, 0 for none.
    library: usize,
}
rustycad_tkernel::impl_transient!(
    FontFTLibrary,
    "Font_FTLibrary",
    StandardTransient,
    transient
);

impl Default for FontFTLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl FontFTLibrary {
    /// `FT_Init_FreeType`; a failure (or a missing FreeType) leaves the library invalid.
    /// Example: `FontFTLibrary::new()`.
    pub fn new() -> Self {
        Self {
            transient: StandardTransient::default(),
            library: init_freetype(),
        }
    }
    /// Whether the library is initialised; `library.is_valid()`.
    pub fn is_valid(&self) -> bool {
        self.library != 0
    }
    /// The `FT_Library` handle (null when invalid); `library.instance()`.
    pub fn instance(&self) -> *mut c_void {
        self.library as *mut c_void
    }
}

/// `FT_Init_FreeType`: the library address, 0 on failure.
fn init_freetype() -> usize {
    let Some(api) = freetype() else { return 0 };
    let mut library: *mut c_void = std::ptr::null_mut();
    // SAFETY: FT_Init_FreeType writes a new library handle into `library`.
    let error = unsafe { (api.FT_Init_FreeType)(&mut library) };
    if error != FT_ERR_OK {
        0
    } else {
        library as usize
    }
}

impl Drop for FontFTLibrary {
    /// `FT_Done_FreeType` of a valid library.
    fn drop(&mut self) {
        if let (true, Some(api)) = (self.is_valid(), freetype()) {
            // SAFETY: the handle came from FT_Init_FreeType and is released once, here.
            unsafe { (api.FT_Done_FreeType)(self.instance()) };
        }
    }
}
