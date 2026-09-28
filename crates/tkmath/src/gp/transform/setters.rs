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

//! gp_Trsf Set* members that replace the whole state; each delegates to the matching
//! constructor so the two spellings cannot drift apart.
use super::super::{
    Axis1, Axis2, Axis3, Failure, Point3, Quaternion, Transform3, TransformForm, Vector3,
    construction,
};
impl Transform3 {
    /// gp_Trsf::SetMirror(gp_Pnt); `transform.set_mirror_point(center)`.
    pub fn set_mirror_point(&mut self, center: Point3) {
        *self = Self::mirror_point(center);
    }
    /// gp_Trsf::SetMirror(gp_Ax1); `transform.set_mirror_axis(axis)`.
    pub fn set_mirror_axis(&mut self, axis: Axis1) {
        *self = Self::mirror_axis(axis);
    }
    /// gp_Trsf::SetMirror(gp_Ax2); `transform.set_mirror_plane(frame)`.
    pub fn set_mirror_plane(&mut self, frame: Axis2) {
        *self = Self::mirror_plane(frame);
    }
    /// gp_Trsf::SetRotation(gp_Ax1, double); `transform.set_rotation(axis, 0.5)?`.
    pub fn set_rotation(&mut self, axis: Axis1, angle: f64) -> Result<(), Failure> {
        *self = Self::rotation(axis, angle)?;
        Ok(())
    }
    /// gp_Trsf::SetRotation(gp_Quaternion); `transform.set_rotation_quaternion(q)`.
    pub fn set_rotation_quaternion(&mut self, rotation: Quaternion) {
        *self = Self::from_quaternion(rotation);
    }
    /// gp_Trsf::SetScale: the form, scale and offset are assigned before the zero-scale
    /// raise, as natively; the matrix becomes identity only on success;
    /// `transform.set_scale(center, 2.0)?`.
    pub fn set_scale(&mut self, center: Point3, scale: f64) -> Result<(), Failure> {
        self.form = TransformForm::Scale;
        self.scale = scale;
        self.offset = center.xyz();
        if scale.abs() <= f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_Trsf::SetScaleFactor - scale {scale} is not above gp::Resolution"
            )));
        }
        self.matrix.set_identity();
        self.offset *= 1.0 - scale;
        Ok(())
    }
    /// gp_Trsf::SetDisplacement; `transform.set_displacement(from, to)`.
    pub fn set_displacement(&mut self, from: Axis3, to: Axis3) {
        *self = Self::displacement(from, to);
    }
    /// gp_Trsf::SetTransformation(gp_Ax3, gp_Ax3); `transform.set_transformation_between(a, b)`.
    pub fn set_transformation_between(&mut self, from: Axis3, to: Axis3) {
        *self = Self::transformation_between(from, to);
    }
    /// gp_Trsf::SetTransformation(gp_Ax3); `transform.set_transformation_to(frame)`.
    pub fn set_transformation_to(&mut self, frame: Axis3) {
        *self = Self::transformation_to(frame);
    }
    /// gp_Trsf::SetTransformation(gp_Quaternion, gp_Vec); `transform.set_rotation_translation(q, v)`.
    pub fn set_rotation_translation(&mut self, rotation: Quaternion, translation: Vector3) {
        *self = Self::rotation_translation(rotation, translation);
    }
    /// gp_Trsf::SetTranslation(gp_Vec); `transform.set_translation(vector)`.
    pub fn set_translation(&mut self, vector: Vector3) {
        *self = Self::translation_vector(vector);
    }
    /// gp_Trsf::SetTranslation(gp_Pnt, gp_Pnt); `transform.set_translation_between(a, b)`.
    pub fn set_translation_between(&mut self, from: Point3, to: Point3) {
        *self = Self::translation_between(from, to);
    }
    /// Overwrites the form without touching the numbers, as `SetForm`; `transform.set_form(form)`.
    pub fn set_form(&mut self, form: TransformForm) {
        self.form = form;
    }
}
