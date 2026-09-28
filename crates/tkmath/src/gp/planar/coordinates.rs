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
// Rust translation of FoundationClasses/TKMath/gp/gp_XY.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_XY planar coordinate algebra in native evaluation order.
use super::{Failure, Matrix2, construction};
mod operators;
/// Two Cartesian scalars for planar algebra; `Xy::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Xy {
    pub(super) coordinates: [f64; 2],
}
impl Xy {
    /// Stores two unmodified coordinates; `Xy::new(1.0,2.0)`.
    pub const fn new(x: f64, y: f64) -> Self {
        Self {
            coordinates: [x, y],
        }
    }
    /// Reads X; `xy.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates[0]
    }
    /// Reads Y; `xy.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates[1]
    }
    /// Borrows the coordinate pair (gp_XY::Coord(double&,double&)); `xy.coordinates()`.
    pub const fn coordinates(&self) -> &[f64; 2] {
        &self.coordinates
    }
    /// Replaces X; `xy.set_x(1.0)`.
    pub const fn set_x(&mut self, x: f64) {
        self.coordinates[0] = x;
    }
    /// Replaces Y; `xy.set_y(1.0)`.
    pub const fn set_y(&mut self, y: f64) {
        self.coordinates[1] = y;
    }
    /// Replaces both coordinates (gp_XY::SetCoord(double,double)); `xy.set_coordinates(1.0,2.0)`.
    pub const fn set_coordinates(&mut self, x: f64, y: f64) {
        self.coordinates = [x, y];
    }
    /// Reads a one-based coordinate with the header's range check; `xy.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        Ok(self.coordinates[coordinate_index(index)?])
    }
    /// Borrows a writable one-based coordinate; `*xy.change_coord(2)?=5.0`.
    pub fn change_coord(&mut self, index: i32) -> Result<&mut f64, Failure> {
        Ok(&mut self.coordinates[coordinate_index(index)?])
    }
    /// Sets a one-based coordinate; `xy.set_coord(2,5.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        *self.change_coord(index)? = value;
        Ok(())
    }
    /// Computes the norm; `xy.modulus()`.
    pub fn modulus(self) -> f64 {
        self.square_modulus().sqrt()
    }
    /// Computes the squared norm; `xy.square_modulus()`.
    pub fn square_modulus(self) -> f64 {
        self.x() * self.x() + self.y() * self.y()
    }
    /// Uses strict per-coordinate tolerances, so zero tolerance is always false;
    /// `xy.is_equal(&other,1e-7)`.
    pub fn is_equal(&self, other: &Self, tolerance: f64) -> bool {
        (self.x() - other.x()).abs() < tolerance && (self.y() - other.y()).abs() < tolerance
    }
    /// Computes scalar product with no added zero accumulator; `xy.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.x() * other.x() + self.y() * other.y()
    }
    /// Computes the signed planar cross product; `xy.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.x() * other.y() - self.y() * other.x()
    }
    /// Computes the absolute planar cross product; `xy.cross_magnitude(other)`.
    pub fn cross_magnitude(self, other: Self) -> f64 {
        self.crossed(other).abs()
    }
    /// Computes the squared planar cross product; `xy.cross_square_magnitude(other)`.
    pub fn cross_square_magnitude(self, other: Self) -> f64 {
        let cross = self.crossed(other);
        cross * cross
    }
    /// Multiplies coordinate pairs (gp_XY::Multiplied(gp_XY)); `xy.multiplied_components(other)`.
    pub fn multiplied_components(self, other: Self) -> Self {
        Self::new(self.x() * other.x(), self.y() * other.y())
    }
    /// Multiplies coordinate pairs in place (gp_XY::Multiply(gp_XY)); `xy.multiply_components(other)`.
    pub fn multiply_components(&mut self, other: Self) {
        *self = self.multiplied_components(other);
    }
    /// Applies a matrix to this column (gp_XY::Multiplied(gp_Mat2d)); `xy.multiplied_matrix(matrix)`.
    pub fn multiplied_matrix(self, matrix: Matrix2) -> Self {
        let m = matrix.coefficients;
        Self::new(
            m[0][0] * self.x() + m[0][1] * self.y(),
            m[1][0] * self.x() + m[1][1] * self.y(),
        )
    }
    /// Applies a matrix in place (gp_XY::Multiply(gp_Mat2d)); `xy.multiply_matrix(matrix)`.
    pub fn multiply_matrix(&mut self, matrix: Matrix2) {
        *self = self.multiplied_matrix(matrix);
    }
    /// Divides by the norm with the copying overload's message; `xy.normalized()?`.
    pub fn normalized(self) -> Result<Self, Failure> {
        self.normalized_with("gp_XY::Normalized() - vector has zero norm")
    }
    /// Normalizes in place, preserving the pair on failure; `xy.normalize()?`.
    pub fn normalize(&mut self) -> Result<(), Failure> {
        *self = self.normalized_with("gp_XY::Normalize() - vector has zero norm")?;
        Ok(())
    }
    fn normalized_with(self, message: &str) -> Result<Self, Failure> {
        let norm = self.modulus();
        if norm <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "{message}: ({}, {}) has norm {norm}",
                self.x(),
                self.y()
            )));
        }
        Ok(self / norm)
    }
    /// Reverses the signs; `xy.reversed()`.
    pub fn reversed(self) -> Self {
        Self::new(-self.x(), -self.y())
    }
    /// Reverses the signs in place; `xy.reverse()`.
    pub fn reverse(&mut self) {
        *self = self.reversed();
    }
}
impl Xy {
    /// Sets `a*first + b*second` without a zero accumulator; `xy.set_linear_form(2.0,a,3.0,b)`.
    pub fn set_linear_form(&mut self, a: f64, first: Self, b: f64, second: Self) {
        self.coordinates = [
            a * first.x() + b * second.x(),
            a * first.y() + b * second.y(),
        ];
    }
    /// Sets `a*first + b*second + third`, summed left to right;
    /// `xy.set_linear_form_offset(2.0,a,3.0,b,c)`.
    pub fn set_linear_form_offset(
        &mut self,
        a: f64,
        first: Self,
        b: f64,
        second: Self,
        third: Self,
    ) {
        self.coordinates = [
            a * first.x() + b * second.x() + third.x(),
            a * first.y() + b * second.y() + third.y(),
        ];
    }
    /// Sets `a*first + second`; `xy.set_scaled_sum(2.0,a,b)`.
    pub fn set_scaled_sum(&mut self, a: f64, first: Self, second: Self) {
        self.coordinates = [a * first.x() + second.x(), a * first.y() + second.y()];
    }
    /// Sets `first + second`; `xy.set_sum(a,b)`.
    pub fn set_sum(&mut self, first: Self, second: Self) {
        self.coordinates = [first.x() + second.x(), first.y() + second.y()];
    }
}
fn coordinate_index(index: i32) -> Result<usize, Failure> {
    if !(1..=2).contains(&index) {
        return Err(Failure::out_of_range(&format!(
            "gp_XY - coordinate index {index} outside [1, 2]"
        )));
    }
    Ok(index as usize - 1)
}
