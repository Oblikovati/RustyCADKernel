//! Font: font manager, FreeType fonts and text layout (OCCT package Font).
//!
//! FreeType and fontconfig are loaded at run time (visualization-design.md section 2); without
//! them the library is invalid and the font database holds only what can be read without them.
mod enums;
mod font_check;
#[cfg(not(windows))]
mod font_directory_discovery;
mod font_mgr;
mod font_mgr_database;
mod font_mgr_find;
#[cfg(any(windows, test))]
mod font_windows;
#[cfg(windows)]
mod font_windows_ffi;
#[cfg(not(windows))]
mod fontconfig_ffi;
pub(crate) mod freetype_ffi;
mod ft_font;
mod ft_font_metrics;
mod ft_font_outline;
mod ft_font_render;
mod ft_font_select;
mod ft_library;
mod glyph_outline;
mod rect;
mod system_font;
mod text_formatter;
mod text_formatter_format;

pub use enums::*;
pub use font_mgr::FontFontMgr;
pub use ft_font::{FontFTFont, FontFTFontParams};
pub use ft_library::FontFTLibrary;
pub use glyph_outline::{
    FontGlyphOutline, FontGlyphOutlineContour, FontGlyphOutlineFillRule, FontGlyphOutlineGlyph,
    FontGlyphOutlineGlyphHasher, FontGlyphOutlineGlyphIdentity, FontGlyphOutlineSegment,
    FontGlyphOutlineSegmentKind,
};
pub use rect::FontRect;
pub use system_font::FontSystemFont;
pub use text_formatter::{
    FontTextFormatter, FontTextFormatterIterationFilter, FontTextFormatterIterator,
};

mod native_font_aspect;
