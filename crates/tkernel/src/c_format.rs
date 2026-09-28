// Created on: 2001-04-27
// Created by: OCC Team
// Copyright (c) 2001-2014 OPEN CASCADE SAS
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

//! The subset of C `sprintf` used by `Message_Msg::Arg`, `Message::FillTime` and the OSD
//! timing classes (`OSD::RealToCString`, `OSD_Timer`/`OSD_Chronometer` stream output).
//!
//! Message formats are user data (`%-10.3f`, `%05d`, `%s`), passed verbatim to `Sprintf`,
//! so their output must follow glibc: flags `-+ #0`, width, precision, the `h`/`hh`/`l`/`ll`
//! length modifiers and the conversions `d i o u x X f F e E g G s`. Rust's float formatting
//! is exact (like glibc's), so only the C layout rules are implemented here.
mod float;
mod integer;

/// One variadic argument; the conversion reads it the way C's `va_arg` would.
#[derive(Clone, Copy, Debug)]
pub(crate) enum CArg<'a> {
    /// An `int` argument.
    Int(i32),
    /// A `double` argument.
    Real(f64),
    /// A NUL-free `const char*` argument.
    Text(&'a [u8]),
}

/// Parsed flags, width, precision and length modifier of one conversion.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Spec {
    pub(crate) left: bool,
    pub(crate) plus: bool,
    pub(crate) space: bool,
    pub(crate) alternate: bool,
    pub(crate) zero: bool,
    pub(crate) width: usize,
    pub(crate) precision: Option<usize>,
    /// Length modifier: -2 `hh`, -1 `h`, 0 none, 1 `l`, 2 `ll`.
    pub(crate) length: i8,
    pub(crate) conversion: u8,
}

/// Formats `format` with `args` like C `sprintf`; `c_sprintf(b"%02d", &[CArg::Int(5)]) == b"05"`.
pub(crate) fn c_sprintf(format: &[u8], args: &[CArg<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut arguments = args.iter();
    let mut index = 0;
    while index < format.len() {
        if format[index] != b'%' {
            output.push(format[index]);
            index += 1;
            continue;
        }
        let (spec, end) = parse_spec(format, index + 1);
        index = end;
        render(&spec, &mut arguments, &format[..end], &mut output);
    }
    output
}

fn render(
    spec: &Spec,
    arguments: &mut std::slice::Iter<'_, CArg<'_>>,
    consumed: &[u8],
    output: &mut Vec<u8>,
) {
    let body = match spec.conversion {
        b'%' => vec![b'%'],
        b'd' | b'i' | b'o' | b'u' | b'x' | b'X' => integer::format(spec, arguments.next()),
        b'f' | b'F' | b'e' | b'E' | b'g' | b'G' => float::format(spec, arguments.next()),
        b's' => text(spec, arguments.next()),
        // An invalid conversion is undefined behavior in C; glibc echoes the directive.
        _ => consumed[consumed.iter().rposition(|&byte| byte == b'%').unwrap_or(0)..].to_vec(),
    };
    output.extend_from_slice(&body);
}

fn parse_spec(format: &[u8], mut index: usize) -> (Spec, usize) {
    let mut spec = Spec::default();
    while let Some(&flag) = format.get(index) {
        match flag {
            b'-' => spec.left = true,
            b'+' => spec.plus = true,
            b' ' => spec.space = true,
            b'#' => spec.alternate = true,
            b'0' => spec.zero = true,
            _ => break,
        }
        index += 1;
    }
    (spec.width, index) = digits(format, index);
    if format.get(index) == Some(&b'.') {
        let (precision, end) = digits(format, index + 1);
        spec.precision = Some(precision);
        index = end;
    }
    (spec.length, index) = length_modifier(format, index);
    spec.conversion = format.get(index).copied().unwrap_or(0);
    (spec, (index + 1).min(format.len()))
}

fn digits(format: &[u8], mut index: usize) -> (usize, usize) {
    let mut value: usize = 0;
    while let Some(digit) = format.get(index).filter(|byte| byte.is_ascii_digit()) {
        value = value
            .saturating_mul(10)
            .saturating_add(usize::from(digit - b'0'));
        index += 1;
    }
    (value, index)
}

fn length_modifier(format: &[u8], index: usize) -> (i8, usize) {
    match (format.get(index), format.get(index + 1)) {
        (Some(b'h'), Some(b'h')) => (-2, index + 2),
        (Some(b'h'), _) => (-1, index + 1),
        (Some(b'l'), Some(b'l')) => (2, index + 2),
        (Some(b'l'), _) => (1, index + 1),
        _ => (0, index),
    }
}

fn text(spec: &Spec, argument: Option<&CArg<'_>>) -> Vec<u8> {
    let bytes: &[u8] = match argument {
        Some(CArg::Text(bytes)) => bytes,
        _ => b"(null)",
    };
    let shown = spec
        .precision
        .map_or(bytes, |limit| &bytes[..limit.min(bytes.len())]);
    pad(spec, Vec::new(), shown.to_vec(), false)
}

/// Joins sign/prefix and digits, padding to the width with spaces or (numeric) zeros.
pub(crate) fn pad(spec: &Spec, prefix: Vec<u8>, digits: Vec<u8>, zero_allowed: bool) -> Vec<u8> {
    let length = prefix.len() + digits.len();
    let fill = spec.width.saturating_sub(length);
    if spec.left {
        return [prefix, digits, vec![b' '; fill]].concat();
    }
    if spec.zero && zero_allowed {
        return [prefix, vec![b'0'; fill], digits].concat();
    }
    [vec![b' '; fill], prefix, digits].concat()
}

/// The `+`/space sign prefix of a signed conversion.
pub(crate) fn sign_prefix(spec: &Spec, negative: bool) -> Vec<u8> {
    match (negative, spec.plus, spec.space) {
        (true, _, _) => b"-".to_vec(),
        (false, true, _) => b"+".to_vec(),
        (false, false, true) => b" ".to_vec(),
        _ => Vec::new(),
    }
}

/// `printf("%.*e", precision, value)`: `OSD::RealToCString`; `format_e(1.5, 2) == "1.50e+00"`.
pub(crate) fn format_e(value: f64, precision: usize) -> String {
    format_real(&format!("%.{precision}e"), value)
}

/// `printf("%.*g", precision, value)`, the default `ostream << double` spelling at
/// `precision(N)`; `format_g(5.5, 12) == "5.5"`.
pub(crate) fn format_g(value: f64, precision: usize) -> String {
    format_real(&format!("%.{precision}g"), value)
}

fn format_real(format: &str, value: f64) -> String {
    let bytes: Vec<u8> = c_sprintf(format.as_bytes(), &[CArg::Real(value)]);
    String::from_utf8(bytes).expect("C float formatting is ASCII")
}

#[cfg(test)]
mod real_tests {
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
