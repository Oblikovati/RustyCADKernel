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

// Rust translation of FoundationClasses/TKMath/gp/gp_Vec.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Vec symmetries, rotation, scaling, transformation and JSON dump.
use super::super::{Axis1, Axis2, Failure, Transform3, TransformForm, Vector3, Xyz, reflect};
use rustycad_tkernel::standard::dump::{self, DumpOutput};
use std::{cmp::Ordering, io};
impl Vector3 {
    /// Mirrors about another vector's line; a vector whose magnitude is not above
    /// gp::Resolution (including NaN) leaves this one unchanged; `vector.mirrored_vector(other)`.
    pub fn mirrored_vector(self, other: Self) -> Self {
        let magnitude = other.magnitude();
        if magnitude.partial_cmp(&f64::MIN_POSITIVE) != Some(Ordering::Greater) {
            return self;
        }
        let unit = other
            .coordinates
            .coordinates()
            .map(|value| value / magnitude);
        self.reflected(unit)
    }
    /// Mirrors about another vector in place; `vector.mirror_vector(other)`.
    pub fn mirror_vector(&mut self, other: Self) {
        *self = self.mirrored_vector(other);
    }
    /// Mirrors about an axis direction (all three components, unlike gp_Dir's axis mirror);
    /// `vector.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis1) -> Self {
        self.reflected(*axis.direction().coordinates())
    }
    /// Mirrors about an axis in place; `vector.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis1) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors through the frame's XY plane: a vector parallel to the normal is reversed,
    /// otherwise it is reflected about `N ^ (N ^ V)`; `vector.mirrored_plane(frame)`.
    pub fn mirrored_plane(self, frame: Axis2) -> Self {
        let normal = Xyz::from(frame.direction());
        let mirror = normal.crossed(self.coordinates);
        if mirror.modulus() <= f64::MIN_POSITIVE {
            return self.reversed();
        }
        self.mirrored_vector(Self::from(normal.crossed(mirror)))
    }
    /// Mirrors through a frame's plane in place; `vector.mirror_plane(frame)`.
    pub fn mirror_plane(&mut self, frame: Axis2) {
        *self = self.mirrored_plane(frame);
    }
    fn reflected(self, unit: [f64; 3]) -> Self {
        let [x, y, z] = reflect(unit, *self.coordinates.coordinates());
        Self::new(x, y, z)
    }
    /// Multiplies by the rotation's vectorial part; `vector.rotated(axis,0.5)?`.
    pub fn rotated(self, axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let rotation = Transform3::rotation(axis, angle)?;
        Ok(Self::from(self.coordinates * rotation.vectorial_part()))
    }
    /// Rotates in place; `vector.rotate(axis,0.5)?`.
    pub fn rotate(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        *self = self.rotated(axis, angle)?;
        Ok(())
    }
    /// Multiplies by a scalar (gp_Vec::Scaled); `vector.scaled(2.0)`.
    pub fn scaled(self, factor: f64) -> Self {
        self * factor
    }
    /// Scales in place; `vector.scale(2.0)`.
    pub fn scale(&mut self, factor: f64) {
        *self = self.scaled(factor);
    }
    /// gp_Vec::Transform: translations do nothing, point mirrors negate, scales multiply,
    /// other forms apply the scaled vectorial part; `vector.transformed(transform)`.
    pub fn transformed(self, transform: Transform3) -> Self {
        match transform.form() {
            TransformForm::Identity | TransformForm::Translation => self,
            TransformForm::PointMirror => self.reversed(),
            TransformForm::Scale => self * transform.scale_factor(),
            _ => Self::from(self.coordinates * transform.vectorial_part()),
        }
    }
    /// Transforms in place; `vector.transform(transform)`.
    pub fn transform(&mut self, transform: Transform3) {
        *self = self.transformed(transform);
    }
    /// Writes the native `gp_Vec` JSON fragment; `vector.dump_json(&mut stream)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput) -> io::Result<()> {
        dump::dump_vector(output, "gp_Vec", self.coordinates.coordinates())
    }
}
