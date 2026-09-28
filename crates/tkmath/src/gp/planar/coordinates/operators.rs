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

//! gp_XY operators: Added/Subtracted/Multiplied/Divided and their in-place forms.
use super::super::Matrix2;
use super::Xy;
use std::ops;
impl ops::Add for Xy {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::new(self.x() + other.x(), self.y() + other.y())
    }
}
impl ops::Sub for Xy {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::new(self.x() - other.x(), self.y() - other.y())
    }
}
impl ops::Mul<f64> for Xy {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::new(self.x() * scalar, self.y() * scalar)
    }
}
impl ops::Mul<Xy> for f64 {
    type Output = Xy;
    fn mul(self, xy: Xy) -> Xy {
        xy * self
    }
}
/// `gp_XY::operator*(gp_XY)` is the scalar product.
impl ops::Mul for Xy {
    type Output = f64;
    fn mul(self, other: Self) -> f64 {
        self.dot(other)
    }
}
impl ops::Mul<Matrix2> for Xy {
    type Output = Self;
    fn mul(self, matrix: Matrix2) -> Self {
        self.multiplied_matrix(matrix)
    }
}
impl ops::Mul<Xy> for Matrix2 {
    type Output = Xy;
    fn mul(self, xy: Xy) -> Xy {
        xy.multiplied_matrix(self)
    }
}
impl ops::Div<f64> for Xy {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x() / scalar, self.y() / scalar)
    }
}
/// `gp_XY::operator^` is the signed planar cross product.
impl ops::BitXor for Xy {
    type Output = f64;
    fn bitxor(self, other: Self) -> f64 {
        self.crossed(other)
    }
}
impl ops::Neg for Xy {
    type Output = Self;
    fn neg(self) -> Self {
        self.reversed()
    }
}
impl ops::AddAssign for Xy {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl ops::SubAssign for Xy {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl ops::MulAssign<f64> for Xy {
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar;
    }
}
impl ops::MulAssign for Xy {
    fn mul_assign(&mut self, other: Self) {
        self.multiply_components(other);
    }
}
impl ops::MulAssign<Matrix2> for Xy {
    fn mul_assign(&mut self, matrix: Matrix2) {
        self.multiply_matrix(matrix);
    }
}
impl ops::DivAssign<f64> for Xy {
    fn div_assign(&mut self, scalar: f64) {
        *self = *self / scalar;
    }
}
