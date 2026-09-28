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
// Rust translation of FoundationClasses/TKMath/gp/gp_Vec2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Vec2d: a planar vector with magnitude.
use super::{
    Direction2, Failure, Point2, Xy, construction, oriented_angle, vector_with_null_magnitude,
};
use std::cmp::Ordering;
mod geometry;
mod operators;
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
    /// Forms the displacement between points; `Vector2::between(first,second)`.
    pub fn between(first: Point2, second: Point2) -> Self {
        Self::from(second.xy() - first.xy())
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
    /// Reads a one-based coordinate; `vector.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.coordinates.coord(index)
    }
    /// Replaces a one-based coordinate; `vector.set_coord(1,2.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        self.coordinates.set_coord(index, value)
    }
    /// Replaces both coordinates; `vector.set_coordinates(1.0,2.0)`.
    pub fn set_coordinates(&mut self, x: f64, y: f64) {
        self.coordinates = Xy::new(x, y);
    }
    /// Replaces X; `vector.set_x(1.0)`.
    pub fn set_x(&mut self, x: f64) {
        self.coordinates.set_x(x);
    }
    /// Replaces Y; `vector.set_y(1.0)`.
    pub fn set_y(&mut self, y: f64) {
        self.coordinates.set_y(y);
    }
    /// Replaces the coordinates from planar algebra; `vector.set_xy(xy)`.
    pub fn set_xy(&mut self, xy: Xy) {
        self.coordinates = xy;
    }
    /// Computes scalar product; `vector.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.coordinates.dot(other.coordinates)
    }
    /// Computes signed cross product; `vector.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.coordinates.crossed(other.coordinates)
    }
    /// Computes the absolute cross product; `vector.cross_magnitude(other)`.
    pub fn cross_magnitude(self, other: Self) -> f64 {
        self.coordinates.cross_magnitude(other.coordinates)
    }
    /// Computes the squared cross product; `vector.cross_square_magnitude(other)`.
    pub fn cross_square_magnitude(self, other: Self) -> f64 {
        self.coordinates.cross_square_magnitude(other.coordinates)
    }
    /// Computes magnitude; `vector.magnitude()`.
    pub fn magnitude(self) -> f64 {
        self.coordinates.modulus()
    }
    /// Computes squared magnitude; `vector.square_magnitude()`.
    pub fn square_magnitude(self) -> f64 {
        self.coordinates.square_modulus()
    }
    /// Returns `(Y, (-1)*X)`; the multiplication, unlike a negation, keeps a NaN's sign;
    /// `vector.normal()`.
    #[allow(clippy::neg_multiply)] // (-1)*X keeps the NaN sign that negation would flip.
    pub fn normal(self) -> Self {
        Self::new(self.y(), -1.0 * self.x())
    }
    /// Normalizes with the copying overload's message; `vector.normalized()?`.
    pub fn normalized(self) -> Result<Self, Failure> {
        self.normalized_with("gp_Vec2d::Normalized() - vector has zero norm")
    }
    /// Normalizes in place, keeping the vector on failure; `vector.normalize()?`.
    pub fn normalize(&mut self) -> Result<(), Failure> {
        *self = self.normalized_with("gp_Vec2d::Normalize() - vector has zero norm")?;
        Ok(())
    }
    fn normalized_with(self, message: &str) -> Result<Self, Failure> {
        let norm = self.magnitude();
        if norm <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "{message}: ({}, {})",
                self.x(),
                self.y()
            )));
        }
        Ok(self / norm)
    }
    /// Reverses signs; `vector.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from(self.coordinates.reversed())
    }
    /// Reverses in place; `vector.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
}
impl Vector2 {
    /// Uses the product of magnitudes and native 45-degree threshold; `vector.angle(other)?`.
    pub fn angle(self, other: Self) -> Result<f64, Failure> {
        let first = self.magnitude();
        let second = other.magnitude();
        if first <= f64::MIN_POSITIVE || second <= f64::MIN_POSITIVE {
            return Err(vector_with_null_magnitude(&format!(
                "gp_Vec2d::Angle() - magnitudes {first} and {second}"
            )));
        }
        let product = first * second;
        Ok(oriented_angle(
            self.dot(other) / product,
            self.crossed(other) / product,
            std::f64::consts::FRAC_1_SQRT_2,
        ))
    }
    /// Compares magnitudes, then the angle only when both exceed the linear tolerance;
    /// `vector.is_equal(other,1e-7,1e-12)?`.
    pub fn is_equal(self, other: Self, linear: f64, angular: f64) -> Result<bool, Failure> {
        let norm = self.magnitude();
        let other_norm = other.magnitude();
        let equal_length = (norm - other_norm).abs() <= linear;
        if norm > linear && other_norm > linear {
            let angle = self.angle(other)?.abs();
            return Ok(equal_length && angle <= angular);
        }
        Ok(equal_length)
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
    /// Sets `a*first + b*second`; `vector.set_linear_form(2.0,first,3.0,second)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        self.coordinates
            .set_linear_form(a, first.coordinates, b, second.coordinates);
    }
    /// Sets `a*first + b*second + third`; `vector.set_linear_form_offset(2.0,a,3.0,b,c)`.
    pub fn set_linear_form_offset(
        &mut self,
        a: f64,
        first: Self,
        b: f64,
        second: Self,
        third: Self,
    ) {
        let (first, second, third) = (first.coordinates, second.coordinates, third.coordinates);
        self.coordinates
            .set_linear_form_offset(a, first, b, second, third);
    }
    /// Sets `a*first + second`; `vector.set_scaled_sum(2.0,a,b)`.
    pub fn set_scaled_sum(&mut self, a: f64, first: Self, second: Self) {
        self.coordinates
            .set_scaled_sum(a, first.coordinates, second.coordinates);
    }
    /// Sets `first + second`; `vector.set_sum(a,b)`.
    pub fn set_sum(&mut self, first: Self, second: Self) {
        self.coordinates
            .set_sum(first.coordinates, second.coordinates);
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
