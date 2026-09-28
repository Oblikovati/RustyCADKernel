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
// Rust translation of FoundationClasses/TKMath/gp/gp_Ax2d.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Ax2d planar oriented axis.
use super::{Direction2, Point2, Transform2, Vector2};
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use std::io;
/// A planar oriented axis; `Axis2d::new(Point2::default(),Direction2::X)`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Axis2d {
    location: Point2,
    direction: Direction2,
}
impl Axis2d {
    /// Stores an origin and unit direction; `Axis2d::new(origin,direction)`.
    pub const fn new(location: Point2, direction: Direction2) -> Self {
        Self {
            location,
            direction,
        }
    }
    /// Returns the origin; `axis.location()`.
    pub const fn location(&self) -> Point2 {
        self.location
    }
    /// Returns the direction; `axis.direction()`.
    pub const fn direction(&self) -> Direction2 {
        self.direction
    }
    /// Replaces the origin; `axis.set_location(point)`.
    pub const fn set_location(&mut self, location: Point2) {
        self.location = location;
    }
    /// Replaces the direction; `axis.set_direction(direction)`.
    pub const fn set_direction(&mut self, direction: Direction2) {
        self.direction = direction;
    }
    /// Uses parallel (not equal) directions and both signed-cross distances; `axis.is_coaxial(other,1e-12,1e-7)`.
    pub fn is_coaxial(&self, other: Self, angular: f64, linear: f64) -> bool {
        let first = (self.location.xy() - other.location.xy())
            .crossed(other.direction.xy())
            .abs();
        let second = (other.location.xy() - self.location.xy())
            .crossed(self.direction.xy())
            .abs();
        self.direction.is_parallel(other.direction, angular) && first <= linear && second <= linear
    }
    /// Tests perpendicular directions; `axis.is_normal(other,1e-12)`.
    pub fn is_normal(&self, other: Self, angular: f64) -> bool {
        self.direction.is_normal(other.direction, angular)
    }
    /// Tests opposite directions; `axis.is_opposite(other,1e-12)`.
    pub fn is_opposite(&self, other: Self, angular: f64) -> bool {
        self.direction.is_opposite(other.direction, angular)
    }
    /// Tests parallel or opposite directions; `axis.is_parallel(other,1e-12)`.
    pub fn is_parallel(&self, other: Self, angular: f64) -> bool {
        self.direction.is_parallel(other.direction, angular)
    }
    /// Returns the signed angle between directions; `axis.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.direction.angle(other.direction)
    }
    /// Reverses the direction in place; `axis.reverse()`.
    pub fn reverse(&mut self) {
        self.direction.reverse();
    }
    /// Returns the reversed axis; `axis.reversed()`.
    pub fn reversed(self) -> Self {
        Self::new(self.location, self.direction.reversed())
    }
}
impl Axis2d {
    /// Mirrors the origin and reverses the direction; `axis.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point2) -> Self {
        Self::new(
            self.location.mirrored_point(center),
            self.direction.reversed(),
        )
    }
    /// Mirrors through a point in place; `axis.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point2) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors the origin through gp_Trsf2d and the direction by gp_Dir2d's direction overload;
    /// `axis.mirrored_axis(symmetry)`.
    pub fn mirrored_axis(self, symmetry: Self) -> Self {
        Self::new(
            self.location.mirrored_axis(symmetry),
            self.direction.mirrored_direction(symmetry.direction),
        )
    }
    /// Mirrors about an axis in place; `axis.mirror_axis(symmetry)`.
    pub fn mirror_axis(&mut self, symmetry: Self) {
        *self = self.mirrored_axis(symmetry);
    }
    /// Rotates the origin about a center and the direction by the angle; `axis.rotated(center,angle)`.
    pub fn rotated(self, center: Point2, angle: f64) -> Self {
        Self::new(
            self.location.rotated(center, angle),
            self.direction.rotated(angle),
        )
    }
    /// Rotates in place; `axis.rotate(center,angle)`.
    pub fn rotate(&mut self, center: Point2, angle: f64) {
        *self = self.rotated(center, angle);
    }
    /// Scales the origin and reverses the direction for negative factors; `axis.scaled(center,-2.0)`.
    pub fn scaled(self, center: Point2, factor: f64) -> Self {
        let mut direction = self.direction;
        if factor < 0.0 {
            direction.reverse();
        }
        Self::new(self.location.scaled(center, factor), direction)
    }
    /// Scales in place; `axis.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point2, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Transforms origin and direction independently; `axis.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        Self::new(
            self.location.transformed(transform),
            self.direction.transformed(transform),
        )
    }
    /// Transforms in place; `axis.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
    /// Translates the origin; `axis.translated(vector)`.
    pub fn translated(self, vector: Vector2) -> Self {
        Self::new(self.location.translated(vector), self.direction)
    }
    /// Translates in place; `axis.translate(vector)`.
    pub fn translate(&mut self, vector: Vector2) {
        *self = self.translated(vector);
    }
    /// Translates the origin with gp_Pnt2d's add-then-subtract order; `axis.translated_between(from,to)`.
    pub fn translated_between(self, from: Point2, to: Point2) -> Self {
        Self::new(self.location.translated_between(from, to), self.direction)
    }
    /// Translates between points in place; `axis.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point2, to: Point2) {
        *self = self.translated_between(from, to);
    }
    /// Writes the class name and nested `loc`/`vdir` fields; depth 0 omits the fields;
    /// `axis.dump_json(&mut stream,-1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "gp_Ax2d")?;
        if depth == 0 {
            return Ok(());
        }
        let mut location = DumpStream::default();
        self.location.dump_json(&mut location)?;
        dump::dump_key_to_class(output, "loc", location.text())?;
        let mut direction = DumpStream::default();
        self.direction.dump_json(&mut direction)?;
        dump::dump_key_to_class(output, "vdir", direction.text())
    }
}
