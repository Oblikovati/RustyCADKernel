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

//! gp_Lin symmetries and motions, each applied to the located gp_Ax1.
use super::super::{Axis1, Axis2, Failure, Line3, Point3, Transform3, Vector3};
impl Line3 {
    /// Mirrors through a point; `line.mirrored_point(center)`.
    pub fn mirrored_point(self, center: Point3) -> Self {
        Self::from_axis(self.axis.mirrored_point(center))
    }
    /// Mirrors through a point in place; `line.mirror_point(center)`.
    pub fn mirror_point(&mut self, center: Point3) {
        self.axis.mirror_point(center);
    }
    /// Uses gp_Ax1's direction-overload mirror, not gp_Dir's axis-overload bug;
    /// `line.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        Self::from_axis(self.axis.mirrored_axis(axis))
    }
    /// Mirrors about an axis in place; `line.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        self.axis.mirror_axis(axis);
    }
    /// Mirrors through a frame's XY plane; `line.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        Self::from_axis(self.axis.mirrored_plane(frame))
    }
    /// Mirrors through a plane in place; `line.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        self.axis.mirror_plane(frame);
    }
    /// Rotates origin and direction by their native paths; `line.rotated(axis,angle)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        Ok(Self::from_axis(self.axis.rotated(axis, angle)?))
    }
    /// Rotates in place; `line.rotate(axis,angle)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        self.axis.rotate(axis, angle)
    }
    /// Scales the origin, reversing the direction for negative factors; `line.scaled(center,2.0)`.
    pub fn scaled(self, center: Point3, factor: f64) -> Self {
        Self::from_axis(self.axis.scaled(center, factor))
    }
    /// Scales in place; `line.scale(center,2.0)`.
    pub fn scale(&mut self, center: Point3, factor: f64) {
        self.axis.scale(center, factor);
    }
    /// Transforms the located axis; `line.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        Self::from_axis(self.axis.transformed(transform))
    }
    /// Transforms in place; `line.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        self.axis.transform(transform);
    }
    /// Translates the origin without changing orientation; `line.translated(vector)`.
    pub fn translated(self, vector: Vector3) -> Self {
        Self::from_axis(self.axis.translated(vector))
    }
    /// Translates in place; `line.translate(vector)`.
    pub fn translate(&mut self, vector: Vector3) {
        self.axis.translate(vector);
    }
    /// Native quirk: gp_Lin::Translated(P1,P2) adds the precomputed vector `P2 - P1`, while
    /// Translate(P1,P2) adds P2 then subtracts P1, so the two can differ in the last ulp;
    /// `line.translated_between(from,to)`.
    pub fn translated_between(self, from: Point3, to: Point3) -> Self {
        Self::from_axis(self.axis.translated(Vector3::between(from, to)))
    }
    /// Adds `to` then subtracts `from` (gp_Lin::Translate(P1,P2)); `line.translate_between(from,to)`.
    pub fn translate_between(&mut self, from: Point3, to: Point3) {
        self.axis.translate_between(from, to);
    }
}
