//! Font aspect conversion preserves every valid native enum value.
use super::FontFontAspect;
impl FontFontAspect {
    /// Preserve the original native value; `FontFontAspect::from_native_value(-2)`.
    pub fn from_native_value(value: i32) -> Option<Self> {
        NATIVE_FONT_ASPECTS
            .iter()
            .copied()
            .find(|candidate| *candidate as i32 == value)
    }
}
const NATIVE_FONT_ASPECTS: &[FontFontAspect] = &[
    FontFontAspect::NativeMinus4,
    FontFontAspect::NativeMinus3,
    FontFontAspect::NativeMinus2,
    FontFontAspect::Undefined,
    FontFontAspect::Regular,
    FontFontAspect::Bold,
    FontFontAspect::Italic,
    FontFontAspect::BoldItalic,
];
