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

//! Exact bit extraction and ties-to-even rounding for hexadecimal floating inputs.
use super::exponent_end;
pub(super) fn parse_hexadecimal(bytes: &[u8], start: usize, negative: bool) -> (f64, usize) {
    let (bits, mut end, fractional) = hex_bits(bytes, start + 2);
    if bits.is_empty() {
        return (signed_zero(negative), start + 1);
    }
    let exponent_end = exponent_end(bytes, end, b'p');
    let power = if exponent_end > end {
        exponent(bytes, end + 1, exponent_end)
    } else {
        0
    };
    end = exponent_end;
    let Some(first) = bits.iter().position(|bit| *bit != 0) else {
        return (signed_zero(negative), end);
    };
    let bits = &bits[first..];
    let top = bits.len() as i64 - 1 + power - 4 * fractional as i64;
    let encoded = encode_bits(bits, top) | (u64::from(negative) << 63);
    (f64::from_bits(encoded), end)
}
fn signed_zero(negative: bool) -> f64 {
    f64::from_bits(u64::from(negative) << 63)
}
fn hex_bits(bytes: &[u8], start: usize) -> (Vec<u8>, usize, usize) {
    let mut bits = Vec::new();
    let mut cursor = start;
    let mut decimal = false;
    let mut fractional = 0;
    while let Some(&byte) = bytes.get(cursor) {
        if byte == b'.' && !decimal {
            decimal = true;
            cursor += 1;
            continue;
        }
        let Some(digit) = (byte as char).to_digit(16) else {
            break;
        };
        bits.extend((0..4).rev().map(|shift| ((digit >> shift) & 1) as u8));
        fractional += usize::from(decimal);
        cursor += 1;
    }
    (bits, cursor, fractional)
}
fn exponent(bytes: &[u8], start: usize, end: usize) -> i64 {
    let negative = bytes[start] == b'-';
    let signed = matches!(bytes[start], b'+' | b'-');
    let mut value = 0_i64;
    for &byte in &bytes[start + usize::from(signed)..end] {
        value = (value * 10 + i64::from(byte - b'0')).min(20000);
    }
    if negative { -value } else { value }
}
fn encode_bits(bits: &[u8], mut top: i64) -> u64 {
    if top > 1023 {
        return 0x7ff0000000000000;
    }
    if top < -1075 {
        return 0;
    }
    let keep = if top >= -1022 {
        53
    } else {
        (top + 1075) as usize
    };
    let mut significand = rounded_significand(bits, keep);
    if top < -1022 {
        // Pinned OCCT 3d097a0 Standard_Strtod.cxx masks bit 52 even when
        // subnormal rounding carries into the minimum normal value.
        return significand & 0x000fffffffffffff;
    }
    if significand == (1_u64 << 53) {
        significand >>= 1;
        top += 1;
    }
    if top > 1023 {
        return 0x7ff0000000000000;
    }
    (((top + 1023) as u64) << 52) | (significand & 0x000fffffffffffff)
}
fn rounded_significand(bits: &[u8], keep: usize) -> u64 {
    let mut result = 0_u64;
    for index in 0..keep {
        result = (result << 1) | u64::from(*bits.get(index).unwrap_or(&0));
    }
    let guard = bits.get(keep) == Some(&1);
    let sticky = bits.get(keep + 1..).is_some_and(|tail| tail.contains(&1));
    if guard && (sticky || result & 1 != 0) {
        result += 1;
    }
    result
}
