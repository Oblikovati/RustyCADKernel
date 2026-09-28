// Copyright (c) 2025 OPEN CASCADE SAS
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

use super::AsciiString;
use crate::standard::{Failure, strtod};
impl AsciiString {
    /// Formats six significant digits, following C's default `%g`; `AsciiString::from_real(3.3)`.
    pub fn from_real(value: f64) -> Self {
        Self::new(&format_general(value))
    }
    /// Parses a signed decimal prefix; `AsciiString::new("42x").integer_value()?`.
    pub fn integer_value(&self) -> Result<i32, Failure> {
        let (value, end) = integer_prefix(self.as_bytes());
        if end == 0 {
            return Err(Failure::new(
                "AsciiString integer: expected a decimal prefix",
            ));
        }
        Ok(value as i32)
    }
    /// Recognizes an integer prefix unless a decimal point follows it; `text.is_integer_value()`.
    pub fn is_integer_value(&self) -> bool {
        let (_, end) = integer_prefix(self.as_bytes());
        end != 0 && !self.as_bytes()[end..].contains(&b'.')
    }
    /// Parses a real prefix using the OCCT numeric scanner; `text.real_value()?`.
    pub fn real_value(&self) -> Result<f64, Failure> {
        let (value, end) = self.numeric_prefix();
        if end == 0 {
            Err(Failure::new("AsciiString real: expected a numeric prefix"))
        } else {
            Ok(value)
        }
    }
    /// Optionally requires full consumption, including OCCT's empty-string result; `text.is_real_value(true)`.
    pub fn is_real_value(&self, full: bool) -> bool {
        let (_, end) = self.numeric_prefix();
        if full {
            end == self.len() || self.as_bytes().get(end) == Some(&0)
        } else {
            end != 0
        }
    }
    fn numeric_prefix(&self) -> (f64, usize) {
        // A non-ASCII byte cannot extend a C-locale number; retain its byte offset.
        let end = self
            .as_bytes()
            .iter()
            .position(|byte| !byte.is_ascii())
            .unwrap_or(self.len());
        strtod(std::str::from_utf8(&self.as_bytes()[..end]).unwrap())
    }
}
fn integer_prefix(bytes: &[u8]) -> (i64, usize) {
    let mut index = 0;
    while bytes
        .get(index)
        .is_some_and(|&byte| crate::standard::character::is_space(byte))
    {
        index += 1;
    }
    let negative = bytes.get(index) == Some(&b'-');
    if matches!(bytes.get(index), Some(b'+' | b'-')) {
        index += 1;
    }
    let start = index;
    let mut magnitude = 0_u64;
    while let Some(byte) = bytes.get(index).filter(|byte| byte.is_ascii_digit()) {
        magnitude = magnitude
            .saturating_mul(10)
            .saturating_add(u64::from(*byte - b'0'));
        index += 1;
    }
    if index == start {
        return (0, 0);
    }
    let value = if negative {
        -(magnitude.min(1_u64 << 63) as i128) as i64
    } else {
        magnitude.min(i64::MAX as u64) as i64
    };
    (value, index)
}
fn format_general(value: f64) -> String {
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
    let scientific = format!("{value:.5e}");
    let (mantissa, exponent) = scientific.split_once('e').unwrap();
    let exponent: i32 = exponent.parse().unwrap();
    if !(-4..6).contains(&exponent) {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        return format!("{mantissa}e{exponent:+03}");
    }
    let decimals = (5 - exponent).max(0) as usize;
    let fixed = format!("{value:.decimals$}");
    if fixed.contains('.') {
        fixed.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        fixed
    }
}
