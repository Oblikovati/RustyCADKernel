//! Multibyte conversion in the active C locale, with OCCT's buffer-sizing quirks.
use super::UtfString;
use crate::ncollection::{UtfUnit, WideChar};

#[cfg(unix)]
#[path = "locale_unix.rs"]
mod platform;
#[cfg(not(unix))]
#[path = "locale_fallback.rs"]
mod platform;

impl<T: UtfUnit> UtfString<T> {
    /// Replaces the contents with multibyte text in the active C locale (native `FromLocale`).
    ///
    /// Quirk preserved: the native tool sizes its wide buffer with `mblen`, which measures the
    /// first character only. That byte count limits the number of converted wide characters.
    /// An invalid first character or empty input clears the string.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut text = UtfString::<u8>::new();
    /// text.from_locale(b"abc", -1);
    /// assert_eq!(text.to_c_string(), b"a\0");
    /// ```
    pub fn from_locale(&mut self, text: &[u8], length: i32) {
        let wide = platform::decode(text);
        if wide.is_empty() {
            self.clear();
            return;
        }
        self.from_unicode(&wide, length);
    }

    /// Writes the string in the active C locale into `buffer` (native `ToLocale`, size = `buffer.len()`).
    ///
    /// Quirks preserved from `wcstombs` and the native `<= 0` test on its unsigned result:
    /// only an empty conversion reports false (and stores a terminator); an unrepresentable symbol
    /// stops the copy but still reports true, and a full buffer gets no terminator.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut buffer = [0xFFu8; 4];
    /// assert!(UtfString::<u8>::from_unicode_units(b"ab", -1).to_locale(&mut buffer));
    /// assert_eq!(buffer, [b'a', b'b', 0, 0xFF]);
    /// ```
    pub fn to_locale(&self, buffer: &mut [u8]) -> bool {
        let wide = UtfString::<WideChar>::from_unicode_units(&self.units, self.length);
        if platform::encode(wide.to_c_string(), buffer) == 0 {
            // The native code stores the terminator even into a zero-sized buffer; a slice
            // cannot be written past its end, so an empty buffer stays untouched.
            if let Some(first) = buffer.first_mut() {
                *first = 0;
            }
            return false;
        }
        true
    }
}
