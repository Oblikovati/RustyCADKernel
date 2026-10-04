//! Multibyte conversion in the active locale, with OCCT's buffer-sizing quirks.
use super::UtfString;
use crate::ncollection::UtfUnit;
#[cfg(not(target_os = "android"))]
use crate::ncollection::WideChar;

#[cfg(all(unix, not(target_os = "android")))]
#[path = "locale_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "locale_windows.rs"]
mod platform;
#[cfg(all(not(unix), not(windows), not(target_os = "android")))]
#[path = "locale_fallback.rs"]
mod platform;
#[cfg(all(test, not(windows), not(target_os = "android")))]
#[path = "locale_windows.rs"]
mod windows_locale_tests;

/// Temporary wide-character conversion storage used by `NCollection_UtfStringTool`.
///
/// `tool.from_locale(b"text")` returns a mutable borrowed, NUL-terminated wide buffer.
/// The buffer stays owned by the tool and is replaced by its next conversion.
#[cfg(not(target_os = "android"))]
pub struct UtfStringTool {
    wide_buffer: Vec<WideChar>,
}

#[cfg(not(target_os = "android"))]
#[allow(clippy::new_without_default)] // Keep the source constructor surface without adding an unmapped Default trait.
impl UtfStringTool {
    /// Creates an empty locale converter; `UtfStringTool::new().from_locale(b"abc")`.
    pub fn new() -> Self {
        Self {
            wide_buffer: Vec::new(),
        }
    }

    /// Converts active-locale bytes to a mutable borrowed NUL-terminated wide buffer.
    ///
    /// The returned slice is owned by this tool and becomes invalid after the next conversion.
    /// Example: `let wide = tool.from_locale(b"abc").unwrap(); assert_eq!(wide[0], 97.into());`.
    pub fn from_locale(&mut self, text: &[u8]) -> Option<&mut [WideChar]> {
        self.wide_buffer = platform::decode(text);
        (!self.wide_buffer.is_empty()).then_some(&mut self.wide_buffer)
    }

    /// Converts a wide-character sequence into the active locale; `UtfStringTool::to_locale(wide, &mut bytes)`.
    ///
    /// The input may include its terminating zero. A zero conversion stores a terminator when
    /// possible; a full buffer follows `wcstombs` and may have no terminator.
    pub fn to_locale(wide_string: &[WideChar], buffer: &mut [u8]) -> bool {
        if platform::encode(wide_string, buffer) != 0 {
            return true;
        }
        if let Some(first) = buffer.first_mut() {
            *first = 0;
        }
        false
    }
}

impl<T: UtfUnit> UtfString<T> {
    /// Replaces the contents with multibyte text in the active C locale (native `FromLocale`).
    ///
    /// Unix preserves the native `mblen` sizing quirk: the first character's byte count limits
    /// converted wide characters. Windows uses the active ANSI code page. Android bypasses
    /// locale conversion and interprets the bytes as UTF-8.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut text = UtfString::<u8>::new();
    /// text.from_locale(b"abc", -1);
    /// #[cfg(target_os = "android")]
    /// assert_eq!(text.to_c_string(), b"abc\0");
    /// #[cfg(windows)]
    /// assert_eq!(text.to_c_string(), b"abc\0");
    /// #[cfg(all(unix, not(target_os = "android")))]
    /// assert_eq!(text.to_c_string(), b"a\0");
    /// #[cfg(all(not(unix), not(windows), not(target_os = "android")))]
    /// assert_eq!(text.to_c_string(), b"a\0");
    /// ```
    pub fn from_locale(&mut self, text: &[u8], length: i32) {
        self.convert_locale_into(text, length);
    }

    /// Writes the string in the active C locale into `buffer` (native `ToLocale`, size = `buffer.len()`).
    ///
    /// Unix preserves `wcstombs` and its native result quirks. Windows converts through the active
    /// ANSI code page; Android copies UTF-8 and requires room for the converted text and NUL.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut buffer = [0xFFu8; 4];
    /// assert!(UtfString::<u8>::from_unicode_units(b"ab", -1).to_locale(&mut buffer));
    /// assert_eq!(buffer, [b'a', b'b', 0, 0xFF]);
    /// ```
    pub fn to_locale(&self, buffer: &mut [u8]) -> bool {
        self.to_locale_platform(buffer)
    }

    #[cfg(target_os = "android")]
    fn convert_locale_into(&mut self, text: &[u8], length: i32) {
        *self = android_from_locale(text, length);
    }

    #[cfg(not(target_os = "android"))]
    fn convert_locale_into(&mut self, text: &[u8], length: i32) {
        self.convert_locale_with_tool(text, length);
    }

    #[cfg(target_os = "android")]
    fn to_locale_platform(&self, buffer: &mut [u8]) -> bool {
        android_to_locale(&self.units, self.length, buffer)
    }

    #[cfg(not(target_os = "android"))]
    fn to_locale_platform(&self, buffer: &mut [u8]) -> bool {
        self.to_locale_tool(buffer)
    }

    #[cfg(not(target_os = "android"))]
    fn convert_locale_with_tool(&mut self, text: &[u8], length: i32) {
        let mut tool = UtfStringTool::new();
        let Some(wide) = tool.from_locale(text) else {
            self.clear();
            return;
        };
        self.from_unicode(wide, length);
    }

    #[cfg(not(target_os = "android"))]
    fn to_locale_tool(&self, buffer: &mut [u8]) -> bool {
        let wide = UtfString::<WideChar>::from_unicode_units(&self.units, self.length);
        UtfStringTool::to_locale(wide.to_c_string(), buffer)
    }
}

#[cfg(any(target_os = "android", test))]
fn android_from_locale<T: UtfUnit>(text: &[u8], length: i32) -> UtfString<T> {
    UtfString::from_unicode_units(text, length)
}

#[cfg(any(target_os = "android", test))]
fn android_to_locale<T: UtfUnit>(wide: &[T], length: i32, buffer: &mut [u8]) -> bool {
    let utf8 = UtfString::<u8>::from_unicode_units(wide, length);
    let output = utf8.to_c_string();
    if buffer.len() < output.len() {
        return false;
    }
    buffer[..output.len()].copy_from_slice(output);
    true
}

#[cfg(test)]
mod android_conversion_tests {
    use super::{android_from_locale, android_to_locale};
    use crate::ncollection::UtfString;

    #[test]
    fn android_input_uses_unicode_conversion_and_length_limit() {
        let wide = android_from_locale::<u16>("hé!".as_bytes(), 2);
        assert_eq!(wide.to_c_string(), &[b'h' as u16, 0xe9, 0]);
        assert_eq!(wide.length(), 2);
    }

    #[test]
    fn android_output_requires_room_for_utf8_terminator_and_preserves_short_buffer() {
        let text = UtfString::<u16>::from_unicode_units("hé".as_bytes(), -1);
        let mut short = [0xff; 2];
        assert!(!android_to_locale(text.to_c_string(), -1, &mut short));
        assert_eq!(short, [0xff; 2]);

        let mut complete = [0xff; 5];
        assert!(android_to_locale(text.to_c_string(), -1, &mut complete));
        assert_eq!(complete, [b'h', 0xc3, 0xa9, 0, 0xff]);
    }
}
