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

//! gp_Vec metrics, products, normalization and angular predicates.
use super::super::{Direction3, Failure, Vector3, construction, vector_with_null_magnitude};
use std::f64::consts::{FRAC_PI_2, PI};
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
    /// Replaces this vector with its vector product; `vector.cross(other)`.
    pub fn cross(&mut self, other: Self) {
        *self = self.crossed(other);
    }
    /// Computes cross-product magnitude; `vector.cross_magnitude(other)`.
    pub fn cross_magnitude(self, other: Self) -> f64 {
        self.coordinates.cross_magnitude(other.coordinates)
    }
    /// Computes cross-product squared magnitude; `vector.cross_square_magnitude(other)`.
    pub fn cross_square_magnitude(self, other: Self) -> f64 {
        self.coordinates.cross_square_magnitude(other.coordinates)
    }
    /// Computes `self ^ (first ^ second)`; `vector.cross_crossed(first,second)`.
    pub fn cross_crossed(self, first: Self, second: Self) -> Self {
        Self::from(
            self.coordinates
                .cross_crossed(first.coordinates, second.coordinates),
        )
    }
    /// Replaces this vector with `self ^ (first ^ second)`; `vector.cross_cross(first,second)`.
    pub fn cross_cross(&mut self, first: Self, second: Self) {
        *self = self.cross_crossed(first, second);
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
            return Err(construction(&format!(
                "{message}: ({}, {}, {})",
                self.x(),
                self.y(),
                self.z()
            )));
        }
        Ok(self / magnitude)
    }
    /// Computes the direction angle after native magnitude checks; `vector.angle(other)?`.
    pub fn angle(self, other: Self) -> Result<f64, Failure> {
        let magnitudes = [self.magnitude(), other.magnitude()];
        if magnitudes.iter().any(|&m| m <= f64::MIN_POSITIVE) {
            return Err(null_magnitude("Angle", &magnitudes));
        }
        Ok(Direction3::from_vector(self)?.angle(&Direction3::from_vector(other)?))
    }
    /// Signs the direction angle by a reference vector after gp_Vec's three magnitude checks;
    /// `vector.angle_with_ref(other,reference)?`.
    pub fn angle_with_ref(self, other: Self, reference: Self) -> Result<f64, Failure> {
        let magnitudes = [self.magnitude(), reference.magnitude(), other.magnitude()];
        if magnitudes.iter().any(|&m| m <= f64::MIN_POSITIVE) {
            return Err(null_magnitude("AngleWithRef", &magnitudes));
        }
        let this = Direction3::from_vector(self)?;
        Ok(this.angle_with_ref(
            Direction3::from_vector(other)?,
            Direction3::from_vector(reference)?,
        ))
    }
    /// gp_Vec::IsEqual: magnitudes within the linear tolerance, and the angle within the
    /// angular one only when both magnitudes exceed the linear tolerance;
    /// `vector.is_equal(other,1e-7,1e-12)?`.
    pub fn is_equal(self, other: Self, linear: f64, angular: f64) -> Result<bool, Failure> {
        let (magnitude, other_magnitude) = (self.magnitude(), other.magnitude());
        let equal_length = (magnitude - other_magnitude).abs() <= linear;
        if magnitude <= linear || other_magnitude <= linear {
            return Ok(equal_length);
        }
        // `&&` short-circuits in C++, so Angle (which may raise) runs only for equal lengths.
        Ok(equal_length && self.angle(other)? <= angular)
    }
    /// Tests parallel or antiparallel vectors; `vector.is_parallel(other,1e-12)?`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        let angle = self.angle(other)?;
        Ok(angle <= tolerance || PI - angle <= tolerance)
    }
    /// Tests perpendicular vectors; `vector.is_normal(other,1e-12)?`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        Ok((FRAC_PI_2 - self.angle(other)?).abs() <= tolerance)
    }
    /// Tests opposite vectors; `vector.is_opposite(other,1e-12)?`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        Ok(PI - self.angle(other)? <= tolerance)
    }
    /// Reverses signs without changing magnitude; `vector.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from(self.coordinates.reversed())
    }
    /// Reverses this vector; `vector.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
}
/// gp_VectorWithNullMagnitude keeps the native " " message and appends the magnitudes.
fn null_magnitude(member: &str, magnitudes: &[f64]) -> Failure {
    vector_with_null_magnitude(&format!(
        " gp_Vec::{member}() - magnitudes {magnitudes:?} must exceed gp::Resolution()"
    ))
}
