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

//! Formatted `operator>>`/`operator<<` of libstdc++ in the "C" locale.
use super::{IoStream, format_general, is_c_space, state};

/// The digits `num_get::_M_extract_int` read for a decimal integer.
struct ScannedInteger {
    negative: bool,
    magnitude: u64,
    overflow: bool,
    digits: bool,
}

impl IoStream {
    /// `num_get::_M_extract_int` in base 10 against `max` (the unsigned magnitude limit).
    fn scan_integer(&mut self, signed_min_magnitude: Option<u64>, max: u64) -> ScannedInteger {
        let mut scan = ScannedInteger {
            negative: false,
            magnitude: 0,
            overflow: false,
            digits: false,
        };
        if let Some(sign @ (b'+' | b'-')) = self.peek_byte() {
            scan.negative = sign == b'-';
            self.cursor += 1;
        }
        let limit = match signed_min_magnitude {
            Some(magnitude) if scan.negative => magnitude,
            _ => max,
        };
        while let Some(byte) = self.peek_byte().filter(u8::is_ascii_digit) {
            self.accumulate(&mut scan, u64::from(byte - b'0'), limit);
            self.cursor += 1;
        }
        scan
    }

    fn accumulate(&self, scan: &mut ScannedInteger, digit: u64, limit: u64) {
        scan.digits = true;
        if scan.magnitude > limit / 10 {
            scan.overflow = true;
            return;
        }
        scan.magnitude *= 10;
        scan.overflow |= scan.magnitude > limit - digit;
        scan.magnitude = scan.magnitude.wrapping_add(digit);
    }

    /// `operator>>(long&)`: the value `num_get` stores (0 or the clamped bound on failure).
    fn extract_long(&mut self) -> Option<(i64, u8)> {
        if !self.input_sentry(false) {
            return None;
        }
        let scan = self.scan_integer(Some(1 << 63), i64::MAX as u64);
        let mut error = if self.peek_byte().is_none() {
            state::EOF
        } else {
            0
        };
        let value = if !scan.digits {
            error |= state::FAIL;
            0
        } else if scan.overflow {
            error |= state::FAIL;
            if scan.negative { i64::MIN } else { i64::MAX }
        } else if scan.negative {
            (scan.magnitude as i64).wrapping_neg()
        } else {
            scan.magnitude as i64
        };
        Some((value, error))
    }

    /// `operator>>(int&)`: extracts a `long` and clamps it to `int` with `failbit`
    /// (libstdc++ resolution of LWG 696); returns `!fail()`; `stream.extract_int(&mut v)`.
    pub fn extract_int(&mut self, value: &mut i32) -> bool {
        if let Some((long, error)) = self.extract_long() {
            let (clamped, range) = clamp_long(long, i64::from(i32::MIN), i64::from(i32::MAX));
            *value = clamped as i32;
            self.set_state(error | range);
        }
        !self.fail()
    }

    /// `operator>>(short&)`, clamped like `int`; `stream.extract_short(&mut v)`.
    pub fn extract_short(&mut self, value: &mut i16) -> bool {
        if let Some((long, error)) = self.extract_long() {
            let (clamped, range) = clamp_long(long, i64::from(i16::MIN), i64::from(i16::MAX));
            *value = clamped as i16;
            self.set_state(error | range);
        }
        !self.fail()
    }

    /// `operator>>(unsigned short&)`: a leading minus negates modulo 2^16, as `strtoul`.
    /// Example: `stream.extract_unsigned_short(&mut v)`.
    pub fn extract_unsigned_short(&mut self, value: &mut u16) -> bool {
        if !self.input_sentry(false) {
            return false;
        }
        let scan = self.scan_integer(None, u64::from(u16::MAX));
        let mut error = self.eof_flag();
        if !scan.digits {
            error |= state::FAIL;
            *value = 0;
        } else if scan.overflow {
            error |= state::FAIL;
            *value = u16::MAX;
        } else {
            let magnitude = scan.magnitude as u16;
            *value = if scan.negative {
                magnitude.wrapping_neg()
            } else {
                magnitude
            };
        }
        self.set_state(error);
        !self.fail()
    }

    /// `operator>>(bool&)` without `boolalpha`: 0 and 1 only, anything else is `true`
    /// with `failbit`; `stream.extract_bool(&mut flag)`.
    pub fn extract_bool(&mut self, value: &mut bool) -> bool {
        let Some((long, error)) = self.extract_long() else {
            return false;
        };
        if long == 0 || long == 1 {
            *value = long == 1;
            self.set_state(error);
        } else {
            *value = true;
            self.set_state(state::FAIL | (error & state::EOF));
        }
        !self.fail()
    }

    fn eof_flag(&mut self) -> u8 {
        if self.peek_byte().is_none() {
            state::EOF
        } else {
            0
        }
    }

    /// `operator>>(char*)` into a buffer of `capacity` bytes whose size GCC knows
    /// (`__builtin_object_size` under optimisation): at most `capacity - 1` bytes, or
    /// `width() - 1` when smaller; a buffer filled exactly to the end of input gains
    /// `eofbit`. Returns the stored word; `stream.extract_word(100)`.
    pub fn extract_word(&mut self, capacity: usize) -> Vec<u8> {
        let width = self.width;
        let limit = if width > 0 && (width as usize) < capacity {
            width as usize
        } else {
            capacity
        };
        let word = self.extract_bounded_word(limit);
        let unbounded = width <= 0 || (capacity as i64) < width;
        if self.good() && unbounded && self.peek_byte().is_none() {
            self.set_state(state::EOF);
        }
        word
    }

    /// `__istream_extract`: skips spaces, copies up to `limit - 1` non-space bytes.
    fn extract_bounded_word(&mut self, limit: usize) -> Vec<u8> {
        let mut word = Vec::new();
        let mut error = 0;
        if self.input_sentry(false) {
            while word.len() + 1 < limit {
                match self.peek_byte() {
                    Some(byte) if !is_c_space(byte) => {
                        word.push(byte);
                        self.cursor += 1;
                    }
                    Some(_) => break,
                    None => {
                        error |= state::EOF;
                        break;
                    }
                }
            }
            self.width = 0;
        }
        if word.is_empty() {
            error |= state::FAIL;
        }
        self.set_state(error);
        word
    }
}

fn clamp_long(value: i64, min: i64, max: i64) -> (i64, u8) {
    if value < min {
        (min, state::FAIL)
    } else if value > max {
        (max, state::FAIL)
    } else {
        (value, 0)
    }
}

impl IoStream {
    /// `__ostream_insert`: bytes through the output sentry, `badbit` on a short write.
    /// Example: `stream.insert_bytes(b"BEGIN_INFO_SECTION\n")`.
    pub fn insert_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.write(bytes)
    }

    /// `operator<<(long)` in decimal, which `int`, `short` and `unsigned short` use.
    /// Example: `stream.insert_integer(-7)`.
    pub fn insert_integer(&mut self, value: i64) -> &mut Self {
        self.insert_bytes(value.to_string().as_bytes())
    }

    /// `operator<<(double)` with the default float field: `%.*g` at `precision()`.
    /// A `float` is promoted to `double` first; `stream.insert_real(0.1)`.
    pub fn insert_real(&mut self, value: f64) -> &mut Self {
        let precision = self.precision;
        self.insert_bytes(format_general(value, precision).as_bytes())
    }
}
