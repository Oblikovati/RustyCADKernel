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
// Rust translation of FoundationClasses/TKMath/gp/gp_Elips.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Elips: an ellipse whose major axis is the X axis of its right-handed frame.
use super::placement::{axis2_curve_placement, scaled_length};
use super::{Axis1, Axis2, Failure, Point3, Transform3, Xyz, construction};
/// A located ellipse; `Ellipse3::new(frame,3.0,2.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Ellipse3 {
    position: Axis2,
    major_radius: f64,
    minor_radius: f64,
}
impl Default for Ellipse3 {
    /// Native default: RealLast major radius and RealSmall (DBL_MIN) minor radius.
    fn default() -> Self {
        Self {
            position: Axis2::default(),
            major_radius: f64::MAX,
            minor_radius: f64::MIN_POSITIVE,
        }
    }
}
axis2_curve_placement!(Ellipse3);
impl Ellipse3 {
    /// Requires `0 <= minor <= major`; `Ellipse3::new(frame,3.0,2.0)?`.
    pub fn new(position: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < minor_radius {
            return Err(construction(&format!(
                "gp_Elips() - invalid construction parameters: major {major_radius}, minor {minor_radius}; expected 0 <= minor <= major"
            )));
        }
        Ok(Self {
            position,
            major_radius,
            minor_radius,
        })
    }
    /// Moves the centre only; `ellipse.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        self.position.set_location(location);
    }
    /// Requires the new major radius to stay at least the minor one; `ellipse.set_major_radius(4.0)?`.
    pub fn set_major_radius(&mut self, major_radius: f64) -> Result<(), Failure> {
        if major_radius < self.minor_radius {
            return Err(construction(&format!(
                "gp_Elips::SetMajorRadius() - major radius should be greater or equal to minor radius: major {major_radius}, minor {}",
                self.minor_radius
            )));
        }
        self.major_radius = major_radius;
        Ok(())
    }
    /// Requires `0 <= minor <= major`; `ellipse.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&mut self, minor_radius: f64) -> Result<(), Failure> {
        if minor_radius < 0.0 || self.major_radius < minor_radius {
            return Err(construction(&format!(
                "gp_Elips::SetMinorRadius() - minor radius should be a positive number lesser or equal to major radius: minor {minor_radius}, major {}",
                self.major_radius
            )));
        }
        self.minor_radius = minor_radius;
        Ok(())
    }
    /// Returns the major radius; `ellipse.major_radius()`.
    pub const fn major_radius(&self) -> f64 {
        self.major_radius
    }
    /// Returns the minor radius; `ellipse.minor_radius()`.
    pub const fn minor_radius(&self) -> f64 {
        self.minor_radius
    }
    /// Computes `PI * major * minor`; `ellipse.area()`.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.major_radius * self.minor_radius
    }
    /// Returns zero for a zero major radius, otherwise `sqrt(a²-b²)/a`; `ellipse.eccentricity()`.
    pub fn eccentricity(&self) -> f64 {
        if self.major_radius == 0.0 {
            return 0.0;
        }
        self.focal_half() / self.major_radius
    }
    /// Distance between the foci; `ellipse.focal()`.
    pub fn focal(&self) -> f64 {
        2.0 * self.focal_half()
    }
    fn focal_half(&self) -> f64 {
        (self.major_radius * self.major_radius - self.minor_radius * self.minor_radius).sqrt()
    }
    /// Focus on the positive X side; `ellipse.focus1()`.
    pub fn focus1(&self) -> Point3 {
        self.focus(self.focal_half())
    }
    /// Focus on the negative X side; `ellipse.focus2()`.
    pub fn focus2(&self) -> Point3 {
        self.focus(-self.focal_half())
    }
    fn focus(&self, offset: f64) -> Point3 {
        let (origin, x) = (self.location(), self.position.x_direction());
        Point3::new(
            origin.x() + offset * x.x(),
            origin.y() + offset * x.y(),
            origin.z() + offset * x.z(),
        )
    }
    /// Semi-latus rectum `b²/a`, or zero for a zero major radius; `ellipse.parameter()`.
    pub fn parameter(&self) -> f64 {
        if self.major_radius == 0.0 {
            return 0.0;
        }
        (self.minor_radius * self.minor_radius) / self.major_radius
    }
    /// Directrix on the positive X side; a circle has none; `ellipse.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis1, Failure> {
        self.directrix(1.0, "gp_Elips::Directrix1() - zero eccentricity")
    }
    /// Directrix on the negative X side; `ellipse.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis1, Failure> {
        self.directrix(-1.0, "gp_Elips::Directrix2() - zero eccentricity")
    }
    fn directrix(&self, side: f64, message: &str) -> Result<Axis1, Failure> {
        let eccentricity = self.eccentricity();
        if eccentricity <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "{message}: eccentricity {eccentricity}"
            )));
        }
        // side * major reproduces the native `-majorRadius / anE` negation before the division.
        let origin = Xyz::from(self.position.x_direction())
            * ((side * self.major_radius) / eccentricity)
            + self.location().xyz();
        Ok(Axis1::new(
            Point3::from(origin),
            self.position.y_direction(),
        ))
    }
    /// Scales radii and frame; `ellipse.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `ellipse.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.scale(center, factor);
    }
    /// Transforms radii by the scale factor and the frame; `ellipse.transformed(transform)?`.
    pub fn transformed(mut self, transform: Transform3) -> Result<Self, Failure> {
        self.transform(transform)?;
        Ok(self)
    }
    /// Transforms in place; `ellipse.transform(transform)?`.
    pub fn transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        let factor = transform.scale_factor();
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.transform(transform)
    }
}
