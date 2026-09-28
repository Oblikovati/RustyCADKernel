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

use super::{Axis2d, Direction2, Failure, Point2, construction};
/// An unbounded planar line with native equation construction; `Line2::from_equation(1.0,-1.0,0.0)?`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Line2 {
    axis: Axis2d,
}
impl Line2 {
    /// Retains a located direction; `Line2::new(origin,direction)`.
    pub const fn new(origin: Point2, direction: Direction2) -> Self {
        Self {
            axis: Axis2d::new(origin, direction),
        }
    }

    /// Constructs the nearest-origin point of A*x+B*y+C=0 (OCC15489); `Line2::from_equation(a,b,c)?`.
    pub fn from_equation(a: f64, b: f64, c: f64) -> Result<Self, Failure> {
        let squared = a * a + b * b;
        if squared <= f64::MIN_POSITIVE {
            return Err(construction(" "));
        }
        let point = Point2::new(-a * c / squared, -b * c / squared);
        let direction = Direction2::new(-b, a)?;
        Ok(Self {
            axis: Axis2d::new(point, direction),
        })
    }
    /// Returns the equation constructor's nearest-origin point; `line.location()`.
    pub const fn location(&self) -> Point2 {
        self.axis.location()
    }
    /// Returns the located axis (gp_Lin2d::Position); `line.position()`.
    pub const fn position(&self) -> Axis2d {
        self.axis
    }
    /// Returns the oriented line direction; `line.direction()`.
    pub const fn direction(&self) -> Direction2 {
        self.axis.direction()
    }
}
