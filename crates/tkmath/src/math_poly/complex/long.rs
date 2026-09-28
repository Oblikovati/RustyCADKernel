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

//! `std::complex<long double>` on software x87: `__mulxc3`, `__divxc3` and `cabsl`.
use super::super::extended::Extended as E;
use super::gcc;
use std::ops::{Add, Div, Mul, Sub};
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct LongComplex {
    pub(crate) re: E,
    pub(crate) im: E,
}
impl LongComplex {
    pub(crate) fn new(re: E, im: E) -> Self {
        Self { re, im }
    }
    /// A real value widened to a complex with a +0 imaginary part.
    pub(crate) fn real(re: E) -> Self {
        Self { re, im: E::ZERO }
    }
    pub(crate) fn abs(self) -> E {
        self.re.hypot(self.im)
    }
    pub(crate) fn finite(self) -> bool {
        self.re.finite() && self.im.finite()
    }
    /// Adds a real, leaving the imaginary part untouched (GCC complex lowering).
    pub(crate) fn plus_real(self, value: E) -> Self {
        Self::new(self.re + value, self.im)
    }
}
impl Mul for LongComplex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let (re, im) = gcc::multiply(self.re, self.im, rhs.re, rhs.im);
        Self::new(re, im)
    }
}
impl Div for LongComplex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let (re, im) = gcc::divide(self.re, self.im, rhs.re, rhs.im);
        Self::new(re, im)
    }
}
impl Add for LongComplex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}
impl Sub for LongComplex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.re - rhs.re, self.im - rhs.im)
    }
}
