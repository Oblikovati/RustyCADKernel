//! Font aspects, hinting flags, search levels and Unicode subsets.

/// Specifies aspect of system font.
///
/// Native enumeration `Font_FontAspect` (`Font_FontAspect.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontFontAspect {
    /// special value reserved for undefined aspect.
    Undefined = -1,
    /// normal (regular) aspect.
    Regular = 0,
    /// bold aspect.
    Bold = 1,
    /// italic aspect.
    Italic = 2,
    /// bold+italic aspect.
    BoldItalic = 3,
    /// Unnamed native value `-4` retained from the C++ enumeration.
    NativeMinus4 = -4,
    /// Unnamed native value `-3` retained from the C++ enumeration.
    NativeMinus3 = -3,
    /// Unnamed native value `-2` retained from the C++ enumeration.
    NativeMinus2 = -2,
}
impl FontFontAspect {
    /// Obsolete alias `Font_FA_Undefined` of [`Self::Undefined`].
    pub const FA_UNDEFINED: Self = Self::Undefined;
    /// Obsolete alias `Font_FA_Regular` of [`Self::Regular`].
    pub const FA_REGULAR: Self = Self::Regular;
    /// Obsolete alias `Font_FA_Bold` of [`Self::Bold`].
    pub const FA_BOLD: Self = Self::Bold;
    /// Obsolete alias `Font_FA_Italic` of [`Self::Italic`].
    pub const FA_ITALIC: Self = Self::Italic;
    /// Obsolete alias `Font_FA_BoldItalic` of [`Self::BoldItalic`].
    pub const FA_BOLD_ITALIC: Self = Self::BoldItalic;
}

/// Enumeration defining font hinting options.
///
/// Native enumeration `Font_Hinting` (`Font_Hinting.hxx`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FontHinting(pub i32);
impl FontHinting {
    /// no      hinting (FT_LOAD_NO_HINTING).
    pub const OFF: Self = Self(0);
    /// default hinting (FT_LOAD_TARGET_NORMAL).
    pub const NORMAL: Self = Self(1);
    /// light   hinting (FT_LOAD_TARGET_LIGHT).
    pub const LIGHT: Self = Self(2);
    /// prefer autohinting over native hinting (FT_LOAD_FORCE_AUTOHINT).
    pub const FORCE_AUTOHINT: Self = Self(16);
    /// disallow autohinting (FT_LOAD_NO_AUTOHINT).
    pub const NO_AUTOHINT: Self = Self(32);
}

/// Enumeration defining font search restrictions.
///
/// Native enumeration `Font_StrictLevel` (`Font_StrictLevel.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontStrictLevel {
    /// search only for exact font.
    Strict = 0,
    /// search for exact font match and for aliases (ignore global fallback).
    Aliases = 1,
    /// search for any font, including global fallback.
    Any = 2,
}

/// Enumeration defining Unicode subsets.
///
/// Native enumeration `Font_UnicodeSubset` (`Font_UnicodeSubset.hxx`).
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FontUnicodeSubset {
    /// western letters.
    Western = 0,
    /// modern Korean letters.
    Korean = 1,
    /// Chinese characters (Chinese, Japanese, Korean and Vietnam).
    Cjk = 2,
    /// Arabic  characters.
    Arabic = 3,
    /// number of Unicode subsets.
    Nb = 4,
}

/// Number of defined aspects (`Font_FontAspect_NB`).
pub const FONT_FONT_ASPECT_NB: i32 = FontFontAspect::BoldItalic as i32 + 1;
