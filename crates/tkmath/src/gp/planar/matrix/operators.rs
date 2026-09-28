// Copyright (c) 1991-1999 Matra Datavision
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

//! gp_Mat2d operators delegating to the named members. gp_Mat2d declares no
//! `operator*=(gp_Mat2d)`, so there is no `MulAssign<Matrix2>`; use `multiply`.
use super::Matrix2;
use std::ops;
impl ops::Add for Matrix2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.added(other)
    }
}
impl ops::Sub for Matrix2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.subtracted(other)
    }
}
impl ops::Mul for Matrix2 {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiplied(other)
    }
}
impl ops::Mul<f64> for Matrix2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        self.multiplied_scalar(scalar)
    }
}
impl ops::Mul<Matrix2> for f64 {
    type Output = Matrix2;
    fn mul(self, matrix: Matrix2) -> Matrix2 {
        matrix.multiplied_scalar(self)
    }
}
impl ops::Div<f64> for Matrix2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        self.divided(scalar)
    }
}
impl ops::AddAssign for Matrix2 {
    fn add_assign(&mut self, other: Self) {
        *self = self.added(other);
    }
}
impl ops::SubAssign for Matrix2 {
    fn sub_assign(&mut self, other: Self) {
        *self = self.subtracted(other);
    }
}
impl ops::MulAssign<f64> for Matrix2 {
    fn mul_assign(&mut self, scalar: f64) {
        self.multiply_scalar(scalar);
    }
}
impl ops::DivAssign<f64> for Matrix2 {
    fn div_assign(&mut self, scalar: f64) {
        self.divide(scalar);
    }
}
