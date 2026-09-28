/****************************************************************
 *
 * The author of this software is David M. Gay.
 *
 * Copyright (c) 1991, 2000, 2001 by Lucent Technologies.
 *
 * Permission to use, copy, modify, and distribute this software for any
 * purpose without fee is hereby granted, provided that this entire notice
 * is included in all copies of any software which is or includes a copy
 * or modification of this software and in all copies of the supporting
 * documentation for such software.
 *
 * THIS SOFTWARE IS BEING PROVIDED "AS IS", WITHOUT ANY EXPRESS OR IMPLIED
 * WARRANTY.  IN PARTICULAR, NEITHER THE AUTHOR NOR LUCENT MAKES ANY
 * REPRESENTATION OR WARRANTY OF ANY KIND CONCERNING THE MERCHANTABILITY
 * OF THIS SOFTWARE OR ITS FITNESS FOR ANY PARTICULAR PURPOSE.
 *
 ***************************************************************/

//! Infinity and NaN payload spellings accepted by OCCT's bundled David Gay parser.
pub(super) fn parse_special(bytes: &[u8], start: usize, negative: bool) -> Option<(f64, usize)> {
    let tail = bytes.get(start..)?;
    let sign = u64::from(negative) << 63;
    if tail
        .get(..3)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"inf"))
    {
        let count = if tail
            .get(..8)
            .is_some_and(|p| p.eq_ignore_ascii_case(b"infinity"))
        {
            8
        } else {
            3
        };
        return Some((f64::from_bits(0x7ff0000000000000 | sign), start + count));
    }
    if !tail
        .get(..3)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"nan"))
    {
        return None;
    }
    let (payload, end) = nan_payload(bytes, start + 3);
    Some((f64::from_bits(payload | sign), end))
}
fn nan_payload(bytes: &[u8], start: usize) -> (u64, usize) {
    if bytes.get(start) != Some(&b'(') {
        return (f64::NAN.to_bits(), start);
    }
    let mut cursor = start + 1;
    while bytes.get(cursor).is_some_and(|b| *b <= b' ') {
        cursor += 1;
    }
    if bytes
        .get(cursor..cursor + 2)
        .is_some_and(|p| p.eq_ignore_ascii_case(b"0x"))
    {
        cursor += 2;
    }
    let mut payload = Payload::default();
    while let Some(&byte) = bytes.get(cursor) {
        if let Some(digit) = (byte as char).to_digit(16) {
            payload.push(digit);
        } else if byte <= b' ' {
            payload.space();
        } else {
            break;
        }
        cursor += 1;
    }
    let end = bytes[cursor..]
        .iter()
        .position(|b| *b == b')')
        .map_or(start, |offset| cursor + offset + 1);
    (payload.bits(), end)
}
#[derive(Default)]
struct Payload {
    high: u32,
    low: u32,
    seen: bool,
    split: bool,
    shift: bool,
}
impl Payload {
    fn push(&mut self, digit: u32) {
        self.seen = true;
        if self.shift {
            self.high = self.low;
            self.low = 0;
            self.shift = false;
        }
        if !self.split {
            self.high = (self.high << 4) | (self.low >> 28);
        }
        self.low = (self.low << 4) | digit;
    }
    fn space(&mut self) {
        if !self.split && self.seen {
            self.split = true;
            self.shift = true;
        }
    }
    fn bits(&self) -> u64 {
        let fraction = (u64::from(self.high & 0xfffff) << 32) | u64::from(self.low);
        if fraction == 0 {
            f64::NAN.to_bits()
        } else {
            0x7ff0000000000000 | fraction
        }
    }
}
