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

use super::super::{Axis1, Axis2, Axis3, Direction3, Failure, Point3, Transform3, Vector3};
impl Axis3 {
    /// Uses native component ordering for the auxiliary X direction; `Axis3::from_normal(origin,normal)?`.
    pub fn from_normal(origin: Point3, normal: Direction3) -> Result<Self, Failure> {
        let [x, y, z] = *normal.coordinates();
        let [a, b, c] = super::super::axis2::perpendicular_candidate(x, y, z);
        let x_direction = Direction3::new(a, b, c)?;
        let y_direction = normal.crossed(x_direction)?;
        Ok(Self::from_stored(origin, normal, x_direction, y_direction))
    }
    /// Mirrors each stored direction without reorthogonalizing; `frame.mirrored_plane(plane)`.
    pub fn mirrored_plane(mut self, plane: Axis2) -> Self {
        self.y_direction.mirror_plane(plane);
        self.x_direction.mirror_plane(plane);
        self.location.mirror_plane(plane);
        self.direction.mirror_plane(plane);
        self
    }
    /// Rotates all four components in native order; `frame.rotated(axis,angle)?`.
    pub fn rotated(mut self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        self.location.rotate(axis, angle)?;
        self.direction.rotate(axis, angle)?;
        self.x_direction.rotate(axis, angle)?;
        self.y_direction.rotate(axis, angle)?;
        Ok(self)
    }
    /// Preserves gp_Ax1's negative-scale normal reversal despite gp_Ax3's comment; `frame.scaled(center,-1.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.location.scale(center, factor);
        if factor < 0.0 {
            self.direction.reverse();
            self.x_direction.reverse();
            self.y_direction.reverse();
        }
        self
    }
    /// Moves only the origin; `frame.translated(vector)`.
    pub fn translated(mut self, vector: Vector3) -> Self {
        self.location.translate(vector);
        self
    }
    /// Applies native point and direction transform paths independently; `frame.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform3) -> Self {
        self.location.transform(transform);
        self.direction.transform(transform);
        self.x_direction.transform(transform);
        self.y_direction.transform(transform);
        self
    }
}
