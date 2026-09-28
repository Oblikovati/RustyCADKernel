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

//! gp_Vec2d mirror, rotation, scale and transformation members.
use super::super::{Axis2d, Point2, Transform2, TransformForm, Xy};
use super::Vector2;
impl Vector2 {
    /// Reflects about a unit direction with the native precomputed terms.
    fn reflected(self, a: f64, b: f64) -> Self {
        let cross_term = 2.0 * a * b;
        let xx_term = 2.0 * a * a - 1.0;
        let yy_term = 2.0 * b * b - 1.0;
        Self::new(
            xx_term * self.x() + cross_term * self.y(),
            cross_term * self.x() + yy_term * self.y(),
        )
    }
    /// Mirrors about an axis direction; `vector.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        self.reflected(axis.direction().x(), axis.direction().y())
    }
    /// Mirrors about an axis in place; `vector.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis2d) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors about another vector's direction; a vector at or below gp::Resolution leaves this
    /// one unchanged; `vector.mirrored_vector(other)`.
    pub fn mirrored_vector(self, other: Self) -> Self {
        let magnitude = other.magnitude();
        if magnitude <= f64::MIN_POSITIVE || magnitude.is_nan() {
            return self;
        }
        self.reflected(other.x() / magnitude, other.y() / magnitude)
    }
    /// Mirrors about another vector in place; `vector.mirror_vector(other)`.
    pub fn mirror_vector(&mut self, other: Self) {
        *self = self.mirrored_vector(other);
    }
    /// Multiplies by the rotation's vectorial part; `vector.rotated(0.5)`.
    pub fn rotated(self, angle: f64) -> Self {
        let rotation = Transform2::rotation(Point2::default(), angle);
        Self::from(self.xy().multiplied_matrix(rotation.vectorial_part()))
    }
    /// Rotates in place; `vector.rotate(0.5)`.
    pub fn rotate(&mut self, angle: f64) {
        *self = self.rotated(angle);
    }
    /// Multiplies by a scalar (gp_Vec2d::Scaled); `vector.scaled(2.0)`.
    pub fn scaled(self, factor: f64) -> Self {
        self * factor
    }
    /// Scales in place; `vector.scale(2.0)`.
    pub fn scale(&mut self, factor: f64) {
        *self = self.scaled(factor);
    }
    /// gp_Vec2d::Transform: translations do nothing, point mirrors negate, scales multiply,
    /// other forms apply the scaled vectorial part; `vector.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        let xy: Xy = match transform.form() {
            TransformForm::Identity | TransformForm::Translation => return self,
            TransformForm::PointMirror => self.xy().reversed(),
            TransformForm::Scale => self.xy() * transform.scale_factor(),
            _ => self.xy().multiplied_matrix(transform.vectorial_part()),
        };
        Self::from(xy)
    }
    /// Transforms in place; `vector.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
}
