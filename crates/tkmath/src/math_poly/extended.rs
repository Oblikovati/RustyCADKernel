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

//! Software x87 arithmetic for the pinned Linux/GCC polynomial oracle.
use rustc_apfloat::ieee::{Double, X87DoubleExtended};
use rustc_apfloat::{Float, FloatConvert, Round};
use std::ops::{Add, Div, Mul, Neg, Sub};
#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) struct Extended(X87DoubleExtended);
mod sqrt;
impl Extended {
    pub(crate) fn floor(self) -> Self {
        Self(self.0.round_to_integral(Round::TowardNegative).value)
    }
    pub(crate) fn ceil(self) -> Self {
        Self(self.0.round_to_integral(Round::TowardPositive).value)
    }
    pub(crate) fn from_usize(value: usize) -> Self {
        Self(X87DoubleExtended::from_u128(value as u128).value)
    }
    pub(crate) fn to_usize(self) -> usize {
        self.0.to_u128(usize::BITS as usize).value as usize
    }

    pub(crate) const ZERO: Self = Self(X87DoubleExtended::ZERO);
    pub(crate) fn new(value: f64) -> Self {
        Self(
            Double::from_bits(value.to_bits() as u128)
                .convert(&mut false)
                .value,
        )
    }
    pub(crate) fn abs(self) -> Self {
        Self(self.0.abs())
    }
    pub(crate) fn finite(self) -> bool {
        !self.0.is_infinite() && !self.0.is_nan()
    }
    pub(crate) fn max(self, other: Self) -> Self {
        if self < other { other } else { self }
    }
    pub(crate) fn scale(self, exponent: i32) -> Self {
        Self(self.0.scalbn(exponent))
    }
    pub(crate) fn ceil_integer(self) -> i32 {
        self.0
            .round_to_integral(Round::TowardPositive)
            .value
            .to_i128(32)
            .value as i32
    }
    pub(crate) fn double(self) -> Option<f64> {
        if !self.finite() {
            return None;
        }
        let double: Double = self.0.convert(&mut false).value;
        let value: f64 = f64::from_bits(double.to_bits() as u64);
        value.is_finite().then_some(value)
    }
}
impl Add for Extended {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self((self.0 + rhs.0).value)
    }
}
impl Sub for Extended {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self((self.0 - rhs.0).value)
    }
}
impl Mul for Extended {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self((self.0 * rhs.0).value)
    }
}
impl Div for Extended {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        Self((self.0 / rhs.0).value)
    }
}
impl Neg for Extended {
    type Output = Self;
    fn neg(self) -> Self {
        Self(-self.0)
    }
}
pub(crate) fn exponent(value: f64) -> i32 {
    Double::from_bits(value.to_bits() as u128).ilogb()
}
pub(crate) fn scale_double(value: f64, power: i32) -> f64 {
    f64::from_bits(
        Double::from_bits(value.to_bits() as u128)
            .scalbn(power)
            .to_bits() as u64,
    )
}
