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

//! The two C (glibc) floating-point spellings the timing classes print: `%.Ne`
//! (`OSD::RealToCString`) and `ostream << double` at `precision(N)`, i.e. `%.Ng`.
//! Rust's exact decimal expansion rounds like glibc (round-half-even on exact ties).

/// glibc's text for non-finite values, or `None` for a finite value.
fn non_finite(value: f64) -> Option<String> {
    let sign: &str = if value.is_sign_negative() { "-" } else { "" };
    if value.is_nan() {
        return Some(format!("{sign}nan"));
    }
    value.is_infinite().then(|| format!("{sign}inf"))
}

/// Rust's `{:.Ne}` mantissa and decimal exponent of a finite value.
fn scientific_parts(value: f64, precision: usize) -> (String, i32) {
    let text: String = format!("{value:.precision$e}");
    let (mantissa, exponent) = text
        .split_once('e')
        .expect("Rust `e` format has an exponent");
    (
        mantissa.to_string(),
        exponent.parse().expect("decimal exponent"),
    )
}

fn c_exponent(exponent: i32) -> String {
    format!(
        "e{}{:02}",
        if exponent < 0 { '-' } else { '+' },
        exponent.unsigned_abs()
    )
}

/// `printf("%.*e", precision, value)`; `format_e(1.5, 2) == "1.50e+00"`.
pub(crate) fn format_e(value: f64, precision: usize) -> String {
    if let Some(text) = non_finite(value) {
        return text;
    }
    let (mantissa, exponent) = scientific_parts(value, precision);
    mantissa + &c_exponent(exponent)
}

fn trim_fraction(text: &str) -> &str {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.')
    } else {
        text
    }
}

/// `printf("%.*g", precision, value)` (a precision of 0 means 1), the default
/// `ostream` float spelling; `format_g(5.5, 12) == "5.5"`.
pub(crate) fn format_g(value: f64, precision: usize) -> String {
    if let Some(text) = non_finite(value) {
        return text;
    }
    let precision: usize = precision.max(1);
    let (mantissa, exponent) = scientific_parts(value, precision - 1);
    if exponent < -4 || exponent >= precision as i32 {
        return trim_fraction(&mantissa).to_string() + &c_exponent(exponent);
    }
    let decimals: usize = (precision as i32 - 1 - exponent) as usize;
    trim_fraction(&format!("{value:.decimals$}")).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn c_spellings_of_finite_and_special_values() {
        assert_eq!(format_e(1.5, 2), "1.50e+00");
        assert_eq!(format_e(-1e-300, 1), "-1.0e-300");
        assert_eq!(format_e(-0.0, 0), "-0e+00");
        assert_eq!(format_e(f64::NAN, 17), "nan");
        assert_eq!(format_e(f64::NEG_INFINITY, 17), "-inf");
        assert_eq!(format_g(5.5, 12), "5.5");
        assert_eq!(format_g(0.0, 12), "0");
        assert_eq!(format_g(1e12, 12), "1e+12");
        assert_eq!(format_g(123456789012.0, 12), "123456789012");
        assert_eq!(format_g(0.0001, 12), "0.0001");
        assert_eq!(format_g(0.00001, 12), "1e-05");
        assert_eq!(format_g(0.125, 2), "0.12");
    }
}
