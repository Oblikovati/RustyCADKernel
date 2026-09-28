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

use super::super::{Axis1, Axis2, Failure, Matrix3, Point3, Transform3, Vector3, Xyz};
impl Point3 {
    /// Assigns all coordinates; `point.set_coordinates(1.0,2.0,3.0)`.
    pub const fn set_coordinates(&mut self, x: f64, y: f64, z: f64) {
        self.coordinates = [x, y, z];
    }
    /// Replaces X; `point.set_x(1.0)`.
    pub const fn set_x(&mut self, x: f64) {
        self.coordinates[0] = x;
    }
    /// Replaces Y; `point.set_y(1.0)`.
    pub const fn set_y(&mut self, y: f64) {
        self.coordinates[1] = y;
    }
    /// Replaces Z; `point.set_z(1.0)`.
    pub const fn set_z(&mut self, z: f64) {
        self.coordinates[2] = z;
    }
    /// Returns coordinate algebra by value; `point.xyz()`.
    pub fn xyz(&self) -> Xyz {
        Xyz::from(*self)
    }
    /// Replaces coordinates from a triple; `point.set_xyz(xyz)`.
    pub fn set_xyz(&mut self, xyz: Xyz) {
        *self = Self::from(xyz);
    }
    /// Reads one coordinate by its one-based index; `point.coord(1)?`.
    pub fn coord(&self, index: i32) -> Result<f64, Failure> {
        self.xyz().coord(index)
    }
    /// Updates one coordinate, preserving the point on invalid index; `point.set_coord(1,2.0)?`.
    pub fn set_coord(&mut self, index: i32, value: f64) -> Result<(), Failure> {
        let mut xyz = self.xyz();
        xyz.set_coord(index, value)?;
        self.set_xyz(xyz);
        Ok(())
    }
    /// Computes Euclidean distance; `point.distance(&other)`.
    pub fn distance(&self, other: &Self) -> f64 {
        self.square_distance(other).sqrt()
    }
    /// Applies the native weighted-sum-then-divide sequence; `point.barycenter(1.0,other,1.0)`.
    pub fn barycenter(&mut self, alpha: f64, other: Self, beta: f64) {
        self.set_xyz((self.xyz() * alpha + other.xyz() * beta) / (alpha + beta));
    }
    /// Translates by a vector; `point.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::from(self.xyz() + *vector.xyz())
    }
    /// Translates in place; `point.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        *self = self.translated(vector);
    }
    /// Adds the destination then subtracts the origin, preserving native rounding; `point.translated_between(from,to)`.
    pub fn translated_between(self, from: Self, to: Self) -> Self {
        Self::from(self.xyz() + to.xyz() - from.xyz())
    }
    /// Translates between two origins in place; `point.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Self, to: Self) {
        *self = self.translated_between(from, to);
    }
    /// Scales about a center using the native linear form; `point.scaled(center,2.0)`.
    pub fn scaled(self, center: Self, factor: f64) -> Self {
        Self::from(self.xyz() * factor + center.xyz() * (1.0 - factor))
    }
    /// Scales this point in place; `point.scale(center,2.0)`.
    pub fn scale(&mut self, center: Self, factor: f64) {
        *self = self.scaled(center, factor);
    }
    /// Rotates through the native matrix and offset path; `point.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        Ok(Self::from(
            Transform3::rotation(axis, angle)?.apply_coordinates(self.xyz()),
        ))
    }
    /// Rotates this point in place; `point.rotate(axis,angle)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        *self = self.rotated(axis, angle)?;
        Ok(())
    }
    /// Mirrors through a point; `point.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Self) -> Self {
        Self::from(self.xyz().reversed() + center.xyz() * 2.0)
    }
    /// Mirrors this point through a center; `point.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Self) {
        *self = self.mirrored_point(center);
    }
    /// Mirrors through an oriented line; `point.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        self.mirrored_frame(axis.location(), Xyz::from(axis.direction()), false)
    }
    /// Mirrors this point through an oriented line; `point.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors through the frame's XY plane; `point.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        self.mirrored_frame(frame.location(), Xyz::from(frame.direction()), true)
    }
    /// Mirrors this point through a plane; `point.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        *self = self.mirrored_plane(frame);
    }
    /// Applies the native point-specific transform path; `point.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        transform.apply(self)
    }
    /// Transforms this point in place; `point.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        *self = self.transformed(transform);
    }
    fn mirrored_frame(self, origin: Self, normal: Xyz, plane: bool) -> Self {
        let mut matrix = Matrix3::default();
        matrix.set_dot(normal);
        matrix *= if plane { 2.0 } else { -2.0 };
        let offset = if plane { -1.0 } else { 1.0 };
        let diagonal = matrix.diagonal();
        matrix.set_diagonal(
            diagonal.x() + offset,
            diagonal.y() + offset,
            diagonal.z() + offset,
        );
        let translation = matrix.apply(origin.xyz()) + origin.xyz();
        if plane {
            Self::from(matrix.apply(self.xyz()) * -1.0 + translation)
        } else {
            matrix *= -1.0;
            Self::from(matrix.apply(self.xyz()) + translation)
        }
    }
}
