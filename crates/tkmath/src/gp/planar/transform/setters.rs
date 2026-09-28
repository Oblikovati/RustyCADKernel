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

//! gp_Trsf2d Set* members that replace the whole state; each delegates to the matching
//! constructor so the setter and constructor spellings cannot drift apart.
use super::super::{Axis2d, Point2, Vector2};
use super::Transform2;
impl Transform2 {
    /// gp_Trsf2d::SetMirror(gp_Pnt2d); `transform.set_mirror_point(center)`.
    pub fn set_mirror_point(&mut self, center: Point2) {
        *self = Self::mirror_point(center);
    }
    /// gp_Trsf2d::SetMirror(gp_Ax2d); `transform.set_mirror_axis(axis)`.
    pub fn set_mirror_axis(&mut self, axis: Axis2d) {
        *self = Self::mirror_axis(axis);
    }
    /// gp_Trsf2d::SetRotation; `transform.set_rotation(center,0.5)`.
    pub fn set_rotation(&mut self, center: Point2, angle: f64) {
        *self = Self::rotation(center, angle);
    }
    /// gp_Trsf2d::SetScale; `transform.set_scale(center,2.0)`.
    pub fn set_scale(&mut self, center: Point2, scale: f64) {
        *self = Self::scaling(center, scale);
    }
    /// gp_Trsf2d::SetTransformation(gp_Ax2d,gp_Ax2d); `transform.set_transformation_between(a,b)`.
    pub fn set_transformation_between(&mut self, from: Axis2d, to: Axis2d) {
        *self = Self::transformation_between(from, to);
    }
    /// gp_Trsf2d::SetTransformation(gp_Ax2d); `transform.set_transformation_to(axis)`.
    pub fn set_transformation_to(&mut self, axis: Axis2d) {
        *self = Self::transformation_to(axis);
    }
    /// gp_Trsf2d::SetTranslation(gp_Vec2d); `transform.set_translation(vector)`.
    pub fn set_translation(&mut self, vector: Vector2) {
        *self = Self::translation_vector(vector);
    }
    /// gp_Trsf2d::SetTranslation(gp_Pnt2d,gp_Pnt2d); `transform.set_translation_between(a,b)`.
    pub fn set_translation_between(&mut self, from: Point2, to: Point2) {
        *self = Self::translation_between(from, to);
    }
}
