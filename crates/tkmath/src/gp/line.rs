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

//! gp_Lin metric and transformation operations over an oriented axis.
use super::{Axis1, Direction3, Failure, Point3, Transform3, Vector3, Xyz};
/// An unbounded oriented line, initially the Z axis; `Line3::new(origin,direction)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Line3 {
    axis: Axis1,
}
impl Line3 {
    /// Retains a point and unit direction; `Line3::new(origin,direction)`.
    pub const fn new(location: Point3, direction: Direction3) -> Self {
        Self {
            axis: Axis1::new(location, direction),
        }
    }
    /// Creates a line from an axis; `Line3::from_axis(axis)`.
    pub const fn from_axis(axis: Axis1) -> Self {
        Self { axis }
    }
    /// Returns the origin; `line.location()`.
    pub const fn location(&self) -> Point3 {
        self.axis.location()
    }
    /// Returns the oriented direction; `line.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.axis.direction()
    }
    /// Computes the norm of the displacement cross direction; `line.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.square_distance(point).sqrt()
    }
    /// Computes squared point distance; `line.square_distance(point)`.
    pub fn square_distance(&self, point: Point3) -> f64 {
        (point.xyz() - self.location().xyz())
            .crossed(Xyz::from(self.direction()))
            .square_modulus()
    }
    /// Computes line distance, using gp resolution for the parallel branch; `line.distance_line(other)?`.
    pub fn distance_line(&self, other: Self) -> Result<f64, Failure> {
        if self
            .direction()
            .is_parallel(other.direction(), f64::MIN_POSITIVE)
        {
            return Ok(other.distance(self.location()));
        }
        let normal = self.direction().crossed(other.direction())?;
        Ok(Vector3::between(self.location(), other.location())
            .dot(Vector3::from(normal))
            .abs())
    }
    /// Tests distance within tolerance; `line.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Returns the oriented-direction angle; `line.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.direction().angle(&other.direction())
    }
    /// Translates the origin without changing orientation; `line.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::new(self.location().translated(vector), self.direction())
    }
    /// Rotates origin and direction by their native paths; `line.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        Ok(Self::new(
            self.location().rotated(axis, angle)?,
            self.direction().rotated(axis, angle)?,
        ))
    }
    /// Uses gp_Ax1's direction-overload mirror, not gp_Dir's axis-overload bug; `line.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        Self::new(
            self.location().mirrored_axis(axis),
            self.direction().mirrored_direction(axis.direction()),
        )
    }
    /// Transforms the located axis; `line.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        Self::new(
            self.location().transformed(transform),
            self.direction().transformed(transform),
        )
    }
}
