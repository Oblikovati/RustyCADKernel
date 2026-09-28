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

//! gp_Vec2d operators: Added/Subtracted/Multiplied/Divided and their in-place forms.
use super::Vector2;
use std::ops;
impl ops::Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self::from(self.coordinates + other.coordinates)
    }
}
impl ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self::from(self.coordinates - other.coordinates)
    }
}
impl ops::Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self::from(self.coordinates * scalar)
    }
}
impl ops::Mul<Vector2> for f64 {
    type Output = Vector2;
    fn mul(self, vector: Vector2) -> Vector2 {
        vector * self
    }
}
/// `gp_Vec2d::operator*(gp_Vec2d)` is the scalar product.
impl ops::Mul for Vector2 {
    type Output = f64;
    fn mul(self, other: Self) -> f64 {
        self.dot(other)
    }
}
impl ops::Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::from(self.coordinates / scalar)
    }
}
/// `gp_Vec2d::operator^` is the signed planar cross product.
impl ops::BitXor for Vector2 {
    type Output = f64;
    fn bitxor(self, other: Self) -> f64 {
        self.crossed(other)
    }
}
impl ops::Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self {
        self.reversed()
    }
}
impl ops::AddAssign for Vector2 {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}
impl ops::SubAssign for Vector2 {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}
impl ops::MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, scalar: f64) {
        *self = *self * scalar;
    }
}
impl ops::DivAssign<f64> for Vector2 {
    fn div_assign(&mut self, scalar: f64) {
        *self = *self / scalar;
    }
}
