// Copyright (c) 1998-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
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

//! `OSD::RealToCString` and `OSD::CStringToReal`: locale-independent real text.
use super::c_format::format_e;
use crate::standard::strtod;

/// `OSD::RealToCString`: `%.17e`, then an exact `e+00` exponent is dropped and trailing
/// fraction zeros are removed (with the point when no fraction digit remains);
/// `real_to_c_string(1.5) == "1.5"`, `real_to_c_string(1e5) == "1e+05"`.
///
/// The native `Sprintf(...) <= 0` failure (BUC60808) cannot occur for a finite or
/// non-finite double, so the Rust call always returns the text.
pub fn real_to_c_string(value: f64) -> String {
    let mut text: Vec<u8> = format_e(value, 17).into_bytes();
    let Some(exponent_at) = text.iter().position(|&byte| byte == b'e') else {
        return String::from_utf8(text).expect("ASCII number text");
    };
    let mut exponent: Vec<u8> = text.split_off(exponent_at);
    if exponent == b"e+00" {
        exponent.clear();
    }
    strip_fraction_zeros(&mut text);
    text.extend_from_slice(&exponent);
    String::from_utf8(text).expect("ASCII number text")
}

/// The native `q` walk: drop the zeros before the exponent; when any were dropped, the
/// exponent lands after the last kept digit, or over the point when it is reached.
fn strip_fraction_zeros(mantissa: &mut Vec<u8>) {
    let kept: usize = mantissa.len()
        - mantissa
            .iter()
            .rev()
            .take_while(|&&byte| byte == b'0')
            .count();
    if kept == mantissa.len() {
        return;
    }
    let point_reached: bool = mantissa[kept - 1] == b'.';
    mantissa.truncate(if point_reached { kept - 1 } else { kept });
}

/// `OSD::CStringToReal`: parses with OCCT's `Strtod` and succeeds only when the whole
/// text (up to a NUL) was consumed, so the empty string succeeds with 0;
/// `c_string_to_real("2.5") == (true, 2.5)`.
pub fn c_string_to_real(text: &str) -> (bool, f64) {
    let text: &str = text.split('\0').next().unwrap_or_default();
    let (value, end) = strtod(text);
    (end == text.len(), value)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_text_strips_like_the_native_pointer_walk() {
        assert_eq!(real_to_c_string(1.0), "1");
        assert_eq!(real_to_c_string(1.5), "1.5");
        assert_eq!(real_to_c_string(-0.25), "-2.5e-01");
        assert_eq!(real_to_c_string(1e5), "1e+05");
        assert_eq!(real_to_c_string(0.1), "1.00000000000000006e-01");
        assert_eq!(real_to_c_string(0.0), "0");
        assert_eq!(real_to_c_string(f64::NAN), "nan");
        assert_eq!(c_string_to_real(""), (true, 0.0));
        assert_eq!(c_string_to_real("2.5x"), (false, 2.5));
    }
}
