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

//! Local-frame algebra shared by the ElSLib evaluators.
use crate::gp::{Axis3, Point3, Vector3, Xyz};

/// The four gp_XYZ references every ElSLib evaluator reads from its gp_Ax3.
pub(super) struct Frame {
    pub(super) x: Xyz,
    pub(super) y: Xyz,
    pub(super) z: Xyz,
    pub(super) origin: Xyz,
}
impl Frame {
    pub(super) fn of(position: Axis3) -> Self {
        Self {
            x: Xyz::from(position.x_direction()),
            y: Xyz::from(position.y_direction()),
            z: Xyz::from(position.direction()),
            origin: position.location().xyz(),
        }
    }
    /// `a·X + b·Y`, per component `a*X.x + b*Y.x`.
    pub(super) fn pair(&self, a: f64, b: f64) -> Xyz {
        self.x * a + self.y * b
    }
    /// `a·X - b·Y`, per component `a*X.x - b*Y.x`.
    pub(super) fn difference(&self, a: f64, b: f64) -> Xyz {
        self.x * a - self.y * b
    }
    /// `a·X + b·Y + c·Z`, left to right.
    pub(super) fn triple(&self, a: f64, b: f64, c: f64) -> Xyz {
        self.x * a + self.y * b + self.z * c
    }
    /// `(-a)·X - b·Y + c·Z`, the native `-A3*X - A4*Y + R1*Z` pattern.
    pub(super) fn meridian(&self, a: f64, b: f64, c: f64) -> Xyz {
        self.x * -a - self.y * b + self.z * c
    }
    pub(super) fn point(&self, local: Xyz) -> Point3 {
        Point3::from(local + self.origin)
    }
}
pub(super) fn vector(xyz: Xyz) -> Vector3 {
    Vector3::from(xyz)
}
/// Torus evaluators flush terms within `10·(r+R)·ε` to zero before combining them.
pub(super) fn torus_tolerance(major: f64, minor: f64) -> f64 {
    10.0 * (minor + major) * f64::EPSILON
}
pub(super) fn flush(value: f64, tolerance: f64) -> f64 {
    if value.abs() <= tolerance { 0.0 } else { value }
}
