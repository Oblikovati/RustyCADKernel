// Copyright (c) 2016-2019 OPEN CASCADE SAS
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

//! `FSD_Base64`: the Base64 codec with the RFC 4648 alphabet.
//!
//! The native decoder indexes its table with `unsigned(char)`, which reads far outside the
//! table for bytes of 0x80 and above; Rust indexes by byte value, where such bytes decode
//! as 0. Like the native reads of a C string, positions past the end read as NUL.
use crate::ncollection::{BaseAllocator, Buffer};
use crate::standard::Handle;
use crate::tcollection::AsciiString;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// `aBase64Codes`, copied verbatim: the value of each byte below 123, 0 elsewhere. Besides
/// the alphabet, `-` and `.` decode as 62 and `,` and `_` as 63.
#[rustfmt::skip]
const CODES: [u8; 256] = {
    const HEAD: [u8; 123] = [
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,
        0,  62, 63, 62, 62, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 0,  0,  0,  0,  0,
        0,  0,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9,  10, 11, 12, 13, 14, 15, 16, 17, 18,
        19, 20, 21, 22, 23, 24, 25, 0,  0,  0,  0,  63, 0,  26, 27, 28, 29, 30, 31, 32, 33,
        34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    ];
    let mut table = [0_u8; 256];
    let mut index = 0;
    while index < HEAD.len() {
        table[index] = HEAD[index];
        index += 1;
    }
    table
};

/// The Base64 codec.
///
/// ```
/// use rustycad_tkernel::fsd::Base64;
/// assert_eq!(Base64::encode(b"OCCT").as_bytes(), b"T0NDVA==");
/// let decoded = Base64::decode(b"T0NDVA==");
/// assert_eq!(decoded.get().unwrap().as_slice(), b"OCCT");
/// ```
pub struct Base64;

impl Base64 {
    /// `Encode(char*, size_t, const uint8_t*, size_t)`: with no destination, the encoded
    /// size; a destination shorter than that receives nothing and 0 is returned.
    /// Example: `Base64::encode_into(Some(&mut text), b"ab")`.
    pub fn encode_into(destination: Option<&mut [u8]>, data: &[u8]) -> usize {
        if data.is_empty() {
            return 0;
        }
        let size = 4 * data.len().div_ceil(3);
        let Some(destination) = destination else {
            return size;
        };
        if size > destination.len() {
            return 0;
        }
        encode_groups(&mut destination[..size], data);
        size
    }

    /// `Encode(const uint8_t*, size_t)`; `Base64::encode(b"data")`.
    pub fn encode(data: &[u8]) -> AsciiString {
        let mut text = vec![0_u8; Self::encode_into(None, data)];
        Self::encode_into(Some(&mut text), data);
        AsciiString::from_raw(&text)
    }

    /// `Decode(uint8_t*, size_t, const char*, size_t)`: with no destination, the decoded
    /// size; a destination shorter than that receives nothing and 0 is returned.
    /// Example: `Base64::decode_into(Some(&mut bytes), b"YWI=")`.
    pub fn decode_into(destination: Option<&mut [u8]>, encoded: &[u8]) -> usize {
        if encoded.is_empty() {
            return 0;
        }
        let (iterations, pad) = decoded_layout(encoded);
        let size = iterations / 4 * 3 + pad;
        let Some(destination) = destination else {
            return size;
        };
        if size > destination.len() {
            return 0;
        }
        decode_groups(destination, encoded, iterations, pad);
        size
    }

    /// `Decode(const char*, size_t)`: a buffer from the common allocator, allocated even
    /// for an empty result; a failed allocation gives a null handle.
    /// Example: `Base64::decode(b"YWI=")`.
    pub fn decode(encoded: &[u8]) -> Handle<Buffer> {
        let size = Self::decode_into(None, encoded);
        let Ok(mut buffer) = Buffer::new(Some(BaseAllocator::common()), 0) else {
            return Handle::default();
        };
        if size == 0 {
            return Handle::new(buffer);
        }
        if !buffer.allocate(size).unwrap_or(false) {
            return Handle::default();
        }
        Self::decode_into(Some(buffer.as_mut_slice()), encoded);
        Handle::new(buffer)
    }
}

fn encode_groups(text: &mut [u8], data: &[u8]) {
    let whole = data.len() - data.len() % 3;
    let mut output = 0;
    for group in data[..whole].as_chunks::<3>().0 {
        let word = (u32::from(group[0]) << 16) | (u32::from(group[1]) << 8) | u32::from(group[2]);
        for shift in [18, 12, 6, 0] {
            text[output] = ALPHABET[(word >> shift & 0x3F) as usize];
            output += 1;
        }
    }
    let tail = encode_tail(&data[whole..]);
    let remaining = text.len() - output;
    text[output..].copy_from_slice(&tail[..remaining]);
}

/// The last one or two bytes with their `=` padding.
fn encode_tail(tail: &[u8]) -> [u8; 4] {
    match *tail {
        [first, second] => {
            let word = (u32::from(first) << 8) | u32::from(second);
            [
                ALPHABET[(word >> 10) as usize],
                ALPHABET[(word >> 4 & 0x3F) as usize],
                ALPHABET[((word & 0xF) << 2) as usize],
                b'=',
            ]
        }
        [first] => {
            let word = u32::from(first);
            [
                ALPHABET[(word >> 2) as usize],
                ALPHABET[((word & 3) << 4) as usize],
                b'=',
                b'=',
            ]
        }
        _ => [b'='; 4],
    }
}

/// The byte at `index`, NUL past the end as in a C string.
fn at(encoded: &[u8], index: usize) -> u32 {
    u32::from(CODES[usize::from(encoded.get(index).copied().unwrap_or(0))])
}

/// `aNbIter` (bytes of complete groups) and `aPad` (bytes decoded from the last group).
fn decoded_layout(encoded: &[u8]) -> (usize, usize) {
    let length = encoded.len();
    let mut pad = usize::from(!length.is_multiple_of(4) || encoded[length - 1] == b'=');
    let iterations = (length.div_ceil(4) - pad) * 4;
    if length > iterations + 2 && encoded[iterations + 2] != b'=' {
        pad += 1;
    }
    (iterations, pad)
}

fn decode_groups(output: &mut [u8], encoded: &[u8], iterations: usize, pad: usize) {
    let mut written = 0;
    for start in (0..iterations).step_by(4) {
        let word = (at(encoded, start) << 18)
            + (at(encoded, start + 1) << 12)
            + (at(encoded, start + 2) << 6)
            + at(encoded, start + 3);
        output[written..written + 3].copy_from_slice(&[
            (word >> 16) as u8,
            (word >> 8) as u8,
            word as u8,
        ]);
        written += 3;
    }
    if pad == 0 {
        return;
    }
    let mut word = (at(encoded, iterations) << 18) + (at(encoded, iterations + 1) << 12);
    output[written] = (word >> 16) as u8;
    if pad > 1 {
        word += at(encoded, iterations + 2) << 6;
        output[written + 1] = (word >> 8) as u8;
    }
}
