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
// Rust translation of FoundationClasses/TKMath/gp/gp_Lin2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Lin2d: an unbounded planar line carried by an axis.
use super::{Axis2d, Direction2, Failure, Point2, Xy, construction};
mod geometry;
/// An unbounded planar line with native equation construction; `Line2::from_equation(1.0,-1.0,0.0)?`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Line2 {
    axis: Axis2d,
}
impl From<Axis2d> for Line2 {
    fn from(axis: Axis2d) -> Self {
        Self { axis }
    }
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
            return Err(construction(&format!(
                "gp_Lin2d(A, B, C) - null normal ({a}, {b})"
            )));
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
    /// Replaces the direction; `line.set_direction(direction)`.
    pub const fn set_direction(&mut self, direction: Direction2) {
        self.axis.set_direction(direction);
    }
    /// Replaces the origin; `line.set_location(point)`.
    pub const fn set_location(&mut self, location: Point2) {
        self.axis.set_location(location);
    }
    /// Replaces the axis; `line.set_position(axis)`.
    pub const fn set_position(&mut self, axis: Axis2d) {
        self.axis = axis;
    }
    /// Reverses the direction in place; `line.reverse()`.
    pub fn reverse(&mut self) {
        self.axis.reverse();
    }
    /// Returns the reversed line; `line.reversed()`.
    pub fn reversed(self) -> Self {
        Self::from(self.axis.reversed())
    }
    /// Returns `(A, B, C)` with `A = Dy`, `B = -Dx`, `C = -(A*x0 + B*y0)`; `line.coefficients()`.
    pub fn coefficients(&self) -> (f64, f64, f64) {
        let a = self.direction().y();
        let b = -self.direction().x();
        (a, b, -(a * self.location().x() + b * self.location().y()))
    }
    /// Returns the signed angle between directions; `line.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.direction().angle(other.direction())
    }
    /// Tests the point distance against a tolerance; `line.contains(point,1e-7)`.
    pub fn contains(&self, point: Point2, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Computes `|(P - O) ^ D|`; `line.distance(point)`.
    pub fn distance(&self, point: Point2) -> f64 {
        self.signed_offset(point).abs()
    }
    /// Computes the squared point distance; `line.square_distance(point)`.
    pub fn square_distance(&self, point: Point2) -> f64 {
        let offset = self.signed_offset(point);
        offset * offset
    }
    fn signed_offset(&self, point: Point2) -> f64 {
        (point.xy() - self.location().xy()).crossed(self.direction().xy())
    }
    /// Returns the other line's distance to this origin when the axes are parallel within
    /// gp::Resolution, otherwise 0; `line.distance_line(other)`.
    pub fn distance_line(&self, other: Self) -> f64 {
        if !self.axis.is_parallel(other.axis, f64::MIN_POSITIVE) {
            return 0.0;
        }
        other.distance(self.location())
    }
    /// Squared form of `distance_line`; `line.square_distance_line(other)`.
    pub fn square_distance_line(&self, other: Self) -> f64 {
        if !self.axis.is_parallel(other.axis, f64::MIN_POSITIVE) {
            return 0.0;
        }
        other.square_distance(self.location())
    }
    /// Returns the line through a point along the quarter-turned direction, renormalized by
    /// gp_Dir2d(-Dy, Dx). A direction built from overflowing components is stored as zero; the
    /// native `noexcept` member then calls std::terminate, while Rust returns the
    /// ConstructionError; `line.normal(point)?`.
    pub fn normal(&self, point: Point2) -> Result<Self, Failure> {
        let direction = self.direction();
        let normal = Direction2::normalized_with(
            Xy::new(-direction.y(), direction.x()),
            "gp_Lin2d::Normal() - gp_Dir2d() - input vector has zero norm",
        )?;
        Ok(Self::new(point, normal))
    }
}
