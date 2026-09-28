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

//! gp_Pln plane position and oriented equation at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::{Axis1, Axis2, Axis3, Direction3, Failure, Point3, Transform3, Vector3};
/// A plane retaining all three frame directions; `Plane3::new(origin,normal)?`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Plane3 {
    position: Axis3,
}
impl Plane3 {
    /// Copies the complete frame, including handedness; `Plane3::from_position(frame)`.
    pub const fn from_position(position: Axis3) -> Self {
        Self { position }
    }
    /// Selects the native auxiliary X direction; `Plane3::new(origin,normal)?`.
    pub fn new(origin: Point3, normal: Direction3) -> Result<Self, Failure> {
        Ok(Self::from_position(Axis2::new(origin, normal)?.axis3()))
    }
    /// Uses unnormalized equation coefficients for origin and frame selection; `Plane3::from_equation(0.0,0.0,1.0,-5.0)?`.
    pub fn from_equation(a: f64, b: f64, c: f64, d: f64) -> Result<Self, Failure> {
        let origin = equation_origin(a, b, c, d);
        let [x, y, z] = super::axis2::perpendicular_candidate(a, b, c);
        let position = Axis3::new(origin, Direction3::new(a, b, c)?, Direction3::new(x, y, z)?)?;
        Ok(Self::from_position(position))
    }
    /// Returns the stored coordinate frame; `plane.position()`.
    pub const fn position(&self) -> Axis3 {
        self.position
    }
    /// Returns the plane origin; `plane.location()`.
    pub const fn location(&self) -> Point3 {
        self.position.location()
    }
    /// Returns the origin and stored normal; `plane.axis()`.
    pub fn axis(&self) -> Axis1 {
        Axis1::new(self.location(), self.position.direction())
    }
    /// Signs the equation by frame handedness; `plane.coefficients()?`.
    pub fn coefficients(&self) -> Result<[f64; 4], Failure> {
        let mut normal = self.position.direction();
        if !self.position.direct()? {
            normal.reverse();
        }
        let [a, b, c] = *normal.coordinates();
        let origin = self.location();
        Ok([a, b, c, -(a * origin.x() + b * origin.y() + c * origin.z())])
    }
    /// Measures along the stored normal, independently of handedness; `plane.signed_distance(point)`.
    pub fn signed_distance(&self, point: Point3) -> f64 {
        let n = self.position.direction();
        let o = self.location();
        n.x() * (point.x() - o.x()) + n.y() * (point.y() - o.y()) + n.z() * (point.z() - o.z())
    }
    /// Returns absolute normal distance; `plane.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.signed_distance(point).abs()
    }
    /// Includes points within the supplied tolerance; `plane.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Mirrors the complete frame, preserving native handedness; `plane.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        Self::from_position(self.position.mirrored_plane(frame))
    }
    /// Rotates the frame around an oriented line; `plane.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        Ok(Self::from_position(self.position.rotated(axis, angle)?))
    }
    /// Scales the frame with native negative-scale direction reversal; `plane.scaled(center,2.0)`.
    pub fn scaled(self, center: Point3, factor: f64) -> Self {
        Self::from_position(self.position.scaled(center, factor))
    }
    /// Translates the stored origin; `plane.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::from_position(self.position.translated(vector))
    }
    /// Transforms the complete stored frame; `plane.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        Self::from_position(self.position.transformed(transform))
    }
}
fn equation_origin(a: f64, b: f64, c: f64, d: f64) -> Point3 {
    if a.abs() > b.abs() && a.abs() > c.abs() {
        return Point3::new(-d / a, 0.0, 0.0);
    }
    if b.abs() > c.abs() {
        return Point3::new(0.0, -d / b, 0.0);
    }
    Point3::new(0.0, 0.0, -d / c)
}
