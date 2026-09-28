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
// Rust translation of FoundationClasses/TKMath/gp/gp_Torus.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Torus: a torus around the Z axis of its local coordinate system.
use super::placement::{
    absolute_factor, axis3_surface_axis, axis3_surface_placement, scaled_length,
};
use super::{Axis3, Failure, Point3, Transform3, construction};
use rustycad_tkernel::ncollection::Array1;
mod coefficients;
/// A located torus; `Torus3::new(frame,5.0,1.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Torus3 {
    position: Axis3,
    major_radius: f64,
    minor_radius: f64,
}
impl Default for Torus3 {
    /// Native default: RealLast major radius and RealSmall (DBL_MIN) minor radius.
    fn default() -> Self {
        Self {
            position: Axis3::default(),
            major_radius: f64::MAX,
            minor_radius: f64::MIN_POSITIVE,
        }
    }
}
axis3_surface_placement!(Torus3);
axis3_surface_axis!(Torus3);
impl Torus3 {
    /// Requires non-negative radii (a minor radius above the major one is accepted);
    /// `Torus3::new(frame,5.0,1.0)?`.
    pub fn new(position: Axis3, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Torus() - invalid construction parameters: major {major_radius}, minor {minor_radius}; expected both >= 0"
            )));
        }
        Ok(Self {
            position,
            major_radius,
            minor_radius,
        })
    }
    /// Requires `major - minor > resolution`; `torus.set_major_radius(6.0)?`.
    pub fn set_major_radius(&mut self, major_radius: f64) -> Result<(), Failure> {
        if major_radius - self.minor_radius <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Torus::SetMajorRadius() - invalid input parameters: major {major_radius}, minor {}; expected major > minor",
                self.minor_radius
            )));
        }
        self.major_radius = major_radius;
        Ok(())
    }
    /// Requires `0 <= minor` and `major - minor > resolution`; `torus.set_minor_radius(0.5)?`.
    pub fn set_minor_radius(&mut self, minor_radius: f64) -> Result<(), Failure> {
        if minor_radius < 0.0 || self.major_radius - minor_radius <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Torus::SetMinorRadius() - invalid input parameters: minor {minor_radius}, major {}; expected 0 <= minor < major",
                self.major_radius
            )));
        }
        self.minor_radius = minor_radius;
        Ok(())
    }
    /// Returns the major radius; `torus.major_radius()`.
    pub const fn major_radius(&self) -> f64 {
        self.major_radius
    }
    /// Returns the minor radius; `torus.minor_radius()`.
    pub const fn minor_radius(&self) -> f64 {
        self.minor_radius
    }
    /// Computes `4·PI·PI·minor·major`; `torus.area()`.
    pub fn area(&self) -> f64 {
        4.0 * std::f64::consts::PI * std::f64::consts::PI * self.minor_radius * self.major_radius
    }
    /// Computes `(PI·r·r)·(2·PI·R)`; `torus.volume()`.
    pub fn volume(&self) -> f64 {
        (std::f64::consts::PI * self.minor_radius * self.minor_radius)
            * (2.0 * std::f64::consts::PI * self.major_radius)
    }
    /// Reverses the V parametrization by reversing the axis; `torus.v_reverse()`.
    pub fn v_reverse(&mut self) {
        self.position.z_reverse();
    }
    /// Writes the 35 quartic coefficients from the array's lower bound; rejects arrays shorter
    /// than 35 with Standard_DimensionError; `torus.coefficients(&mut array)?`.
    pub fn coefficients(&self, output: &mut Array1<'_, f64>) -> Result<(), Failure> {
        if output.len() < 35 {
            return Err(Failure::with_type(
                "Standard_DimensionError",
                &format!(
                    "gp_Torus::theCoefficients(): Dimension mismatch: length {}, expected at least 35",
                    output.len()
                ),
                "",
            ));
        }
        let values = coefficients::quartic(
            Transform3::frame_rows(self.position),
            self.major_radius,
            self.minor_radius,
        );
        output.as_mut_slice()[..35].copy_from_slice(&values);
        Ok(())
    }
    /// Scales radii by multiply-then-negate, unlike the in-place `scale`; `torus.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.position.scale(center, factor);
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self
    }
    /// Multiplies radii by `|factor|`, so zero radii stay `+0.0`; `torus.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.position.scale(center, factor);
        let factor = absolute_factor(factor);
        self.major_radius *= factor;
        self.minor_radius *= factor;
    }
    /// Transforms radii by multiply-then-negate, unlike `transform`; `torus.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform3) -> Self {
        self.position.transform(transform);
        let factor = transform.scale_factor();
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self
    }
    /// Multiplies radii by `|scale|` (minor first, as native); `torus.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        self.position.transform(transform);
        let factor = absolute_factor(transform.scale_factor());
        self.minor_radius *= factor;
        self.major_radius *= factor;
    }
}
