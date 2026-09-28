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

use std::ops;
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
    /// Borrows the coordinate pair; `xy.coordinates()`.
    pub const fn coordinates(&self) -> &[f64; 2] {
        &self.coordinates
    }
    /// Computes scalar product with no added zero accumulator; `xy.dot(other)`.
    pub fn dot(self, other: Self) -> f64 {
        self.x() * other.x() + self.y() * other.y()
    }
    /// Computes the signed planar cross product; `xy.crossed(other)`.
    pub fn crossed(self, other: Self) -> f64 {
        self.x() * other.y() - self.y() * other.x()
    }
    /// Computes squared norm; `xy.square_modulus()`.
    pub fn square_modulus(self) -> f64 {
        self.dot(self)
    }
    /// Computes norm; `xy.modulus()`.
    pub fn modulus(self) -> f64 {
        self.square_modulus().sqrt()
    }
    /// Reverses the signs; `xy.reversed()`.
    pub fn reversed(self) -> Self {
        Self::new(-self.x(), -self.y())
    }
    pub(super) fn multiplied_matrix(self, matrix: [[f64; 2]; 2]) -> Self {
        Self::new(
            matrix[0][0] * self.x() + matrix[0][1] * self.y(),
            matrix[1][0] * self.x() + matrix[1][1] * self.y(),
        )
    }
}
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
impl ops::Div<f64> for Xy {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self::new(self.x() / scalar, self.y() / scalar)
    }
}
