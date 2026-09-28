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
// Rust translation of FoundationClasses/TKMath/gp/gp_Hypr.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Hypr: the branch of a hyperbola on the positive X side of its right-handed frame.
use super::placement::{axis2_curve_placement, scaled_length};
use super::{
    Axis1, Axis2, Direction3, Failure, Point3, Transform3, Vector3, Xyz, construction, domain,
};
/// A located hyperbola branch; `Hyperbola3::new(frame,3.0,2.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Hyperbola3 {
    position: Axis2,
    major_radius: f64,
    minor_radius: f64,
}
impl Default for Hyperbola3 {
    /// Native default: RealLast major radius and RealFirst (-DBL_MAX) minor radius.
    fn default() -> Self {
        Self {
            position: Axis2::default(),
            major_radius: f64::MAX,
            minor_radius: f64::MIN,
        }
    }
}
axis2_curve_placement!(Hyperbola3);
impl Hyperbola3 {
    /// Requires non-negative radii (the minor radius may exceed the major one); `Hyperbola3::new(frame,3.0,2.0)?`.
    pub fn new(position: Axis2, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr() - invalid construction parameters: major {major_radius}, minor {minor_radius}; expected both >= 0"
            )));
        }
        Ok(Self {
            position,
            major_radius,
            minor_radius,
        })
    }
    /// Rebuilds the frame through gp_Ax2(P, N, X) rather than moving the origin; `hyperbola.set_location(point)?`.
    pub fn set_location(&mut self, location: Point3) -> Result<(), Failure> {
        let (normal, x) = (self.position.direction(), self.position.x_direction());
        self.position = Axis2::with_x_direction(location, normal, x)?;
        Ok(())
    }
    /// Requires a non-negative major radius; `hyperbola.set_major_radius(4.0)?`.
    pub fn set_major_radius(&mut self, major_radius: f64) -> Result<(), Failure> {
        if major_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr::SetMajorRadius() - major radius should be greater or equal zero, got {major_radius}"
            )));
        }
        self.major_radius = major_radius;
        Ok(())
    }
    /// Requires a non-negative minor radius; `hyperbola.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&mut self, minor_radius: f64) -> Result<(), Failure> {
        if minor_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr::SetMinorRadius() - minor radius should be greater or equal zero, got {minor_radius}"
            )));
        }
        self.minor_radius = minor_radius;
        Ok(())
    }
    /// Returns the major radius; `hyperbola.major_radius()`.
    pub const fn major_radius(&self) -> f64 {
        self.major_radius
    }
    /// Returns the minor radius; `hyperbola.minor_radius()`.
    pub const fn minor_radius(&self) -> f64 {
        self.minor_radius
    }
    /// Asymptote with slope `+minor/major`; `hyperbola.asymptote1()?`.
    pub fn asymptote1(&self) -> Result<Axis1, Failure> {
        self.asymptote(self.minor_radius / self.major_radius)
    }
    /// Asymptote with slope `-minor/major`; the native message names Asymptote1 for both;
    /// `hyperbola.asymptote2()?`.
    pub fn asymptote2(&self) -> Result<Axis1, Failure> {
        self.asymptote(-self.minor_radius / self.major_radius)
    }
    fn asymptote(&self, slope: f64) -> Result<Axis1, Failure> {
        if self.major_radius <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Hypr::Asymptote1() - major radius is zero: {}",
                self.major_radius
            )));
        }
        let rise = Vector3::from(self.position.y_direction()) * slope;
        let direction = Direction3::from_vector(Vector3::from(self.position.x_direction()) + rise)?;
        Ok(Axis1::new(self.location(), direction))
    }
    /// Branch on the positive Y side with swapped radii; `hyperbola.conjugate_branch1()?`.
    pub fn conjugate_branch1(&self) -> Result<Self, Failure> {
        self.branch(
            self.position.y_direction(),
            self.minor_radius,
            self.major_radius,
        )
    }
    /// Branch on the negative Y side with swapped radii; `hyperbola.conjugate_branch2()?`.
    pub fn conjugate_branch2(&self) -> Result<Self, Failure> {
        let y = self.position.y_direction().reversed();
        self.branch(y, self.minor_radius, self.major_radius)
    }
    /// Branch on the negative X side; `hyperbola.other_branch()?`.
    pub fn other_branch(&self) -> Result<Self, Failure> {
        let x = self.position.x_direction().reversed();
        self.branch(x, self.major_radius, self.minor_radius)
    }
    fn branch(&self, x: Direction3, major: f64, minor: f64) -> Result<Self, Failure> {
        let frame = Axis2::with_x_direction(self.location(), self.position.direction(), x)?;
        Self::new(frame, major, minor)
    }
    /// Directrix on the positive X side; `hyperbola.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis1, Failure> {
        self.directrix(1.0)
    }
    /// Directrix on the negative X side; `hyperbola.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis1, Failure> {
        self.directrix(-1.0)
    }
    fn directrix(&self, side: f64) -> Result<Axis1, Failure> {
        let eccentricity = self.eccentricity()?;
        let origin = Xyz::from(self.position.x_direction())
            * ((side * self.major_radius) / eccentricity)
            + self.location().xyz();
        Ok(Axis1::new(
            Point3::from(origin),
            self.position.y_direction(),
        ))
    }
    /// Computes `sqrt(a²+b²)/a`, rejecting a major radius at or below gp resolution; `hyperbola.eccentricity()?`.
    pub fn eccentricity(&self) -> Result<f64, Failure> {
        self.require_major("gp_Hypr::Eccentricity() - major radius is zero")?;
        Ok(self.focal_half() / self.major_radius)
    }
    fn require_major(&self, message: &str) -> Result<(), Failure> {
        if self.major_radius <= f64::MIN_POSITIVE {
            return Err(domain(&format!("{message}: {}", self.major_radius)));
        }
        Ok(())
    }
    fn focal_half(&self) -> f64 {
        (self.major_radius * self.major_radius + self.minor_radius * self.minor_radius).sqrt()
    }
    /// Distance between the foci; `hyperbola.focal()`.
    pub fn focal(&self) -> f64 {
        2.0 * self.focal_half()
    }
    /// Focus on the positive X side; `hyperbola.focus1()`.
    pub fn focus1(&self) -> Point3 {
        self.focus(self.focal_half())
    }
    /// Focus on the negative X side; `hyperbola.focus2()`.
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
    /// Semi-latus rectum `b²/a`; `hyperbola.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        self.require_major("gp_Hypr::Parameter() - major radius is zero")?;
        Ok((self.minor_radius * self.minor_radius) / self.major_radius)
    }
    /// Scales radii and frame; `hyperbola.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `hyperbola.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.scale(center, factor);
    }
    /// Transforms radii by the scale factor and the frame; `hyperbola.transformed(transform)?`.
    pub fn transformed(mut self, transform: Transform3) -> Result<Self, Failure> {
        self.transform(transform)?;
        Ok(self)
    }
    /// Transforms in place; `hyperbola.transform(transform)?`.
    pub fn transform(&mut self, transform: Transform3) -> Result<(), Failure> {
        let factor = transform.scale_factor();
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.transform(transform)
    }
}
