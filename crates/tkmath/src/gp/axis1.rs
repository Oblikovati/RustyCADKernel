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

//! gp_Ax1 oriented line for rotations and axial symmetries.
use super::{Direction3, Point3};
/// An origin and oriented unit axis; `Axis1::new(Point3::default(),Direction3::Z)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Axis1 {
    location: Point3,
    direction: Direction3,
}
impl Default for Axis1 {
    fn default() -> Self {
        Self::new(Point3::default(), Direction3::Z)
    }
}
impl Axis1 {
    /// Retains the supplied origin and direction; `Axis1::new(origin,normal)`.
    pub const fn new(location: Point3, direction: Direction3) -> Self {
        Self {
            location,
            direction,
        }
    }
    /// Returns the origin; `axis.location()`.
    pub const fn location(&self) -> Point3 {
        self.location
    }
    /// Returns the unit direction; `axis.direction()`.
    pub const fn direction(&self) -> Direction3 {
        self.direction
    }
}
