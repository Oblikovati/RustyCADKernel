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

use super::{Axis2d, Transform2, Vector2, Xy};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::io;
/// A planar point, initially the origin; `Point2::new(1.0,2.0)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point2 {
    coordinates: Xy,
}
impl Point2 {
    /// Stores unmodified coordinates; `Point2::new(1.0,2.0)`.
    pub const fn new(x: f64, y: f64) -> Self {
        Self {
            coordinates: Xy::new(x, y),
        }
    }
    /// Returns component algebra; `point.xy()`.
    pub const fn xy(&self) -> Xy {
        self.coordinates
    }
    /// Reads X; `point.x()`.
    pub const fn x(&self) -> f64 {
        self.coordinates.x()
    }
    /// Reads Y; `point.y()`.
    pub const fn y(&self) -> f64 {
        self.coordinates.y()
    }
    /// Replaces X; `point.set_x(1.0)`.
    pub fn set_x(&mut self, x: f64) {
        self.coordinates.coordinates[0] = x;
    }
    /// Replaces Y; `point.set_y(1.0)`.
    pub fn set_y(&mut self, y: f64) {
        self.coordinates.coordinates[1] = y;
    }
    /// Replaces both components; `point.set_coordinates(1.0,2.0)`.
    pub fn set_coordinates(&mut self, x: f64, y: f64) {
        self.coordinates = Xy::new(x, y);
    }
    /// Computes squared distance; `point.square_distance(other)`.
    pub fn square_distance(self, other: Self) -> f64 {
        (self.coordinates - other.coordinates).square_modulus()
    }
    /// Computes Euclidean distance; `point.distance(other)`.
    pub fn distance(self, other: Self) -> f64 {
        self.square_distance(other).sqrt()
    }
    /// Uses distance less than or equal to tolerance; `point.is_equal(other,1e-7)`.
    pub fn is_equal(self, other: Self, tolerance: f64) -> bool {
        self.distance(other) <= tolerance
    }
    /// Adds a vector; `point.translated(vector)`.
    pub fn translated(self, vector: Vector2) -> Self {
        Self::from(self.coordinates + vector.xy())
    }
    /// Scales about a center with native multiplication order; `point.scaled(center,2.0)`.
    pub fn scaled(self, center: Self, factor: f64) -> Self {
        Self::from(self.coordinates * factor + center.coordinates * (1.0 - factor))
    }
    /// Rotates about a center using gp_Trsf2d arithmetic; `point.rotated(center,angle)`.
    pub fn rotated(self, center: Self, angle: f64) -> Self {
        Self::from(Transform2::rotation(center, angle).apply_coordinates(self.coordinates))
    }
    /// Mirrors through a center; `point.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Self) -> Self {
        Self::from(self.coordinates.reversed() + center.coordinates * 2.0)
    }
    /// Mirrors through an oriented axis; `point.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        Self::from(Transform2::mirror_axis(axis).apply_coordinates(self.coordinates))
    }
    /// Applies native identity and translation special cases; `point.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        transform.apply(self)
    }
    /// Adds the destination then subtracts the origin, as gp_Pnt2d::Translate does; `point.translated_between(from,to)`.
    pub fn translated_between(self, from: Self, to: Self) -> Self {
        Self::from(self.coordinates + to.coordinates - from.coordinates)
    }
    /// Writes the native `gp_Pnt2d` vector fragment; `point.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "gp_Pnt2d", self.coordinates.coordinates())
    }
}
impl From<Xy> for Point2 {
    fn from(coordinates: Xy) -> Self {
        Self { coordinates }
    }
}
