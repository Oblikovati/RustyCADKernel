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

//! OCCT `Standard_GUID` and `Standard_UUID`.
use crate::{Error, ErrorKind};

/// Number of ASCII characters in a GUID representation.
pub const GUID_SIZE: usize = 36;
/// Number of ASCII/UTF-16 code units including the NUL terminator.
pub const GUID_SIZE_ALLOC: usize = GUID_SIZE + 1;

/// Field layout of `Standard_UUID` (also usable in const contexts).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct Uuid {
    /// First 32 bits.
    pub data1: u32,
    /// Next 16 bits.
    pub data2: u16,
    /// Next 16 bits.
    pub data3: u16,
    /// Last eight bytes in displayed order.
    pub data4: [u8; 8],
}

/// A GUID with OCCT's strict 8-4-4-4-12 hexadecimal text format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Guid(Uuid);

impl Guid {
    /// Constructs the all-zero GUID.
    pub const fn new() -> Self {
        Self(Uuid {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8],
        })
    }

    /// Checks OCCT's ASCII format, including null pointers (`None`) and C string termination.
    pub fn check_format(text: Option<&[u8]>) -> bool {
        let Some(text) = text else {
            return false;
        };
        let text = &text[..text.iter().position(|&c| c == 0).unwrap_or(text.len())];
        text.len() == GUID_SIZE
            && text.iter().enumerate().all(|(i, byte)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    *byte == b'-'
                } else {
                    byte.is_ascii_hexdigit()
                }
            })
    }

    /// Constructs from ASCII bytes, applying OCCT's C-string semantics.
    pub fn from_ascii(text: &[u8]) -> Result<Self, Error> {
        if !Self::check_format(Some(text)) {
            return Err(Error::new(ErrorKind::Range, "Invalid format of GUID"));
        }
        let mut bytes = [0; 16];
        let mut nibble = 0;
        for &ch in &text[..GUID_SIZE] {
            if ch == b'-' {
                continue;
            }
            let value = if ch.is_ascii_digit() {
                ch - b'0'
            } else {
                ch.to_ascii_lowercase() - b'a' + 10
            };
            bytes[nibble / 2] = bytes[nibble / 2] * 16 + value;
            nibble += 1;
        }
        Ok(Self(Uuid {
            data1: u32::from_be_bytes(bytes[0..4].try_into().expect("fixed slice")),
            data2: u16::from_be_bytes(bytes[4..6].try_into().expect("fixed slice")),
            data3: u16::from_be_bytes(bytes[6..8].try_into().expect("fixed slice")),
            data4: bytes[8..16].try_into().expect("fixed slice"),
        }))
    }
}

impl std::str::FromStr for Guid {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_ascii(s.as_bytes())
    }
}

impl Guid {
    /// Constructs from the ten OCCT components, preserving the first field's bit pattern.
    #[allow(clippy::too_many_arguments)]
    pub const fn from_components(
        a: i32,
        b: u16,
        c: u16,
        d: u16,
        e: u8,
        f: u8,
        g: u8,
        h: u8,
        i: u8,
        j: u8,
    ) -> Self {
        Self(Uuid {
            data1: a as u32,
            data2: b,
            data3: c,
            data4: [(d >> 8) as u8, d as u8, e, f, g, h, i, j],
        })
    }

    /// Constructs from UUID fields.
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    /// Converts to UUID fields.
    pub const fn to_uuid(self) -> Uuid {
        self.0
    }
    /// Tests equality in a const context (Rust's derived `==` is runtime only).
    pub const fn is_same(self, other: Self) -> bool {
        if self.0.data1 != other.0.data1
            || self.0.data2 != other.0.data2
            || self.0.data3 != other.0.data3
        {
            return false;
        }
        let mut i = 0;
        while i < 8 {
            if self.0.data4[i] != other.0.data4[i] {
                return false;
            }
            i += 1;
        }
        true
    }
    /// Tests inequality in a const context.
    pub const fn is_not_same(self, other: Self) -> bool {
        !self.is_same(other)
    }
    /// Copies another GUID (`Assign`).
    pub const fn assign(&mut self, other: Self) {
        *self = other;
    }
    /// Copies UUID fields (`Assign` overload).
    pub const fn assign_uuid(&mut self, uuid: Uuid) {
        self.0 = uuid;
    }

    /// Constructs from UTF-16 using OCCT's low-byte conversion of the first 36 code units.
    /// A shorter slice is rejected safely; OCCT would read beyond its input buffer.
    pub fn from_utf16(text: &[u16]) -> Result<Self, Error> {
        if text.len() < GUID_SIZE {
            return Err(Error::new(ErrorKind::Range, "Invalid format of GUID"));
        }
        let bytes: [u8; GUID_SIZE] = std::array::from_fn(|i| text[i] as u8);
        Self::from_ascii(&bytes)
    }

    /// Returns the lowercase ASCII representation with its NUL terminator (`ToCString`).
    pub fn to_c_string(self) -> [u8; GUID_SIZE_ALLOC] {
        let mut output = [0; GUID_SIZE_ALLOC];
        output[..GUID_SIZE].copy_from_slice(self.to_string().as_bytes());
        output
    }

    /// Returns the UTF-16 representation with its NUL terminator (`ToExtString`).
    pub fn to_ext_string(self) -> [u16; GUID_SIZE_ALLOC] {
        self.to_c_string().map(u16::from)
    }

    /// Writes the text representation (`ShallowDump`).
    pub fn shallow_dump(self, output: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(output, "{self}")
    }
}

impl std::fmt::Display for Guid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let uuid = self.0;
        write!(
            f,
            "{:08x}-{:04x}-{:04x}-{:02x}{:02x}-",
            uuid.data1, uuid.data2, uuid.data3, uuid.data4[0], uuid.data4[1]
        )?;
        for byte in &uuid.data4[2..] {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}
