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

//! gp_Vec operators: `+ - * / ^` with their compound forms, unary minus and `scalar * vector`.
//! Division by zero follows IEEE arithmetic, as the unchecked native division does.
use super::super::Vector3;
use std::ops;
impl ops::Add for Vector3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::from(self.coordinates + other.coordinates)
    }
}
impl ops::Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::from(self.coordinates - other.coordinates)
    }
}
impl ops::Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::from(self.coordinates * scalar)
    }
}
impl ops::Mul<Vector3> for f64 {
    type Output = Vector3;
    fn mul(self, vector: Vector3) -> Vector3 {
        vector * self
    }
}
impl ops::Mul for Vector3 {
    type Output = f64;
    fn mul(self, other: Self) -> f64 {
        self.dot(other)
    }
}
impl ops::Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::from(self.coordinates / scalar)
    }
}
impl ops::BitXor for Vector3 {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        self.crossed(other)
    }
}
impl ops::Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self {
        self.reversed()
    }
}
impl ops::AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl ops::SubAssign for Vector3 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl ops::MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar;
    }
}
impl ops::DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, scalar: f64) {
        *self = *self / scalar;
    }
}
impl ops::BitXorAssign for Vector3 {
    fn bitxor_assign(&mut self, other: Self) {
        self.cross(other);
    }
}
