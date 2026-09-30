// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of the output conversions of the IntPatch Dump members
// (IntPatch_Point.cxx, IntPatch_WLine.cxx, IntPatch_RLine.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The glibc `printf` and `std::ostream` conversions the IntPatch `Dump` members use:
//! `%+10.20f`, `%10p` and the default `operator<<(double)` (C's `%g`).
use rustycad_tkernel::standard::Failure;
use std::fmt::{self, Write};

/// The Standard_Failure of a text sink that refused a write.
pub(crate) fn sink_failure(_: fmt::Error) -> Failure {
    Failure::with_message("Standard_Failure", "IntPatch Dump: the output sink failed")
}

/// glibc's `inf`/`nan` with the sign of `value` (`-nan` for a negative NaN) and a `+`
/// when `plus`.
fn special(value: f64, plus: bool) -> String {
    let word: &str = if value.is_nan() { "nan" } else { "inf" };
    if value.is_sign_negative() {
        return format!("-{word}");
    }
    if plus {
        format!("+{word}")
    } else {
        word.to_string()
    }
}

/// `%+10.20f`: a sign, fixed notation with 20 decimals, right-aligned in 10.
/// `signed_fixed(0.5) == "+0.50000000000000000000"`.
pub(crate) fn signed_fixed(value: f64) -> String {
    let text: String = if value.is_finite() {
        format!("{value:+.20}")
    } else {
        special(value, true)
    };
    format!("{text:>10}")
}

/// `%10p`: glibc's `(nil)` or `0x` and the hexadecimal address, right-aligned in 10.
/// `pointer(std::ptr::null()) == "     (nil)"`.
pub(super) fn pointer(address: *const ()) -> String {
    let text: String = if address.is_null() {
        "(nil)".to_string()
    } else {
        format!("{:#x}", address as usize)
    };
    format!("{text:>10}")
}

/// `std::cout << value` with the default flags: C's `%g` (six significant digits,
/// trailing zeros removed). `ostream_real(0.1) == "0.1"`.
pub(super) fn ostream_real(value: f64) -> String {
    if !value.is_finite() {
        return special(value, false);
    }
    if value == 0.0 {
        return if value.is_sign_negative() { "-0" } else { "0" }.to_string();
    }
    // The exponent after rounding to six significant digits picks the style (C11 7.21.6.1).
    let scientific: String = format!("{value:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    if !(-4..6).contains(&exponent) {
        let sign: char = if exponent < 0 { '-' } else { '+' };
        return format!("{}e{sign}{:02}", trim_zeros(mantissa), exponent.abs());
    }
    let decimals: usize = usize::try_from(5 - exponent).unwrap_or(0);
    trim_zeros(&format!("{value:.decimals$}"))
}

/// `%g` drops the trailing fraction zeros and a bare decimal point.
fn trim_zeros(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Writes `text` to `out`, mapping a refused write to a failure.
pub(super) fn put(out: &mut dyn Write, text: &str) -> Result<(), Failure> {
    out.write_str(text).map_err(sink_failure)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversions_follow_glibc() {
        assert_eq!(signed_fixed(-1.5), "-1.50000000000000000000");
        assert_eq!(signed_fixed(f64::INFINITY), "      +inf");
        assert_eq!(pointer(std::ptr::null()), "     (nil)");
        assert_eq!(ostream_real(1234567.0), "1.23457e+06");
        assert_eq!(ostream_real(0.0001), "0.0001");
        assert_eq!(ostream_real(0.00001), "1e-05");
        assert_eq!(ostream_real(3.0), "3");
        assert_eq!(ostream_real(-2.5), "-2.5");
    }
}
