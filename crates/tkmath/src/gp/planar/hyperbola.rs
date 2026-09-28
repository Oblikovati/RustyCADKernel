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
// Rust translation of FoundationClasses/TKMath/gp/gp_Hypr2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Hypr2d: the planar hyperbola branch on the positive X side of its frame.
use super::super::placement::scaled_length;
use super::placement::{axis22d_curve_axes, axis22d_curve_placement, central_conic_coefficients};
use super::{Axis2d, Axis22d, Direction2, Failure, Point2, Transform2, Vector2, construction};
use crate::gp::domain;
/// A planar hyperbola branch; `Hyperbola2::new(frame,3.0,2.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hyperbola2 {
    position: Axis22d,
    major_radius: f64,
    minor_radius: f64,
}
impl Default for Hyperbola2 {
    /// Native default: RealLast for both radii.
    fn default() -> Self {
        Self {
            position: Axis22d::default(),
            major_radius: f64::MAX,
            minor_radius: f64::MAX,
        }
    }
}
axis22d_curve_placement!(Hyperbola2);
axis22d_curve_axes!(Hyperbola2);
impl Hyperbola2 {
    /// Requires non-negative radii; `Hyperbola2::new(frame,3.0,2.0)?`.
    pub fn new(position: Axis22d, major_radius: f64, minor_radius: f64) -> Result<Self, Failure> {
        if minor_radius < 0.0 || major_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr2d() - invalid construction parameters: major {major_radius}, minor {minor_radius}; expected both >= 0"
            )));
        }
        Ok(Self {
            position,
            major_radius,
            minor_radius,
        })
    }
    /// Uses the axis as major axis with the given sense; `Hyperbola2::from_x_axis(axis,3.0,2.0,true)?`.
    pub fn from_x_axis(
        major_axis: Axis2d,
        major_radius: f64,
        minor_radius: f64,
        sense: bool,
    ) -> Result<Self, Failure> {
        Self::new(
            Axis22d::from_axis(major_axis, sense),
            major_radius,
            minor_radius,
        )
    }
    /// Requires a non-negative major radius; `hyperbola.set_major_radius(4.0)?`.
    pub fn set_major_radius(&mut self, major_radius: f64) -> Result<(), Failure> {
        if major_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr2d::SetMajorRadius() - major radius should be greater or equal zero, got {major_radius}"
            )));
        }
        self.major_radius = major_radius;
        Ok(())
    }
    /// Requires a non-negative minor radius; `hyperbola.set_minor_radius(1.0)?`.
    pub fn set_minor_radius(&mut self, minor_radius: f64) -> Result<(), Failure> {
        if minor_radius < 0.0 {
            return Err(construction(&format!(
                "gp_Hypr2d::SetMinorRadius() - minor radius should be greater or equal zero, got {minor_radius}"
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
    /// Native OCCT 3d097a bug kept: the direction normalizes `Y + Y·(b/a)`, i.e. the Y
    /// direction itself, instead of `X + Y·(b/a)`; `hyperbola.asymptote1()?`.
    pub fn asymptote1(&self) -> Result<Axis2d, Failure> {
        self.require_positive_major("gp_Hypr2d::Asymptote1() - major radius is zero")?;
        let y = self.position.y_direction().xy();
        let summed = y + y * (self.minor_radius / self.major_radius);
        let direction = Direction2::normalized_with(
            summed,
            "gp_Dir2d::SetCoord() - result vector has zero norm",
        )?;
        Ok(Axis2d::new(self.location(), direction))
    }
    /// Native OCCT 3d097a bug kept: the direction is `Y·(1 - b/a)` built in a gp_Vec2d, so equal
    /// radii raise gp_Dir2d's zero-norm error; `hyperbola.asymptote2()?`.
    pub fn asymptote2(&self) -> Result<Axis2d, Failure> {
        self.require_positive_major("gp_Hypr2d::Asymptote2() - major radius is zero")?;
        let y = self.position.y_direction().xy();
        let summed = y + y * (-self.minor_radius / self.major_radius);
        let direction = Direction2::from_vector(Vector2::from(summed))?;
        Ok(Axis2d::new(self.location(), direction))
    }
    fn require_positive_major(&self, message: &str) -> Result<(), Failure> {
        if self.major_radius <= f64::MIN_POSITIVE {
            return Err(construction(&format!("{message}: {}", self.major_radius)));
        }
        Ok(())
    }
    /// Returns `[A, B, C, D, E, F]` in the global system, all zero for degenerate radii;
    /// `hyperbola.coefficients()`.
    pub fn coefficients(&self) -> [f64; 6] {
        central_conic_coefficients(
            self.position.x_axis(),
            self.major_radius,
            self.minor_radius,
            -1.0,
        )
    }
    /// Branch along Y with swapped radii and the same sense; `hyperbola.conjugate_branch1()`.
    pub fn conjugate_branch1(&self) -> Self {
        self.branch(self.position.y_direction(), true)
    }
    /// Branch along -Y with swapped radii and the same sense; `hyperbola.conjugate_branch2()`.
    pub fn conjugate_branch2(&self) -> Self {
        self.branch(self.position.y_direction().reversed(), true)
    }
    /// Branch along -X with the same radii and sense; `hyperbola.other_branch()`.
    pub fn other_branch(&self) -> Self {
        self.branch(self.position.x_direction().reversed(), false)
    }
    // The native branches are noexcept: stored radii are never negative, so the checked
    // constructor they call cannot raise and is bypassed here.
    fn branch(&self, x: Direction2, swap_radii: bool) -> Self {
        let (major, minor) = if swap_radii {
            (self.minor_radius, self.major_radius)
        } else {
            (self.major_radius, self.minor_radius)
        };
        Self {
            position: Axis22d::new(self.location(), x, self.is_direct()),
            major_radius: major,
            minor_radius: minor,
        }
    }
    /// Directrix on the positive X side; `hyperbola.directrix1()?`.
    pub fn directrix1(&self) -> Result<Axis2d, Failure> {
        let eccentricity = self.eccentricity()?;
        let origin = self.position.x_direction().xy() * (self.major_radius / eccentricity)
            + self.location().xy();
        Ok(Axis2d::new(
            Point2::from(origin),
            self.position.y_direction(),
        ))
    }
    /// Native OCCT 3d097a asymmetry kept: offsets `focus1` by `parameter/e` along X instead of
    /// mirroring `directrix1`; `hyperbola.directrix2()?`.
    pub fn directrix2(&self) -> Result<Axis2d, Failure> {
        let eccentricity = self.eccentricity()?;
        let origin = self.position.x_direction().xy() * (self.parameter()? / eccentricity)
            + self.focus1().xy();
        Ok(Axis2d::new(
            Point2::from(origin),
            self.position.y_direction(),
        ))
    }
    /// Computes `sqrt(a²+b²)/a`, rejecting a major radius at or below gp resolution; `hyperbola.eccentricity()?`.
    pub fn eccentricity(&self) -> Result<f64, Failure> {
        self.require_major("gp_Hypr2d::Eccentricity() - major radius is zero")?;
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
    pub fn focus1(&self) -> Point2 {
        self.focus(self.focal_half())
    }
    /// Focus on the negative X side; `hyperbola.focus2()`.
    pub fn focus2(&self) -> Point2 {
        self.focus(-self.focal_half())
    }
    fn focus(&self, offset: f64) -> Point2 {
        let (origin, x) = (self.location(), self.position.x_direction());
        Point2::new(origin.x() + offset * x.x(), origin.y() + offset * x.y())
    }
    /// Semi-latus rectum `b²/a`; `hyperbola.parameter()?`.
    pub fn parameter(&self) -> Result<f64, Failure> {
        self.require_major("gp_Hypr2d::Parameter() - major radius is zero")?;
        Ok((self.minor_radius * self.minor_radius) / self.major_radius)
    }
    /// Scales radii and frame; `hyperbola.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point2, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `hyperbola.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.scale(center, factor);
    }
    /// Transforms radii by the scale factor and the frame; `hyperbola.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform2) -> Self {
        self.transform(transform);
        self
    }
    /// Transforms in place; `hyperbola.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        let factor = transform.scale_factor();
        self.major_radius = scaled_length(self.major_radius, factor);
        self.minor_radius = scaled_length(self.minor_radius, factor);
        self.position.transform(transform);
    }
}
