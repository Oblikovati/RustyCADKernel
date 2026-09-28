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
// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Trsf identity, translation, rotation, uniform scale, and composition forms.
use super::{Axis1, Axis3, Failure, Matrix3, Point3, Quaternion, Xyz};
/// Transformation form retained independently of numeric identity; `TransformForm::Rotation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformForm {
    /// Default identity transformation.
    #[default]
    Identity = 0,
    /// Translation, including a zero displacement.
    Translation = 2,
    /// Rotation, including zero-angle rotations.
    Rotation = 1,
    /// Point (central) symmetry, gp_PntMirror.
    PointMirror = 3,
    /// Plane (bilateral) symmetry, gp_Ax2Mirror.
    PlaneMirror = 5,
    /// Non-orthogonal general transformation, gp_Other (gp_GTrsf/gp_GTrsf2d only).
    Other = 8,
    /// Composition of different transformation forms.
    Compound = 7,
    /// Axis symmetry, gp_Ax1Mirror (the planar representation has scale -1).
    AxisMirror = 4,
    /// Uniform scaling, including unit and negative factors.
    Scale = 6,
}
/// gp transformation preserving native form-specific arithmetic; `Transform3::translation([1.0,0.0,0.0])`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3 {
    form: TransformForm,
    matrix: Matrix3,
    offset: Xyz,
    scale: f64,
}
impl Default for Transform3 {
    fn default() -> Self {
        let mut matrix = Matrix3::default();
        matrix.set_identity();
        Self {
            form: TransformForm::Identity,
            matrix,
            offset: Xyz::default(),
            scale: 1.0,
        }
    }
}
impl Transform3 {
    /// Builds a rotation from the quaternion matrix without translation; `Transform3::from_quaternion(quaternion)`.
    pub fn from_quaternion(rotation: Quaternion) -> Self {
        Self {
            form: TransformForm::Rotation,
            matrix: rotation.matrix(),
            ..Self::default()
        }
    }
    /// Extracts the native quaternion from the unscaled matrix; `transform.rotation_quaternion()`.
    pub fn rotation_quaternion(&self) -> Quaternion {
        Quaternion::from_matrix(self.matrix)
    }
    /// Maps one frame placement onto another using native multiplication order; `Transform3::displacement(from,to)`.
    pub fn displacement(from: Axis3, to: Axis3) -> Self {
        let mut matrix = Matrix3::from_columns(
            to.x_direction().into(),
            to.y_direction().into(),
            to.direction().into(),
        );
        let inverse = Matrix3::from_columns(
            from.x_direction().into(),
            from.y_direction().into(),
            from.direction().into(),
        )
        .transposed();
        let from_origin = inverse.apply(from.location().xyz()).reversed();
        let offset = to.location().xyz() + matrix.apply(from_origin);
        matrix *= inverse;
        Self {
            form: TransformForm::Compound,
            scale: 1.0,
            matrix,
            offset,
        }
    }

    /// Builds checked native uniform scaling; `Transform3::scaling(center,2.0)`.
    pub fn scaling(center: Point3, scale: f64) -> Result<Self, Failure> {
        if scale.abs() <= f64::MIN_POSITIVE {
            return Err(super::construction("gp_Trsf::SetScaleFactor"));
        }
        Ok(Self {
            form: TransformForm::Scale,
            scale,
            offset: center.xyz() * (1.0 - scale),
            ..Self::default()
        })
    }
    /// Returns the independent native scale factor; `transform.scale_factor()`.
    pub const fn scale_factor(&self) -> f64 {
        self.scale
    }

    /// Marks even a zero vector as a translation; `Transform3::translation([0.0;3])`.
    pub fn translation(vector: [f64; 3]) -> Self {
        Self {
            form: TransformForm::Translation,
            offset: Xyz::new(vector[0], vector[1], vector[2]),
            ..Self::default()
        }
    }
    /// Builds a rotation about a located axis using native offset order; `Transform3::rotation(axis,angle)?`.
    pub fn rotation(axis: Axis1, angle: f64) -> Result<Self, Failure> {
        let mut matrix = Matrix3::default();
        matrix.set_rotation(Xyz::from(axis.direction()), angle)?;
        let origin = Xyz::from(axis.location());
        let offset = matrix.apply(origin.reversed()) + origin;
        Ok(Self {
            form: TransformForm::Rotation,
            matrix,
            offset,
            scale: 1.0,
        })
    }
    /// Returns the stored OCCT form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Tests the stored form, not matrix equality; `transform.is_identity()`.
    pub fn is_identity(&self) -> bool {
        self.form == TransformForm::Identity
    }
    /// Returns the unscaled vectorial part; `transform.matrix()`.
    pub const fn matrix(&self) -> Matrix3 {
        self.matrix
    }
    /// Returns the translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xyz {
        self.offset
    }
    /// Applies the point-specific identity/translation fast paths; `transform.apply(Point3::default())`.
    pub fn apply(self, point: Point3) -> Point3 {
        let xyz = Xyz::from(point);
        match self.form {
            TransformForm::Identity => point,
            TransformForm::Translation => Point3::from(xyz + self.offset),
            TransformForm::Scale => Point3::from(xyz * self.scale + self.offset),
            _ => Point3::from(self.apply_coordinates(xyz)),
        }
    }
    /// Always executes matrix multiplication as gp_Trsf::Transforms does; `transform.apply_coordinates(xyz)`.
    pub fn apply_coordinates(self, coordinates: Xyz) -> Xyz {
        let mut result = self.matrix.apply(coordinates);
        if self.scale != 1.0 {
            result *= self.scale;
        }
        result + self.offset
    }
}
mod algebra;
mod frame_change;
