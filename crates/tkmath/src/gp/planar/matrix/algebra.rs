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

//! gp_Mat2d products, inversion and powers.
use super::super::{Failure, construction};
use super::Matrix2;
impl Matrix2 {
    /// Right-multiplies by another matrix (gp_Mat2d::Multiply(gp_Mat2d)); `matrix.multiply(other)`.
    pub fn multiply(&mut self, other: Self) {
        *self = self.multiplied(other);
    }
    /// Returns `self * other`; `matrix.multiplied(other)`.
    pub fn multiplied(self, other: Self) -> Self {
        let (a, b) = (self.coefficients, other.coefficients);
        Self::new([
            [
                a[0][0] * b[0][0] + a[0][1] * b[1][0],
                a[0][0] * b[0][1] + a[0][1] * b[1][1],
            ],
            [
                a[1][0] * b[0][0] + a[1][1] * b[1][0],
                a[1][0] * b[0][1] + a[1][1] * b[1][1],
            ],
        ])
    }
    /// Left-multiplies by another matrix, `self = other * self`; `matrix.pre_multiply(other)`.
    pub fn pre_multiply(&mut self, other: Self) {
        *self = other.multiplied(*self);
    }
    /// Reproduces `M.Multiply(M)` on one object: gp_Mat2d::Multiply stores the new `[0][1]`
    /// before it reads the operand's `[0][1]` for `[1][1]`, so a self-product mixes the new
    /// and the old coefficient. gp_Mat2d/gp_Trsf2d/gp_GTrsf2d::Power square this way;
    /// `matrix.square_in_place_aliased()`.
    pub(in crate::gp) fn square_in_place_aliased(&mut self) {
        let m = self.coefficients;
        let first_column = [
            m[0][0] * m[0][0] + m[0][1] * m[1][0],
            m[1][0] * m[0][0] + m[1][1] * m[1][0],
        ];
        let corner = m[0][0] * m[0][1] + m[0][1] * m[1][1];
        let last = m[1][0] * corner + m[1][1] * m[1][1];
        self.coefficients = [[first_column[0], corner], [first_column[1], last]];
    }
    /// Inverts with the adjugate; the zero-determinant raise exists only under
    /// `occt-debug-checks`, so a release build returns infinities or NaN; `matrix.inverted()?`.
    pub fn inverted(self) -> Result<Self, Failure> {
        let m = self.coefficients;
        let adjugate = [[m[1][1], -m[0][1]], [-m[1][0], m[0][0]]];
        let determinant = adjugate[0][0] * adjugate[1][1] - adjugate[0][1] * adjugate[1][0];
        if cfg!(feature = "occt-debug-checks") && determinant.abs() <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Mat2d::Invert() - matrix has zero determinant {determinant}"
            )));
        }
        let reciprocal = 1.0 / determinant;
        Ok(Self::new(adjugate.map(|row| row.map(|v| v * reciprocal))))
    }
    /// Inverts in place, keeping the matrix on a checked failure; `matrix.invert()?`.
    pub fn invert(&mut self) -> Result<(), Failure> {
        *self = self.inverted()?;
        Ok(())
    }
    /// Raises to an integer power with the native aliased squaring; `matrix.powered(3)?`.
    pub fn powered(mut self, exponent: i32) -> Result<Self, Failure> {
        self.power(exponent)?;
        Ok(self)
    }
    /// gp_Mat2d::Power: 0 is identity, negative powers invert first; `matrix.power(3)?`.
    pub fn power(&mut self, exponent: i32) -> Result<(), Failure> {
        match exponent {
            1 => return Ok(()),
            0 => self.set_identity(),
            -1 => self.invert()?,
            _ => {
                if exponent < 0 {
                    self.invert()?;
                }
                self.power_positive(exponent.wrapping_abs().wrapping_sub(1));
            }
        }
        Ok(())
    }
    fn power_positive(&mut self, mut remaining: i32) {
        let mut factor = *self;
        loop {
            if remaining % 2 == 1 {
                self.multiply(factor);
            }
            if remaining == 1 {
                return;
            }
            factor.square_in_place_aliased();
            remaining /= 2;
        }
    }
    /// Multiplies every coefficient by a scalar; `matrix.multiplied_scalar(2.0)`.
    pub fn multiplied_scalar(self, scalar: f64) -> Self {
        Self::new(self.coefficients.map(|row| row.map(|v| v * scalar)))
    }
    /// Multiplies every coefficient in place; `matrix.multiply_scalar(2.0)`.
    pub fn multiply_scalar(&mut self, scalar: f64) {
        *self = self.multiplied_scalar(scalar);
    }
    /// Divides every coefficient without a zero check, as the inline header does;
    /// `matrix.divided(2.0)`.
    pub fn divided(self, scalar: f64) -> Self {
        Self::new(self.coefficients.map(|row| row.map(|v| v / scalar)))
    }
    /// Divides in place; `matrix.divide(2.0)`.
    pub fn divide(&mut self, scalar: f64) {
        *self = self.divided(scalar);
    }
    /// Adds coefficient-wise; `matrix.added(other)`.
    pub fn added(self, other: Self) -> Self {
        self.zip_with(other, |a, b| a + b)
    }
    /// Subtracts coefficient-wise; `matrix.subtracted(other)`.
    pub fn subtracted(self, other: Self) -> Self {
        self.zip_with(other, |a, b| a - b)
    }
    fn zip_with(self, other: Self, combine: fn(f64, f64) -> f64) -> Self {
        let (a, b) = (self.coefficients, other.coefficients);
        Self::new(std::array::from_fn(|r| {
            std::array::from_fn(|c| combine(a[r][c], b[r][c]))
        }))
    }
}
