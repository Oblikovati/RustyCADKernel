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

//! gp_Quaternion accessors, component algebra, norms and their operators.
use super::Quaternion;
use crate::gp::Vector3;
use std::ops;
impl Quaternion {
    /// Reads X; `quaternion.x()`.
    pub const fn x(&self) -> f64 {
        self.x
    }
    /// Reads Y; `quaternion.y()`.
    pub const fn y(&self) -> f64 {
        self.y
    }
    /// Reads Z; `quaternion.z()`.
    pub const fn z(&self) -> f64 {
        self.z
    }
    /// Reads W; `quaternion.w()`.
    pub const fn w(&self) -> f64 {
        self.w
    }
    /// Copies another quaternion, as `Set(const gp_Quaternion&)`; `quaternion.set_from(other)`.
    pub fn set_from(&mut self, other: Self) {
        *self = other;
    }
    /// Resets to the identity `(0, 0, 0, 1)`, as `SetIdent`; `quaternion.set_ident()`.
    pub fn set_ident(&mut self) {
        *self = Self::default();
    }
    /// Compares every component within gp::Resolution, as `IsEqual`; `a.is_equal(&b)`.
    pub fn is_equal(&self, other: &Self) -> bool {
        let resolution = f64::MIN_POSITIVE;
        (self.x - other.x).abs() <= resolution
            && (self.y - other.y).abs() <= resolution
            && (self.z - other.z).abs() <= resolution
            && (self.w - other.w).abs() <= resolution
    }
    /// Negates the vector part (the conjugate), as `Reversed`; `quaternion.reversed()`.
    pub fn reversed(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, self.w)
    }
    /// Conjugates in place, as `Reverse`; `quaternion.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
    /// Conjugate times the reciprocal squared norm (no zero check), as `Inverted`;
    /// `quaternion.inverted()`.
    pub fn inverted(self) -> Self {
        let inverse = 1.0 / self.square_norm();
        Self::new(
            -self.x * inverse,
            -self.y * inverse,
            -self.z * inverse,
            self.w * inverse,
        )
    }
    /// Inverts in place, as `Invert`; `quaternion.invert()`.
    pub fn invert(&mut self) {
        *self = self.inverted();
    }
    /// Scales every component in place, as `Scale`; `quaternion.scale(2.0)`.
    pub fn scale(&mut self, factor: f64) {
        *self = self.scaled(factor);
    }
    /// Divides by the sum of absolute components, or resets to identity when that sum is
    /// not positive, as `StabilizeLength`; `quaternion.stabilize_length()`.
    pub fn stabilize_length(&mut self) {
        let sum = self.x.abs() + self.y.abs() + self.z.abs() + self.w.abs();
        if sum > 0.0 {
            *self = Self::new(self.x / sum, self.y / sum, self.z / sum, self.w / sum);
            return;
        }
        self.set_ident();
    }
    /// Scales by the reciprocal norm, first stabilizing a norm below gp::Resolution, as
    /// `Normalize`; `quaternion.normalize()`.
    pub fn normalize(&mut self) {
        let mut magnitude = self.norm();
        if magnitude < f64::MIN_POSITIVE {
            self.stabilize_length();
            magnitude = self.norm();
        }
        self.scale(1.0 / magnitude);
    }
    /// Returns a normalized copy, as `Normalized`; `quaternion.normalized()`.
    pub fn normalized(self) -> Self {
        let mut copy = self;
        copy.normalize();
        copy
    }
    /// Negates every component, as `Negated`/unary `operator-`; `quaternion.negated()`.
    pub fn negated(self) -> Self {
        Self::new(-self.x, -self.y, -self.z, -self.w)
    }
    /// Component-wise sum, as `Added`; `a.added(b)`.
    pub fn added(self, other: Self) -> Self {
        Self::new(
            self.x + other.x,
            self.y + other.y,
            self.z + other.z,
            self.w + other.w,
        )
    }
    /// Component-wise difference, as `Subtracted`; `a.subtracted(b)`.
    pub fn subtracted(self, other: Self) -> Self {
        Self::new(
            self.x - other.x,
            self.y - other.y,
            self.z - other.z,
            self.w - other.w,
        )
    }
    /// Hamilton product in the native term order, as `Multiplied`; `a.multiplied(b)`.
    pub fn multiplied(self, q: Self) -> Self {
        let (x, y, z, w) = (self.x, self.y, self.z, self.w);
        Self::new(
            w * q.x + x * q.w + y * q.z - z * q.y,
            w * q.y + y * q.w + z * q.x - x * q.z,
            w * q.z + z * q.w + x * q.y - y * q.x,
            w * q.w - x * q.x - y * q.y - z * q.z,
        )
    }
    /// Right-multiplies in place, as `Multiply(gp_Quaternion)`; `a.multiply(b)`.
    pub fn multiply(&mut self, other: Self) {
        *self = self.multiplied(other);
    }
    /// Four-component scalar product, as `Dot`; `a.dot(b)`.
    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
    }
    /// Rotation angle in `[-2pi, 2pi]` from `atan2` with the sign of W, as `GetRotationAngle`;
    /// `quaternion.rotation_angle()`.
    pub fn rotation_angle(self) -> f64 {
        let length = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if self.w < 0.0 {
            return 2.0 * (-length).atan2(-self.w);
        }
        2.0 * length.atan2(self.w)
    }
}
impl ops::Neg for Quaternion {
    type Output = Self;
    fn neg(self) -> Self {
        self.negated()
    }
}
impl ops::Add for Quaternion {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.added(other)
    }
}
impl ops::Sub for Quaternion {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.subtracted(other)
    }
}
impl ops::Mul for Quaternion {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        self.multiplied(other)
    }
}
impl ops::Mul<f64> for Quaternion {
    type Output = Self;
    fn mul(self, factor: f64) -> Self {
        self.scaled(factor)
    }
}
/// `gp_Quaternion::operator*(gp_Vec)` rotates the vector.
impl ops::Mul<Vector3> for Quaternion {
    type Output = Vector3;
    fn mul(self, vector: Vector3) -> Vector3 {
        self.multiply_vector(vector)
    }
}
impl ops::AddAssign for Quaternion {
    fn add_assign(&mut self, other: Self) {
        *self = self.added(other);
    }
}
impl ops::SubAssign for Quaternion {
    fn sub_assign(&mut self, other: Self) {
        *self = self.subtracted(other);
    }
}
impl ops::MulAssign for Quaternion {
    fn mul_assign(&mut self, other: Self) {
        self.multiply(other);
    }
}
impl ops::MulAssign<f64> for Quaternion {
    fn mul_assign(&mut self, factor: f64) {
        self.scale(factor);
    }
}
