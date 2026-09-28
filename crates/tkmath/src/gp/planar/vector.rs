// Copyright (c) 1995-1999 Matra Datavision
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

use super::{Direction2, Failure, Point2, Xy, construction, oriented_angle};
use std::{cmp::Ordering, ops};
/// A planar vector retaining magnitude; `Vector2::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector2 {
    coordinates: Xy,
}
impl Vector2 {
    /// Stores unnormalized components; `Vector2::new(1.0,2.0)`.
    pub const fn new(x: f64, y: f64) -> Self {
        Self {
            coordinates: Xy::new(x, y),
        }
    }
    /// Returns component algebra; `vector.xy()`.
    pub const fn xy(&self) -> Xy {
        self.coordinates
    }
    /// Reads X; `vector.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `vector.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Forms the displacement between points; `Vector2::between(first,second)`.
    pub fn between(first: Point2, second: Point2) -> Self {
        Self::from(second.xy() - first.xy())
    }
    /// Computes scalar product; `vector.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.coordinates.dot(other.coordinates)
    }
    /// Computes signed cross product; `vector.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.coordinates.crossed(other.coordinates)
    }
    /// Computes magnitude; `vector.magnitude()`.
    pub fn magnitude(self) -> f64 {
        self.coordinates.modulus()
    }
    /// Computes squared magnitude; `vector.square_magnitude()`.
    pub fn square_magnitude(self) -> f64 {
        self.coordinates.square_modulus()
    }
    /// Normalizes with native error category and message; `vector.normalized()?`.
    pub fn normalized(self) -> Result<Self, Failure> {
        let norm = self.magnitude();
        if norm <= f64::MIN_POSITIVE {
            return Err(construction(
                "gp_Vec2d::Normalized() - vector has zero norm",
            ));
        }
        Ok(self / norm)
    }
    /// Uses the product of magnitudes and native 45-degree threshold; `vector.angle(other)?`.
    pub fn angle(self, other: Self) -> Result<f64, Failure> {
        let first = self.magnitude();
        let second = other.magnitude();
        if first <= f64::MIN_POSITIVE || second <= f64::MIN_POSITIVE {
            return Err(Failure::with_type("gp_VectorWithNullMagnitude", "", ""));
        }
        let product = first * second;
        Ok(oriented_angle(
            self.dot(other) / product,
            self.crossed(other) / product,
            std::f64::consts::FRAC_1_SQRT_2,
        ))
    }
    /// Tests both signs of right angle, including the OCC26750 fix; `vector.is_normal(other,1e-12)?`.
    pub fn is_normal(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        let angle = (std::f64::consts::FRAC_PI_2 - self.angle(other)?.abs()).abs();
        Ok(angle.partial_cmp(&tolerance) != Some(Ordering::Greater))
    }
    /// Tests opposite vectors; `vector.is_opposite(other,1e-12)?`.
    pub fn is_opposite(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        Ok(std::f64::consts::PI - self.angle(other)?.abs() <= tolerance)
    }
    /// Tests parallel or opposite vectors; `vector.is_parallel(other,1e-12)?`.
    pub fn is_parallel(self, other: Self, tolerance: f64) -> Result<bool, Failure> {
        let angle = self.angle(other)?.abs();
        Ok(angle <= tolerance || std::f64::consts::PI - angle <= tolerance)
    }
    /// Reverses signs; `vector.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from(self.coordinates.reversed())
    }
    /// Sets two weighted terms in native order; `vector.set_linear_form(2.0,first,3.0,second)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        self.coordinates = first.coordinates * a + second.coordinates * b;
    }
}
impl From<Xy> for Vector2 {
    fn from(coordinates: Xy) -> Self {
        Self { coordinates }
    }
}
impl From<Direction2> for Vector2 {
    fn from(direction: Direction2) -> Self {
        Self::from(direction.xy())
    }
}
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
impl ops::Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::from(self.coordinates / scalar)
    }
}
