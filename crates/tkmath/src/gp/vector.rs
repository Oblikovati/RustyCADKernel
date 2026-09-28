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

//! gp_Vec Cartesian vectors used by point translations and direction constructors.
use super::{Direction3, Failure, Point3, Xyz};
/// A vector that retains its magnitude; `Vector3::new(1.0,2.0,3.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    coordinates: Xyz,
}
impl Vector3 {
    /// Stores unnormalized components; `Vector3::new(1.0,2.0,3.0)`.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            coordinates: Xyz::new(x, y, z),
        }
    }
    /// Computes the displacement from first to second; `Vector3::between(first,second)`.
    pub fn between(first: Point3, second: Point3) -> Self {
        Self {
            coordinates: Xyz::from(second) - Xyz::from(first),
        }
    }
    /// Borrows component algebra; `vector.xyz()`.
    pub const fn xyz(&self) -> &Xyz {
        &self.coordinates
    }
    /// Reads X; `vector.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `vector.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Reads Z; `vector.z()`.
    pub const fn z(&self) -> f64 {
        self.coordinates.z()
    }
    /// Reads a one-based coordinate, raising Standard_OutOfRange outside 1..=3;
    /// `vector.coord(2)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.coordinates.coord(index)
    }
    /// Replaces a one-based coordinate, keeping the vector on a bad index;
    /// `vector.set_coord(2,5.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        self.coordinates.set_coord(index, value)
    }
    /// Replaces all three components (gp_Vec::SetCoord(x,y,z));
    /// `vector.set_coordinates(1.0,2.0,3.0)`.
    pub const fn set_coordinates(&mut self, x: f64, y: f64, z: f64) {
        self.coordinates.set_coordinates(x, y, z);
    }
    /// Replaces X; `vector.set_x(1.0)`.
    pub const fn set_x(&mut self, x: f64) {
        self.coordinates.set_x(x);
    }
    /// Replaces Y; `vector.set_y(1.0)`.
    pub const fn set_y(&mut self, y: f64) {
        self.coordinates.set_y(y);
    }
    /// Replaces Z; `vector.set_z(1.0)`.
    pub const fn set_z(&mut self, z: f64) {
        self.coordinates.set_z(z);
    }
    /// Replaces the component algebra; `vector.set_xyz(Xyz::new(1.0,2.0,3.0))`.
    pub const fn set_xyz(&mut self, xyz: Xyz) {
        self.coordinates = xyz;
    }
}
impl From<Xyz> for Vector3 {
    fn from(coordinates: Xyz) -> Self {
        Self { coordinates }
    }
}
impl From<Direction3> for Vector3 {
    fn from(direction: Direction3) -> Self {
        Self::from(Xyz::from(direction))
    }
}

mod geometry;
mod linear_forms;
mod operations;
mod operators;
