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

//! gp_Ax2 right-handed frame with native automatic X-axis selection.
use super::direction::CheckedProducts;
use super::{Axis1, Axis3, Direction3, Failure, Point3, Transform3, Vector3};
/// A right-handed coordinate frame; `Axis2::new(origin,normal)?`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Axis2 {
    frame: Axis3,
}
impl Axis2 {
    /// Constructs a right-handed frame with a supplied X direction; `Axis2::with_x_direction(origin,normal,x)?`.
    pub fn with_x_direction(
        origin: Point3,
        normal: Direction3,
        x: Direction3,
    ) -> Result<Self, Failure> {
        Self::frame_using::<CheckedProducts>(origin, normal, x)
    }
    /// Copies the complete frame into gp_Ax3's representation; `frame.axis3()`.
    pub const fn axis3(&self) -> Axis3 {
        self.frame
    }

    /// Selects a perpendicular direction by OCCT's component ordering; `Axis2::new(origin,normal)?`.
    pub fn new(origin: Point3, normal: Direction3) -> Result<Self, Failure> {
        let [x, y, z] = *normal.coordinates();
        let [a, b, c] = perpendicular_candidate(x, y, z);
        Self::with_x_direction(origin, normal, Direction3::new(a, b, c)?)
    }

    /// Returns the frame origin; `frame.location()`.
    pub const fn location(&self) -> Point3 {
        self.frame.location()
    }
    /// Returns the plane normal; `frame.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.frame.direction()
    }
}
pub(super) fn perpendicular_candidate(x: f64, y: f64, z: f64) -> [f64; 3] {
    let [a, b, c] = [x.abs(), y.abs(), z.abs()];
    if b <= a && b <= c {
        return if a > c { [-z, 0.0, x] } else { [z, 0.0, -x] };
    }
    if a <= b && a <= c {
        return if b > c { [0.0, -z, y] } else { [0.0, z, -y] };
    }
    if a > b { [-y, x, 0.0] } else { [y, -x, 0.0] }
}

impl Axis2 {
    /// Returns the stored X direction; `frame.x_direction()`.
    pub const fn x_direction(&self) -> Direction3 {
        self.frame.x_direction()
    }
    /// Returns the stored Y direction; `frame.y_direction()`.
    pub const fn y_direction(&self) -> Direction3 {
        self.frame.y_direction()
    }
    /// Moves only the origin; `frame.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self {
            frame: self.frame.translated(vector),
        }
    }
    /// Reflects the origin and reverses X/Y while retaining the normal; `frame.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point3) -> Self {
        Self {
            frame: Axis3::from_stored(
                self.location().mirrored_point(center),
                self.direction(),
                self.x_direction().reversed(),
                self.y_direction().reversed(),
            ),
        }
    }
    /// Rotates X/Y and recomputes the normalized normal; `frame.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let origin = self.location().rotated(axis, angle)?;
        let x = self.x_direction().rotated(axis, angle)?;
        let y = self.y_direction().rotated(axis, angle)?;
        Ok(Self {
            frame: Axis3::from_stored(origin, x.crossed(y)?, x, y),
        })
    }
    /// Scales the origin and reverses X/Y for negative scale; `frame.scaled(center,-2.0)`.
    pub fn scaled(self, center: Point3, scale: f64) -> Self {
        let (mut x, mut y) = (self.x_direction(), self.y_direction());
        if scale < 0.0 {
            x.reverse();
            y.reverse();
        }
        Self {
            frame: Axis3::from_stored(
                self.location().scaled(center, scale),
                self.direction(),
                x,
                y,
            ),
        }
    }
    /// Transforms the origin and X/Y, then reconstructs the normal; `frame.transformed(transform)?`.
    pub fn transformed(self, transform: Transform3) -> Result<Self, Failure> {
        let origin = self.location().transformed(transform);
        let x = self.x_direction().transformed(transform);
        let y = self.y_direction().transformed(transform);
        Ok(Self {
            frame: Axis3::from_stored(origin, x.crossed(y)?, x, y),
        })
    }
}
mod orientation;
mod symmetry;
