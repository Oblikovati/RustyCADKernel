//! Native font database dispatch, font-file checks and embedded fallback ownership.
use super::font_check::check_font;
use super::{FontFTLibrary, FontFontMgr, FontSystemFont};
use rustycad_tkernel::ncollection::{BaseAllocator, Buffer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::tcollection::AsciiString;
/// The embedded DejaVu Sans Latin subset (`Font_DejavuSans_Latin_woff`).
const EMBED_FALLBACK_FONT: &[u8] = include_bytes!("dejavu_sans_latin.woff");

impl FontFontMgr {
    /// `InitFontDataBase`: forgets the fonts, then registers every font of the system font
    /// directories. A malformed `fonts.dir` line that natively raises from
    /// `TCollection_AsciiString` stops the scan with that failure.
    /// Example: `manager.init_font_data_base()?`.
    pub fn init_font_data_base(&self) -> Result<(), Failure> {
        self.clear_font_data_base();
        let library = FontFTLibrary::new();
        #[cfg(windows)]
        return self.register_windows_font_database(&library);
        #[cfg(not(windows))]
        self.register_directory_font_database(&library)
    }
}
impl FontFontMgr {
    /// `CheckFont(theFonts, thePath)`: appends the fonts of the file's first face (and its
    /// instances); `manager.check_font_into(&mut fonts, &path)?`.
    pub fn check_font_into(
        &self,
        fonts: &mut Vec<Handle<FontSystemFont>>,
        path: &AsciiString,
    ) -> Result<bool, Failure> {
        check_font(fonts, &FontFTLibrary::new(), path, 0)
    }
    /// `CheckFont(thePath)`: the font of the file's first face, or null; `manager.check_font(&path)?`.
    pub fn check_font(&self, path: &AsciiString) -> Result<Handle<FontSystemFont>, Failure> {
        let mut fonts = Vec::new();
        let found = check_font(&mut fonts, &FontFTLibrary::new(), path, 0)?;
        Ok(if found {
            fonts.swap_remove(0)
        } else {
            Handle::default()
        })
    }
    /// `EmbedFallbackFont`: the embedded DejaVu Sans Latin WOFF data (a copy, as the native
    /// buffer wraps static data). Example: `FontFontMgr::embed_fallback_font()`.
    pub fn embed_fallback_font() -> Handle<Buffer> {
        let size = EMBED_FALLBACK_FONT.len();
        let Ok(mut buffer) = Buffer::new(Some(BaseAllocator::common()), size) else {
            return Handle::default();
        };
        if buffer.size() != size {
            return Handle::default();
        }
        buffer.as_mut_slice()[..size].copy_from_slice(EMBED_FALLBACK_FONT);
        Handle::new(buffer)
    }
}
