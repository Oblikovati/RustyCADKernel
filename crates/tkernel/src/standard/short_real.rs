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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_ShortReal.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_ShortReal: `float` limits and the deprecated `Abs`/`Max`/`Min` overloads.

/// `ShortRealSmall()`: the smallest positive normal float, `FLT_MIN`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_small(), f32::MIN_POSITIVE);
/// ```
pub const fn short_real_small() -> f32 {
    f32::MIN_POSITIVE
}

/// `ShortRealDigits()`: decimal digits of precision, `FLT_DIG`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_digits(), 6);
/// ```
pub const fn short_real_digits() -> i32 {
    f32::DIGITS as i32
}

/// `ShortRealEpsilon()`: the gap between 1.0f and the next float, `FLT_EPSILON`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_epsilon(), f32::EPSILON);
/// ```
pub const fn short_real_epsilon() -> f32 {
    f32::EPSILON
}

/// `ShortRealFirst()`: the most negative finite float, `-FLT_MAX`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_first(), f32::MIN);
/// ```
pub const fn short_real_first() -> f32 {
    -f32::MAX
}

/// `ShortRealFirst10Exp()`: the minimum decimal exponent of a normal float, `FLT_MIN_10_EXP`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_first10_exp(), -37);
/// ```
pub const fn short_real_first10_exp() -> i32 {
    f32::MIN_10_EXP
}

/// `ShortRealLast()`: the largest finite float, `FLT_MAX`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_last(), f32::MAX);
/// ```
pub const fn short_real_last() -> f32 {
    f32::MAX
}

/// `ShortRealLast10Exp()`: the maximum decimal exponent of a float, `FLT_MAX_10_EXP`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_last10_exp(), 38);
/// ```
pub const fn short_real_last10_exp() -> i32 {
    f32::MAX_10_EXP
}

/// `ShortRealMantissa()`: significand bits including the hidden bit, `FLT_MANT_DIG`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_mantissa(), 24);
/// ```
pub const fn short_real_mantissa() -> i32 {
    f32::MANTISSA_DIGITS as i32
}

/// `ShortRealRadix()`: the floating-point radix, `FLT_RADIX`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_radix(), 2);
/// ```
pub const fn short_real_radix() -> i32 {
    f32::RADIX as i32
}

/// `ShortRealSize()`: the size of a float in bits, `CHAR_BIT * sizeof(float)`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::short_real_size(), 32);
/// ```
pub const fn short_real_size() -> i32 {
    (u8::BITS as usize * size_of::<f32>()) as i32
}

/// `Abs(float)`: `std::abs`, which clears the sign bit.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::abs(-1.5), 1.5);
/// ```
pub fn abs(value: f32) -> f32 {
    value.abs()
}

/// `Max(float, float)`: `std::max`, i.e. `first < second ? second : first` (NaN-order sensitive).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::max(1.0, 2.0), 2.0);
/// ```
pub fn max(first: f32, second: f32) -> f32 {
    if first < second { second } else { first }
}

/// `Min(float, float)`: `std::min`, i.e. `second < first ? second : first` (NaN-order sensitive).
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::short_real::min(1.0, 2.0), 1.0);
/// ```
pub fn min(first: f32, second: f32) -> f32 {
    if second < first { second } else { first }
}
