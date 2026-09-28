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

//! gp_Dir coordinate access, the renormalizing setters and the `-`/`*` operators.
//!
//! Every native setter funnels through `SetCoord(x,y,z)`, which renormalizes and raises
//! Standard_ConstructionError before storing anything, so a failed set leaves the direction
//! unchanged. `^` and `^=` may raise too, so they map to [`Direction3::crossed`] and
//! [`Direction3::cross`] instead of infallible operator traits.
use super::super::{Direction3, Failure, Xyz};
use std::ops;
impl Direction3 {
    /// Returns the unit coordinates as algebra (gp_Dir::XYZ); `direction.xyz()`.
    pub fn xyz(&self) -> Xyz {
        Xyz::from(*self)
    }
    /// Reads a one-based coordinate, raising Standard_OutOfRange outside 1..=3;
    /// `direction.coord(3)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.xyz().coord(index)
    }
    /// Replaces all components and renormalizes; `direction.set_coordinates(0.0,0.0,2.0)?`.
    pub fn set_coordinates(&mut self, x: f64, y: f64, z: f64) -> Result<(), Failure> {
        let message = format!("gp_Dir::SetCoord() - input vector has zero norm: ({x}, {y}, {z})");
        *self = Self::normalize([x, y, z], &message)?;
        Ok(())
    }
    /// Replaces a one-based component and renormalizes; the index is checked first, as the
    /// native Standard_OutOfRange raise precedes the ConstructionError one;
    /// `direction.set_coord(1,2.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        let mut coordinates = self.xyz();
        coordinates.set_coord(index, value)?;
        self.set_xyz(coordinates)
    }
    /// Replaces X and renormalizes; `direction.set_x(1.0)?`.
    pub fn set_x(&mut self, x: f64) -> Result<(), Failure> {
        self.set_coordinates(x, self.y(), self.z())
    }
    /// Replaces Y and renormalizes; `direction.set_y(1.0)?`.
    pub fn set_y(&mut self, y: f64) -> Result<(), Failure> {
        self.set_coordinates(self.x(), y, self.z())
    }
    /// Replaces Z and renormalizes; `direction.set_z(1.0)?`.
    pub fn set_z(&mut self, z: f64) -> Result<(), Failure> {
        self.set_coordinates(self.x(), self.y(), z)
    }
    /// Replaces every component from algebra and renormalizes; `direction.set_xyz(xyz)?`.
    pub fn set_xyz(&mut self, xyz: Xyz) -> Result<(), Failure> {
        self.set_coordinates(xyz.x(), xyz.y(), xyz.z())
    }
}
impl ops::Neg for Direction3 {
    type Output = Self;
    fn neg(self) -> Self {
        self.reversed()
    }
}
impl ops::Mul for Direction3 {
    type Output = f64;
    fn mul(self, other: Self) -> f64 {
        self.dot(other)
    }
}
