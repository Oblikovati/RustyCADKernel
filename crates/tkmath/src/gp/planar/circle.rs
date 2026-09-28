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
// Rust translation of FoundationClasses/TKMath/gp/gp_Circ2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Circ2d: a planar circle whose frame sense gives its orientation.
use super::super::placement::scaled_length;
use super::placement::{axis22d_curve_axes, axis22d_curve_placement};
use super::{Axis2d, Axis22d, Failure, Point2, Transform2, construction};
/// A planar circle; `Circle2::new(frame,2.0)?`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle2 {
    position: Axis22d,
    radius: f64,
}
impl Default for Circle2 {
    /// Native default: RealLast radius.
    fn default() -> Self {
        Self {
            position: Axis22d::default(),
            radius: f64::MAX,
        }
    }
}
axis22d_curve_placement!(Circle2);
axis22d_curve_axes!(Circle2);
impl Circle2 {
    /// Places the circle on a frame, rejecting negative radii; `Circle2::new(frame,2.0)?`.
    pub fn new(position: Axis22d, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Circ2d() - radius should be positive number, got {radius}"
            )));
        }
        Ok(Self { position, radius })
    }
    /// Uses the axis as X direction with the given sense; `Circle2::from_x_axis(axis,2.0,true)?`.
    pub fn from_x_axis(x_axis: Axis2d, radius: f64, sense: bool) -> Result<Self, Failure> {
        Self::new(Axis22d::from_axis(x_axis, sense)?, radius)
    }
    /// Rejects negative radii; `circle.set_radius(3.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(construction(&format!(
                "gp_Circ2d::SetRadius() - radius should be positive number, got {radius}"
            )));
        }
        self.radius = radius;
        Ok(())
    }
    /// Returns the radius; `circle.radius()`.
    pub const fn radius(&self) -> f64 {
        self.radius
    }
    /// Returns the frame (gp_Circ2d::Position, same as `axis`); `circle.position()`.
    pub const fn position(&self) -> Axis22d {
        self.position
    }
    /// Computes `PI * r * r`; `circle.area()`.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    /// Computes `2 * PI * r`; `circle.length()`.
    pub fn length(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    /// Returns `[A, B, C, D, E, F]` of `A·x² + B·y² + 2C·xy + 2D·x + 2E·y + F = 0`;
    /// `circle.coefficients()`.
    pub fn coefficients(&self) -> [f64; 6] {
        let (x, y) = (self.location().x(), self.location().y());
        [
            1.0,
            1.0,
            0.0,
            -x,
            -y,
            x * x + y * y - self.radius * self.radius,
        ]
    }
    /// Distance to the circumference; `circle.distance(point)`.
    pub fn distance(&self, point: Point2) -> f64 {
        let gap = self.radial_gap(point);
        if gap < 0.0 { -gap } else { gap }
    }
    /// Squared distance to the circumference; `circle.square_distance(point)`.
    pub fn square_distance(&self, point: Point2) -> f64 {
        let gap = self.radial_gap(point);
        gap * gap
    }
    fn radial_gap(&self, point: Point2) -> f64 {
        self.radius - (point.xy() - self.location().xy()).modulus()
    }
    /// Includes points within a distance tolerance; `circle.contains(point,1e-7)`.
    pub fn contains(&self, point: Point2, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Scales radius and frame; `circle.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point2, factor: f64) -> Self {
        self.scale(center, factor);
        self
    }
    /// Scales in place; `circle.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        self.radius = scaled_length(self.radius, factor);
        self.position.scale(center, factor);
    }
    /// Transforms radius by the scale factor and the frame; `circle.transformed(transform)`.
    pub fn transformed(mut self, transform: Transform2) -> Self {
        self.transform(transform);
        self
    }
    /// Transforms in place; `circle.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        self.radius = scaled_length(self.radius, transform.scale_factor());
        self.position.transform(transform);
    }
}
