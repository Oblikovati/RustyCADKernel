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
// Rust translation of FoundationClasses/TKMath/gp/gp_Cylinder.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Cylinder: an infinite cylinder around the Z axis of its local coordinate system.
use super::placement::{
    axis3_surface_axis, axis3_surface_placement, column_product, scaled_length,
};
use super::{Axis3, Failure, Point3, Transform3, construction};
/// A located cylinder; `Cylinder3::new(frame,2.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Cylinder3 {
    position: Axis3,
    radius: f64,
}
impl Default for Cylinder3 {
    /// Native default: RealLast radius.
    fn default() -> Self {
        Self {
            position: Axis3::default(),
            radius: f64::MAX,
        }
    }
}
axis3_surface_placement!(Cylinder3);
axis3_surface_axis!(Cylinder3);
impl Cylinder3 {
    /// Rejects negative radii; `Cylinder3::new(frame,2.0)?`.
    pub fn new(position: Axis3, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Cylinder() - radius should be positive number, got {radius}"
            )));
        }
        Ok(Self { position, radius })
    }
    /// Rejects negative radii; `cylinder.set_radius(3.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Cylinder::SetRadius() - radius should be positive number, got {radius}"
            )));
        }
        self.radius = radius;
        Ok(())
    }
    /// Returns the radius; `cylinder.radius()`.
    pub const fn radius(&self) -> f64 {
        self.radius
    }
    /// Reverses the V parametrization by reversing the axis; `cylinder.v_reverse()`.
    pub fn v_reverse(&mut self) {
        self.position.z_reverse();
    }
    /// Returns `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` from the first two frame-change rows;
    /// `cylinder.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let rows = Transform3::frame_rows(self.position);
        let product = |i: usize, j: usize| column_product(&rows[..2], i, j);
        [
            product(0, 0),
            product(1, 1),
            product(2, 2),
            product(0, 1),
            product(0, 2),
            product(1, 2),
            product(0, 3),
            product(1, 3),
            product(2, 3),
            product(3, 3) - self.radius * self.radius,
        ]
    }
    /// Scales frame then radius; `cylinder.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `cylinder.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.position.scale(center, factor);
        self.radius = scaled_length(self.radius, factor);
    }
    /// Transforms frame then radius; `cylinder.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform3) -> Self {
        self.transform(transform);
        self
    }
    /// Transforms in place; `cylinder.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        self.position.transform(transform);
        self.radius = scaled_length(self.radius, transform.scale_factor());
    }
}
