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

use super::super::{Direction3, Failure, Vector3, construction};
use std::ops;
impl Vector3 {
    /// Computes Euclidean magnitude; `vector.magnitude()`.
    pub fn magnitude(&self) -> f64 {
        self.coordinates.modulus()
    }
    /// Computes squared magnitude; `vector.square_magnitude()`.
    pub fn square_magnitude(&self) -> f64 {
        self.coordinates.square_modulus()
    }
    /// Computes scalar product; `vector.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.coordinates.dot(other.coordinates)
    }
    /// Computes vector product without normalization; `vector.crossed(other)`.
    pub fn crossed(self, other: Self) -> Self {
        Self::from(self.coordinates.crossed(other.coordinates))
    }
    /// Computes cross-product magnitude; `vector.cross_magnitude(other)`.
    pub fn cross_magnitude(self, other: Self) -> f64 {
        self.coordinates.cross_magnitude(other.coordinates)
    }
    /// Computes scalar triple product; `vector.dot_cross(first,second)`.
    pub fn dot_cross(self, first: Self, second: Self) -> f64 {
        self.coordinates
            .dot_cross(first.coordinates, second.coordinates)
    }
    /// Normalizes a copy with the native exception message; `vector.normalized()?`.
    pub fn normalized(self) -> Result<Self, Failure> {
        self.normalize_with_message("gp_Vec::Normalized() - vector has zero norm")
    }
    /// Normalizes in place, preserving values on failure; `vector.normalize()?`.
    pub fn normalize(&mut self) -> Result<(), Failure> {
        *self = self.normalize_with_message("gp_Vec::Normalize() - vector has zero norm")?;
        Ok(())
    }
    fn normalize_with_message(self, message: &str) -> Result<Self, Failure> {
        let magnitude = self.magnitude();
        if magnitude <= f64::MIN_POSITIVE {
            return Err(construction(message));
        }
        Ok(self / magnitude)
    }
    /// Computes the direction angle after native magnitude checks; `vector.angle(other)?`.
    pub fn angle(self, other: Self) -> Result<f64, Failure> {
        if self.magnitude() <= f64::MIN_POSITIVE || other.magnitude() <= f64::MIN_POSITIVE {
            return Err(null_magnitude());
        }
        Ok(Direction3::from_vector(self)?.angle(&Direction3::from_vector(other)?))
    }
    /// Signs the direction angle by a reference vector after gp_Vec's three magnitude checks;
    /// `vector.angle_with_ref(other,reference)?`.
    pub fn angle_with_ref(self, other: Self, reference: Self) -> Result<f64, Failure> {
        let resolution = f64::MIN_POSITIVE;
        if self.magnitude() <= resolution
            || reference.magnitude() <= resolution
            || other.magnitude() <= resolution
        {
            return Err(null_magnitude());
        }
        let this = Direction3::from_vector(self)?;
        Ok(this.angle_with_ref(
            Direction3::from_vector(other)?,
            Direction3::from_vector(reference)?,
        ))
    }
    /// Tests parallel or antiparallel vectors; `vector.is_parallel(other,1e-12)?`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        let angle = self.angle(other)?;
        Ok(angle <= tolerance || std::f64::consts::PI - angle <= tolerance)
    }
    /// Tests perpendicular vectors; `vector.is_normal(other,1e-12)?`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        Ok((std::f64::consts::FRAC_PI_2 - self.angle(other)?).abs() <= tolerance)
    }
    /// Tests opposite vectors; `vector.is_opposite(other,1e-12)?`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        Ok(std::f64::consts::PI - self.angle(other)? <= tolerance)
    }
    /// Reverses signs without changing magnitude; `vector.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from(self.coordinates.reversed())
    }
    /// Reverses this vector; `vector.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
    /// Sets two weighted vector terms; `vector.set_linear_form(2.0,first,3.0,second)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        self.coordinates
            .set_linear_form(a, first.coordinates, b, second.coordinates);
    }
}
fn null_magnitude() -> Failure {
    Failure::with_type("gp_VectorWithNullMagnitude", " ", "")
}
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
impl ops::Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::from(self.coordinates / scalar)
    }
}
