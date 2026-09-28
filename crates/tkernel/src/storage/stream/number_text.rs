// Copyright (c) 1996-1999 Matra Datavision
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

//! glibc `printf("%.*g")`, which libstdc++ uses for `operator<<(double)`.

/// Formats `value` like `printf("%.*g", precision, value)` in the "C" locale.
///
/// Rust's `{:.N e}` rounds the exact binary value half-to-even, as glibc does, so the
/// significant digits agree; this function only rearranges them into `%g` layout.
///
/// ```
/// use rustycad_tkernel::storage::format_general;
/// assert_eq!(format_general(0.1, 17), "0.10000000000000001");
/// assert_eq!(format_general(1e22, 17), "1e+22");
/// assert_eq!(format_general(2.5, 1), "2");
/// ```
pub fn format_general(value: f64, precision: usize) -> String {
    if value.is_nan() {
        return if value.is_sign_negative() {
            "-nan"
        } else {
            "nan"
        }
        .into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-inf"
        } else {
            "inf"
        }
        .into();
    }
    let significant = precision.max(1);
    let scientific = format!("{value:.*e}", significant - 1);
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i64 = exponent.parse().unwrap();
    if exponent < -4 || exponent >= significant as i64 {
        return format!("{}e{exponent:+03}", strip_fraction_zeros(mantissa));
    }
    fixed_layout(mantissa, exponent)
}

/// Rewrites the rounded digits `d.ddd` with decimal exponent `exponent` in positional form.
fn fixed_layout(mantissa: &str, exponent: i64) -> String {
    let negative = mantissa.starts_with('-');
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let text = if exponent >= 0 {
        let integer_length = exponent as usize + 1;
        let (integer, fraction) = digits.split_at(integer_length.min(digits.len()));
        format!("{integer}.{fraction}")
    } else {
        format!("0.{}{digits}", "0".repeat((-exponent - 1) as usize))
    };
    let text = strip_fraction_zeros(&text);
    if negative {
        format!("-{text}")
    } else {
        text.to_string()
    }
}

fn strip_fraction_zeros(text: &str) -> &str {
    if !text.contains('.') {
        return text;
    }
    text.trim_end_matches('0').trim_end_matches('.')
}
