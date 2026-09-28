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
// Rust translation of FoundationClasses/TKernel/Standard/Standard_Integer.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Standard_Integer helpers with C++ truncated-remainder semantics.
//!
//! Where the C++ overflows a signed `int` (undefined behavior), the port takes the
//! two's-complement result that x86-64 produces instead of panicking in debug builds;
//! each such function says so.

/// `IsEven(int)`: `value % 2 == 0`.
///
/// ```
/// assert!(rustycad_tkernel::standard::integer::is_even(-4));
/// ```
pub const fn is_even(value: i32) -> bool {
    value % 2 == 0
}

/// `IsOdd(int)`: `value % 2 == 1`, so negative odd values are neither odd nor even (the
/// truncated remainder is -1), exactly as the native test.
///
/// ```
/// use rustycad_tkernel::standard::integer::is_odd;
/// assert!(is_odd(3) && !is_odd(-3));
/// ```
pub const fn is_odd(value: i32) -> bool {
    value % 2 == 1
}

/// `Abs(int)`: `value < 0 ? -value : value`.
///
/// `-INT_MIN` overflows (undefined behavior in C++); the port wraps to `INT_MIN`, the x86-64 result.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::abs(-7), 7);
/// ```
pub const fn abs(value: i32) -> i32 {
    if value < 0 {
        value.wrapping_neg()
    } else {
        value
    }
}

/// `Max(int, int)`: `std::max`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::max(-1, 2), 2);
/// ```
pub const fn max(first: i32, second: i32) -> i32 {
    if first < second { second } else { first }
}

/// `Min(int, int)`: `std::min`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::min(-1, 2), -1);
/// ```
pub const fn min(first: i32, second: i32) -> i32 {
    if second < first { second } else { first }
}

/// `Modulus(int, int)`: the truncated remainder `value % divisor`, with the dividend's sign.
///
/// A zero divisor and `INT_MIN % -1` trap natively (x86-64 `idiv` raises SIGFPE; both are
/// undefined behavior in C++); the port panics on both, as Rust's `%` does.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::modulus(-7, 3), -1);
/// ```
pub const fn modulus(value: i32, divisor: i32) -> i32 {
    value % divisor
}

/// `Square(int)`: `value * value`.
///
/// The header documents overflow as undefined; the port wraps like x86-64 `imul`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::square(-9), 81);
/// ```
pub const fn square(value: i32) -> i32 {
    value.wrapping_mul(value)
}

/// `IntegerFirst()`: `INT_MIN`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::integer_first(), i32::MIN);
/// ```
pub const fn integer_first() -> i32 {
    i32::MIN
}

/// `IntegerLast()`: `INT_MAX`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::integer_last(), i32::MAX);
/// ```
pub const fn integer_last() -> i32 {
    i32::MAX
}

/// `IntegerSize()`: the size of an `int` in bits, `CHAR_BIT * sizeof(int)`.
///
/// ```
/// assert_eq!(rustycad_tkernel::standard::integer::integer_size(), 32);
/// ```
pub const fn integer_size() -> i32 {
    (u8::BITS as usize * size_of::<i32>()) as i32
}
