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

use super::{Axis1, Axis2, Failure, Point3, Transform3, Vector3, construction};
/// A located circle retaining frame orientation; `Circle3::new(frame,5.0)?`.
#[derive(Clone, Copy, Debug)]
pub struct Circle3 {
    position: Axis2,
    radius: f64,
}
impl Default for Circle3 {
    fn default() -> Self {
        Self {
            position: Axis2::default(),
            radius: f64::MAX,
        }
    }
}
impl Circle3 {
    /// Rejects negative radii while accepting zero; `Circle3::new(frame,radius)?`.
    pub fn new(position: Axis2, radius: f64) -> Result<Self, Failure> {
        if radius < 0.0 {
            return Err(construction("gp_Circ() - radius should be positive number"));
        }
        Ok(Self { position, radius })
    }
    /// Changes radius after the native check; `circle.set_radius(2.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        if radius < 0.0 {
            return Err(construction(
                "gp_Circ::SetRadius() - radius should be positive number",
            ));
        }
        self.radius = radius;
        Ok(())
    }
    /// Returns the stored radius; `circle.radius()`.
    pub const fn radius(&self) -> f64 {
        self.radius
    }
    /// Returns the plane frame; `circle.position()`.
    pub const fn position(&self) -> Axis2 {
        self.position
    }
    /// Returns the circle center; `circle.location()`.
    pub const fn location(&self) -> Point3 {
        self.position.location()
    }
    /// Uses native multiplication order; `circle.area()`.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    /// Computes native circumference; `circle.length()`.
    pub fn length(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
    /// Resolves the point into stored frame directions; `circle.square_distance(point)`.
    pub fn square_distance(&self, point: Point3) -> f64 {
        let displacement = Vector3::between(self.location(), point);
        let x = displacement.dot(Vector3::from(self.position.x_direction()));
        let y = displacement.dot(Vector3::from(self.position.y_direction()));
        let z = displacement.dot(Vector3::from(self.position.direction()));
        let radial = (x * x + y * y).sqrt() - self.radius;
        radial * radial + z * z
    }
    /// Computes minimum distance to the circumference; `circle.distance(point)`.
    pub fn distance(&self, point: Point3) -> f64 {
        self.square_distance(point).sqrt()
    }
    /// Includes points within the given circumference tolerance; `circle.contains(point,1e-7)`.
    pub fn contains(&self, point: Point3, tolerance: f64) -> bool {
        self.distance(point) <= tolerance
    }
    /// Moves the frame origin without changing radius; `circle.translated(vector)`.
    pub fn translated(mut self, vector: Vector3) -> Self {
        self.position = self.position.translated(vector);
        self
    }
    /// Rotates the complete right-handed frame; `circle.rotated(axis,angle)?`.
    pub fn rotated(mut self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        self.position = self.position.rotated(axis, angle)?;
        Ok(self)
    }
    /// Reflects through a point without reversing the plane normal; `circle.mirrored_point(center)`.
    pub fn mirrored_point(mut self, center: Point3) -> Self {
        self.position = self.position.mirrored_point(center);
        self
    }
    /// Scales radius and frame, retaining signed zero in radius; `circle.scaled(center,-2.0)`.
    pub fn scaled(mut self, center: Point3, factor: f64) -> Self {
        self.multiply_radius(factor);
        self.position = self.position.scaled(center, factor);
        self
    }
    /// Applies the transform's scale to radius and rebuilds frame orientation; `circle.transformed(transform)?`.
    pub fn transformed(mut self, transform: Transform3) -> Result<Self, Failure> {
        self.multiply_radius(transform.scale_factor());
        self.position = self.position.transformed(transform)?;
        Ok(self)
    }
    fn multiply_radius(&mut self, factor: f64) {
        self.radius *= factor;
        if self.radius < 0.0 {
            self.radius = -self.radius;
        }
    }
}
