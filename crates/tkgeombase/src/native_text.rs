// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/FEmTool and AppParCurves (stream output helper)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text a native `Standard_OStream << double` writes, shared by the Dump and
//! Out members of the TKGeomBase ports.
use rustycad_tkernel::tcollection::AsciiString;

/// `Standard_OStream << double` with default flags, which formats like C's `%g`
/// (six significant digits); `stream_real(0.5) == "0.5"`.
pub(crate) fn stream_real(value: f64) -> String {
    // AsciiString(double) uses the same `%g` conversion.
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}

/// `Standard_OStream << double` after `setf(std::ios::scientific)` and
/// `setprecision(precision)`: C's `%.*e` (a sign and at least two exponent digits;
/// `inf`, `nan` and their negative forms); `stream_scientific(1.5, 3) == "1.500e+00"`.
pub(crate) fn stream_scientific(value: f64, precision: usize) -> String {
    let sign: &str = if value.is_sign_negative() { "-" } else { "" };
    if value.is_nan() {
        return format!("{sign}nan");
    }
    if value.is_infinite() {
        return format!("{sign}inf");
    }
    // Rust's `{:e}` rounds the exact binary value as glibc's printf does; only the
    // exponent spelling differs.
    let text: String = format!("{value:.precision$e}");
    let (mantissa, exponent) = text.split_once('e').unwrap_or((text.as_str(), "0"));
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let exponent_sign: char = if exponent < 0 { '-' } else { '+' };
    format!("{mantissa}e{exponent_sign}{:02}", exponent.abs())
}
