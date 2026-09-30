//! Resource_FormatType: the non-ASCII encodings Resource_Unicode converts.

/// Native `Resource_FormatType`, with the same discriminants; `FormatType::Sjis as i32 == 0`.
///
/// The legacy aliases map onto these variants: `Resource_FormatType_ANSI` and `Resource_ANSI`
/// are `NoConversion`, `Resource_SJIS`/`Resource_EUC`/`Resource_GB` are `Sjis`/`Euc`/`Gb`
/// (see [`FormatType::ANSI`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum FormatType {
    /// Shift Japanese Industrial Standards.
    Sjis = 0,
    /// Extended Unix Code (Japanese tables).
    Euc = 1,
    /// Bytes widened unchanged (also `Resource_ANSI`).
    NoConversion = 2,
    /// GB2312 simplified Chinese.
    Gb = 3,
    /// Multi-byte UTF-8.
    Utf8 = 4,
    /// The C library's active locale (the "C" locale in OCCT processes).
    SystemLocale = 5,
    /// Windows-1250 Central European.
    Cp1250 = 6,
    /// Windows-1251 Cyrillic.
    Cp1251 = 7,
    /// Windows-1252 Western European.
    Cp1252 = 8,
    /// Windows-1253 Greek.
    Cp1253 = 9,
    /// Windows-1254 Turkish.
    Cp1254 = 10,
    /// Windows-1255 Hebrew.
    Cp1255 = 11,
    /// Windows-1256 Arabic.
    Cp1256 = 12,
    /// Windows-1257 Baltic.
    Cp1257 = 13,
    /// Windows-1258 Vietnamese.
    Cp1258 = 14,
    /// ISO 8859-1 Western European.
    Iso8859_1 = 15,
    /// ISO 8859-2 Central European.
    Iso8859_2 = 16,
    /// ISO 8859-3 South European.
    Iso8859_3 = 17,
    /// ISO 8859-4 North European.
    Iso8859_4 = 18,
    /// ISO 8859-5 Cyrillic.
    Iso8859_5 = 19,
    /// ISO 8859-6 Arabic.
    Iso8859_6 = 20,
    /// ISO 8859-7 Greek.
    Iso8859_7 = 21,
    /// ISO 8859-8 Hebrew.
    Iso8859_8 = 22,
    /// ISO 8859-9 Turkish.
    Iso8859_9 = 23,
    /// Code page 850 (DOS Western European).
    Cp850 = 24,
    /// GBK unified Chinese.
    Gbk = 25,
    /// Big5 traditional Chinese.
    Big5 = 26,
}
impl FormatType {
    /// The `Resource_FormatType_ANSI` / `Resource_ANSI` alias; `FormatType::ANSI == FormatType::NoConversion`.
    pub const ANSI: Self = Self::NoConversion;
    /// Every format in discriminant order; `FormatType::ALL[26] == FormatType::Big5`.
    pub const ALL: [Self; 27] = [
        Self::Sjis,
        Self::Euc,
        Self::NoConversion,
        Self::Gb,
        Self::Utf8,
        Self::SystemLocale,
        Self::Cp1250,
        Self::Cp1251,
        Self::Cp1252,
        Self::Cp1253,
        Self::Cp1254,
        Self::Cp1255,
        Self::Cp1256,
        Self::Cp1257,
        Self::Cp1258,
        Self::Iso8859_1,
        Self::Iso8859_2,
        Self::Iso8859_3,
        Self::Iso8859_4,
        Self::Iso8859_5,
        Self::Iso8859_6,
        Self::Iso8859_7,
        Self::Iso8859_8,
        Self::Iso8859_9,
        Self::Cp850,
        Self::Gbk,
        Self::Big5,
    ];
    /// The format with a native discriminant, if any; `FormatType::from_raw(4) == Some(FormatType::Utf8)`.
    pub fn from_raw(value: i32) -> Option<Self> {
        usize::try_from(value)
            .ok()
            .and_then(|index| Self::ALL.get(index).copied())
    }
    /// Row of `THE_CODEPAGES_ANSI` for the single-byte code pages, `None` otherwise.
    pub(super) fn code_page_index(self) -> Option<usize> {
        let index = self as i32 - Self::Cp1250 as i32;
        (0..=Self::Cp850 as i32 - Self::Cp1250 as i32)
            .contains(&index)
            .then_some(index as usize)
    }
}
