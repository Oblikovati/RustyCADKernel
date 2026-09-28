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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_ExtCharacter.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_ExtCharacter: conversions between `char` and the UTF-16 unit `char16_t`.
//!
//! These are byte-level conversions, not Unicode transcoding: a `char` is zero-extended
//! through `unsigned char`, and a UTF-16 unit keeps only its low byte.

/// `ToExtCharacter(char)`: zero-extends the byte, `(char16_t)((unsigned char)c & 0xff)`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::ext_character::to_ext_character(0xE9), 0x00E9);
/// ```
pub const fn to_ext_character(byte: u8) -> u16 {
    byte as u16
}

/// `ToCharacter(char16_t)`: keeps the low byte, `(char)(unsigned char)(c & 0xff)`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::ext_character::to_character(0x20AC), 0xAC);
/// ```
pub const fn to_character(unit: u16) -> u8 {
    (unit & 0x00ff) as u8
}

/// `IsAnAscii(char16_t)`: true when the high byte is zero, so 128..=255 count as "ASCII".
///
/// ```
/// assert!(rustycad_tkernel::standard::ext_character::is_an_ascii(0x00FF));
/// ```
pub const fn is_an_ascii(unit: u16) -> bool {
    unit & 0xff00 == 0
}

/// `IsEqual(char16_t, char16_t)`: value equality.
///
/// ```
/// assert!(rustycad_tkernel::standard::ext_character::is_equal(0x20AC, 0x20AC));
/// ```
pub const fn is_equal(first: u16, second: u16) -> bool {
    first == second
}
