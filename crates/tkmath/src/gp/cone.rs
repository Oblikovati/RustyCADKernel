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
// Rust translation of FoundationClasses/TKMath/gp/gp_Cone.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Cone: an infinite cone with a reference radius in its local XY plane.
use super::placement::{
    axis3_surface_axis, axis3_surface_placement, column_product, scaled_length,
};
use super::{Axis3, Failure, Point3, Transform3, Xyz, construction};
/// A located cone; `Cone3::new(frame,0.5,2.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Cone3 {
    position: Axis3,
    radius: f64,
    semi_angle: f64,
}
impl Default for Cone3 {
    /// Native default: RealLast reference radius and a PI/4 semi-angle.
    fn default() -> Self {
        Self {
            position: Axis3::default(),
            radius: f64::MAX,
            semi_angle: std::f64::consts::PI * 0.25,
        }
    }
}
axis3_surface_placement!(Cone3);
axis3_surface_axis!(Cone3);
impl Cone3 {
    /// Requires a non-negative radius and `resolution < |angle| < PI/2 - resolution`;
    /// `Cone3::new(frame,0.5,2.0)?`.
    pub fn new(position: Axis3, semi_angle: f64, radius: f64) -> Result<Self, Failure> {
        let absolute = if semi_angle < 0.0 {
            -semi_angle
        } else {
            semi_angle
        };
        if radius < 0.0 || !valid_semi_angle(absolute) {
            return Err(construction(&format!(
                "gp_Cone() - invalid construction parameters: semi-angle {semi_angle}, radius {radius}; expected radius >= 0 and 0 < |angle| < PI/2"
            )));
        }
        Ok(Self {
            position,
            radius,
            semi_angle,
        })
    }
    /// Rejects negative radii; `cone.set_radius(3.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Cone::SetRadius() - radius should be positive number, got {radius}"
            )));
        }
        self.radius = radius;
        Ok(())
    }
    /// Requires `resolution < |angle| < PI/2 - resolution`; `cone.set_semi_angle(0.3)?`.
    pub fn set_semi_angle(&mut self, semi_angle: f64) -> Result<(), Failure> {
        if !valid_semi_angle(semi_angle.abs()) {
            return Err(construction(&format!(
                "gp_Cone::SetSemiAngle() - invalid angle range: {semi_angle}; expected 0 < |angle| < PI/2"
            )));
        }
        self.semi_angle = semi_angle;
        Ok(())
    }
    /// Returns the reference radius; `cone.ref_radius()`.
    pub const fn ref_radius(&self) -> f64 {
        self.radius
    }
    /// Returns the signed semi-angle; `cone.semi_angle()`.
    pub const fn semi_angle(&self) -> f64 {
        self.semi_angle
    }
    /// Computes the apex on the axis at `-radius/tan(angle)`; `cone.apex()`.
    pub fn apex(&self) -> Point3 {
        let offset = Xyz::from(self.position.direction()) * (-self.radius / self.semi_angle.tan())
            + self.location().xyz();
        Point3::from(offset)
    }
    /// Reverses the axis and negates the semi-angle; `cone.v_reverse()`.
    pub fn v_reverse(&mut self) {
        self.position.z_reverse();
        self.semi_angle = -self.semi_angle;
    }
    /// Returns `[A1, A2, A3, B1, B2, B3, C1, C2, C3, D]` with the third frame row scaled by
    /// `tan(angle)`; `cone.coefficients()`.
    pub fn coefficients(&self) -> [f64; 10] {
        let mut rows = Transform3::frame_rows(self.position);
        let slope = self.semi_angle.tan();
        rows[2] = rows[2].map(|value| value * slope);
        let planar = |i: usize, j: usize| column_product(&rows[..2], i, j);
        let (axial, radius) = (rows[2], self.radius);
        let term = |i: usize, j: usize| planar(i, j) - axial[i] * axial[j];
        let offset = |i: usize| planar(i, 3) - axial[i] * (radius + axial[3]);
        [
            term(0, 0),
            term(1, 1),
            term(2, 2),
            term(0, 1),
            term(0, 2),
            term(1, 2),
            offset(0),
            offset(1),
            offset(2),
            planar(3, 3) - radius * radius - axial[3] * axial[3] - 2.0 * radius * axial[3],
        ]
    }
    /// Scales frame then radius; `cone.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `cone.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.position.scale(center, factor);
        self.radius = scaled_length(self.radius, factor);
    }
    /// Transforms frame then radius; `cone.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform3) -> Self {
        self.transform(transform);
        self
    }
    /// Transforms in place; `cone.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        self.position.transform(transform);
        self.radius = scaled_length(self.radius, transform.scale_factor());
    }
}
fn valid_semi_angle(absolute: f64) -> bool {
    !(absolute <= f64::MIN_POSITIVE || std::f64::consts::PI * 0.5 - absolute <= f64::MIN_POSITIVE)
}
