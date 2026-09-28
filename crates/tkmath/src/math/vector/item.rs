// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

//! Element contract for `math_VectorBase<TheItemType>` instantiations.
use rustycad_tkernel::tcollection::AsciiString;
use std::fmt::Debug;

/// Arithmetic of one `math_VectorBase` element type (`double` or `int`).
///
/// The methods spell out the C++ expression semantics the template relies on:
/// same-type arithmetic stays in the item type, mixed `double` expressions are
/// evaluated in `double` and converted back with the C++ implicit conversion
/// (truncation toward zero for `int`). Integer arithmetic wraps where C++ has
/// undefined overflow so that debug builds do not abort on native-UB inputs.
/// `assert_eq!(<i32 as VectorItem>::from_real(-2.7), -2)`.
pub trait VectorItem: Copy + Debug + Default + PartialOrd {
    /// The literal `0` used to seed native sums.
    const ZERO: Self;
    /// Native `static_cast<double>(item)`; `3_i32.to_real()`.
    fn to_real(self) -> f64;
    /// Native implicit `double -> TheItemType` conversion; `i32::from_real(2.9)`.
    fn from_real(value: f64) -> Self;
    /// Native `item + item`; `1_i32.item_add(2)`.
    fn item_add(self, right: Self) -> Self;
    /// Native `item - item`; `1_i32.item_sub(2)`.
    fn item_sub(self, right: Self) -> Self;
    /// Native `item * item`; `2_i32.item_mul(3)`.
    fn item_mul(self, right: Self) -> Self;
    /// Native `item / item`; callers reject zero divisors first; `7_i32.item_div(2)`.
    fn item_div(self, right: Self) -> Self;
    /// Native unary minus; `2_i32.item_neg()`.
    fn item_neg(self) -> Self;
    /// Text produced by `Standard_OStream << item` with default flags; `1.5.stream_text()`.
    fn stream_text(self) -> String;
}

impl VectorItem for f64 {
    const ZERO: Self = 0.0;
    fn to_real(self) -> f64 {
        self
    }
    fn from_real(value: f64) -> Self {
        value
    }
    fn item_add(self, right: Self) -> Self {
        self + right
    }
    fn item_sub(self, right: Self) -> Self {
        self - right
    }
    fn item_mul(self, right: Self) -> Self {
        self * right
    }
    fn item_div(self, right: Self) -> Self {
        self / right
    }
    fn item_neg(self) -> Self {
        -self
    }
    fn stream_text(self) -> String {
        stream_real(self)
    }
}

impl VectorItem for i32 {
    const ZERO: Self = 0;
    fn to_real(self) -> f64 {
        f64::from(self)
    }
    fn from_real(value: f64) -> Self {
        // `as` truncates toward zero like the C++ conversion; out-of-range values are
        // undefined natively and saturate here.
        value as i32
    }
    fn item_add(self, right: Self) -> Self {
        self.wrapping_add(right)
    }
    fn item_sub(self, right: Self) -> Self {
        self.wrapping_sub(right)
    }
    fn item_mul(self, right: Self) -> Self {
        self.wrapping_mul(right)
    }
    fn item_div(self, right: Self) -> Self {
        self.wrapping_div(right)
    }
    fn item_neg(self) -> Self {
        self.wrapping_neg()
    }
    fn stream_text(self) -> String {
        self.to_string()
    }
}

/// `Standard_OStream << double` with default flags, which formats like C's `%g`
/// (six significant digits); `assert_eq!(stream_real(0.1), "0.1")`.
pub(crate) fn stream_real(value: f64) -> String {
    // AsciiString(double) uses the same `%g` conversion, so reuse it instead of a second formatter.
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}
