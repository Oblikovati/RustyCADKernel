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

//! gp_Dir2d mirror, rotation and transformation members.
use super::super::{Axis2d, Point2, Transform2, TransformForm, Xy};
use super::Direction2;
impl Direction2 {
    /// Mirrors about an axis direction using the native component expressions;
    /// `direction.mirrored_axis(axis)`.
    pub fn mirrored_axis(self, axis: Axis2d) -> Self {
        self.mirrored_direction(axis.direction())
    }
    /// Mirrors about an axis in place; `direction.mirror_axis(axis)`.
    pub fn mirror_axis(&mut self, axis: Axis2d) {
        *self = self.mirrored_axis(axis);
    }
    /// Mirrors about another direction without renormalization; `direction.mirrored_direction(other)`.
    pub fn mirrored_direction(self, other: Self) -> Self {
        let (a, b, x, y) = (other.x(), other.y(), self.x(), self.y());
        let mixed = 2.0 * a * b;
        Self::from_stored(Xy::new(
            ((2.0 * a * a) - 1.0) * x + mixed * y,
            mixed * x + ((2.0 * b * b) - 1.0) * y,
        ))
    }
    /// Mirrors about another direction in place; `direction.mirror_direction(other)`.
    pub fn mirror_direction(&mut self, other: Self) {
        *self = self.mirrored_direction(other);
    }
    /// Multiplies by the unscaled rotation matrix without renormalizing; `direction.rotated(angle)`.
    pub fn rotated(self, angle: f64) -> Self {
        let rotation = Transform2::rotation(Point2::default(), angle);
        Self::from_stored(rotation.apply_unscaled(self.xy()))
    }
    /// Rotates in place; `direction.rotate(angle)`.
    pub fn rotate(&mut self, angle: f64) {
        *self = self.rotated(angle);
    }
    /// gp_Dir2d::Transform: point mirrors and scales only flip the sign; other forms apply the
    /// unscaled matrix, renormalize and flip for a negative scale; `direction.transformed(transform)`.
    pub fn transformed(self, transform: Transform2) -> Self {
        let negative = transform.scale_factor() < 0.0;
        let xy = match transform.form() {
            TransformForm::Identity | TransformForm::Translation => return self,
            TransformForm::PointMirror => self.xy().reversed(),
            TransformForm::Scale if negative => self.xy().reversed(),
            TransformForm::Scale => return self,
            _ => {
                let xy = transform.apply_unscaled(self.xy());
                let xy = xy / xy.modulus();
                if negative { xy.reversed() } else { xy }
            }
        };
        Self::from_stored(xy)
    }
    /// Transforms in place; `direction.transform(transform)`.
    pub fn transform(&mut self, transform: Transform2) {
        *self = self.transformed(transform);
    }
    /// Applies gp_Dir2d::SetCoord to a quarter turn of this direction; a stored direction is
    /// never zero, so the native zero-norm raise is unreachable; `direction.quarter_turn(true)`.
    pub(in crate::gp) fn quarter_turn(self, counter_clockwise: bool) -> Self {
        let xy = if counter_clockwise {
            Xy::new(-self.y(), self.x())
        } else {
            Xy::new(self.y(), -self.x())
        };
        let squared = xy.square_modulus();
        debug_assert!(squared.is_nan() || squared > 0.0, "zero Direction2 {xy:?}");
        if squared == 1.0 {
            return Self::from_stored(xy);
        }
        Self::from_stored(xy / squared.sqrt())
    }
}
