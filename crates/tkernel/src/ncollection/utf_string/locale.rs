// Copyright (c) 2013-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! FromLocale/ToLocale over the POSIX "C" locale (NCollection_UtfStringTool on glibc).
//!
//! Every C/C++ program starts in the "C" locale and OCCT never calls `setlocale`; Rust has no
//! process locale, so this module implements exactly that locale: single-byte ASCII, where
//! bytes and wide characters above 0x7F are conversion errors.
use super::UtfString;
use crate::ncollection::{UtfUnit, WideChar};

/// The C-locale `mblen(text, MB_CUR_MAX)`: 0 for an empty string, 1 for ASCII, -1 otherwise.
fn c_locale_mblen(text: &[u8]) -> i32 {
    match text.first().copied().unwrap_or(0) {
        0 => 0,
        0x01..=0x7F => 1,
        _ => -1,
    }
}

/// The C-locale `wcstombs(buffer, wide, buffer.len())`: bytes stored before the terminator,
/// `usize::MAX` (the native `(size_t)-1`) on a non-ASCII character. A full buffer is left
/// without a terminator, as the C function does.
fn c_locale_wcstombs(wide: &[WideChar], buffer: &mut [u8]) -> usize {
    for (written, slot) in buffer.iter_mut().enumerate() {
        #[allow(clippy::useless_conversion)] // WideChar is u16 on Windows
        let symbol: u32 = wide.get(written).map_or(0, |&unit| unit.into());
        if symbol > 0x7F {
            return usize::MAX;
        }
        *slot = symbol as u8;
        if symbol == 0 {
            return written;
        }
    }
    buffer.len()
}

impl<T: UtfUnit> UtfString<T> {
    /// Replaces the contents with multibyte text in the C locale (native `FromLocale`).
    ///
    /// Quirk preserved: the native tool sizes its wide buffer with `mblen`, which measures the
    /// first character only, so at most one character is converted; a leading non-ASCII byte or
    /// an empty input clears the string.
    ///
    /// ```
    /// use rustycad_tkernel::ncollection::UtfString;
    /// let mut text = UtfString::<u8>::new();
    /// text.from_locale(b"abc", -1);
    /// assert_eq!(text.to_c_string(), b"a\0");
    /// ```
    pub fn from_locale(&mut self, text: &[u8], length: i32) {
        if c_locale_mblen(text) <= 0 {
            self.clear();
            return;
        }
        let wide: [WideChar; 2] = [WideChar::from(text[0]), 0];
        self.from_unicode(&wide, length);
    }

    /// Writes the string in the C locale into `buffer` (native `ToLocale`, size = `buffer.len()`).
    ///
    /// Quirks preserved from `wcstombs` and the native `<= 0` test on its unsigned result:
    /// only an empty conversion reports false (and stores a terminator); a non-ASCII symbol
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
        if c_locale_wcstombs(wide.to_c_string(), buffer) == 0 {
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
