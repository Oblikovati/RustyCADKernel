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
// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Trsf: a 3D similarity kept as form, scale, unscaled matrix and translation.
use super::{Axis1, Axis2, Failure, Matrix3, Point3, Quaternion, Transform2, Vector3, Xyz};
mod compose;
mod forms;
mod frame_change;
mod json;
mod nlerp;
pub use nlerp::TransformNLerp;
mod power;
mod pre_compose;
mod setters;
/// gp_TrsfForm, retained independently of numeric identity; `TransformForm::Rotation`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum TransformForm {
    /// Default identity transformation.
    #[default]
    Identity = 0,
    /// Rotation, including zero-angle rotations.
    Rotation = 1,
    /// Translation, including a zero displacement.
    Translation = 2,
    /// Point (central) symmetry, gp_PntMirror.
    PointMirror = 3,
    /// Axis symmetry, gp_Ax1Mirror (the planar representation has scale -1).
    AxisMirror = 4,
    /// Plane (bilateral) symmetry, gp_Ax2Mirror.
    PlaneMirror = 5,
    /// Uniform scaling, including unit and negative factors.
    Scale = 6,
    /// Composition of different transformation forms.
    Compound = 7,
    /// Non-orthogonal general transformation, gp_Other (gp_GTrsf/gp_GTrsf2d only).
    Other = 8,
}
impl TransformForm {
    /// The form of a native integer, as `(gp_TrsfForm)(int)`; other values are undefined
    /// natively and give `None`; `TransformForm::from_native(3)`.
    pub const fn from_native(value: i32) -> Option<Self> {
        use TransformForm::*;
        Some(match value {
            0 => Identity,
            1 => Rotation,
            2 => Translation,
            3 => PointMirror,
            4 => AxisMirror,
            5 => PlaneMirror,
            6 => Scale,
            7 => Compound,
            8 => Other,
            _ => return None,
        })
    }
}
/// gp transformation preserving native form-specific arithmetic; `Transform3::translation([1.0,0.0,0.0])`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform3 {
    pub(super) form: TransformForm,
    pub(super) matrix: Matrix3,
    pub(super) offset: Xyz,
    pub(super) scale: f64,
}
impl Default for Transform3 {
    fn default() -> Self {
        Self {
            form: TransformForm::Identity,
            matrix: identity(),
            offset: Xyz::default(),
            scale: 1.0,
        }
    }
}
fn identity() -> Matrix3 {
    let mut matrix = Matrix3::default();
    matrix.set_identity();
    matrix
}
/// gp_Trsf(const gp_Trsf2d&): the planar matrix fills the upper-left block of a zero matrix
/// with a unit Z entry; an axis mirror becomes the half-turn about the axis (scale 1 and the
/// matrix negated, so the zero entries become -0).
impl From<Transform2> for Transform3 {
    fn from(planar: Transform2) -> Self {
        let m = *planar.matrix().coefficients();
        let offset = planar.translation_part();
        let mut matrix = Matrix3::new([
            [m[0][0], m[0][1], 0.0],
            [m[1][0], m[1][1], 0.0],
            [0.0, 0.0, 1.0],
        ]);
        let mut scale = planar.scale_factor();
        if planar.form() == TransformForm::AxisMirror {
            scale = 1.0;
            matrix *= -1.0;
        }
        Self {
            form: planar.form(),
            matrix,
            offset: Xyz::new(offset.x(), offset.y(), 0.0),
            scale,
        }
    }
}
impl Transform3 {
    /// Marks even a zero vector as a translation; `Transform3::translation([0.0;3])`.
    pub fn translation(vector: [f64; 3]) -> Self {
        Self::translation_vector(Vector3::new(vector[0], vector[1], vector[2]))
    }
    /// gp_Trsf::SetTranslation(gp_Vec); `Transform3::translation_vector(vector)`.
    pub fn translation_vector(vector: Vector3) -> Self {
        Self {
            form: TransformForm::Translation,
            offset: *vector.xyz(),
            ..Self::default()
        }
    }
    /// gp_Trsf::SetTranslation(P1,P2): the offset is `P2 - P1`; `Transform3::translation_between(a,b)`.
    pub fn translation_between(from: Point3, to: Point3) -> Self {
        Self::translation_vector(Vector3::from(to.xyz() - from.xyz()))
    }
    /// Point symmetry: identity matrix, scale -1, offset `2*center`; `Transform3::mirror_point(center)`.
    pub fn mirror_point(center: Point3) -> Self {
        Self {
            form: TransformForm::PointMirror,
            matrix: identity(),
            offset: center.xyz() * 2.0,
            scale: -1.0,
        }
    }
    /// Axis symmetry as the half-turn `2*D*D^T - I` with scale 1; the offset uses the matrix
    /// before its final negation; `Transform3::mirror_axis(axis)`.
    pub fn mirror_axis(axis: Axis1) -> Self {
        let origin = axis.location().xyz();
        let mut matrix = reflection(Xyz::from(axis.direction()), -2.0, 1.0);
        let offset = matrix.apply(origin) + origin;
        matrix *= -1.0;
        Self {
            form: TransformForm::AxisMirror,
            matrix,
            offset,
            scale: 1.0,
        }
    }
    /// Plane symmetry as `2*N*N^T - I` with scale -1; `Transform3::mirror_plane(frame)`.
    pub fn mirror_plane(frame: Axis2) -> Self {
        let origin = frame.location().xyz();
        let matrix = reflection(Xyz::from(frame.direction()), 2.0, -1.0);
        Self {
            form: TransformForm::PlaneMirror,
            matrix,
            offset: matrix.apply(origin) + origin,
            scale: -1.0,
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
    /// Builds a rotation from the quaternion matrix without translation, as
    /// `SetRotation(gp_Quaternion)`; `Transform3::from_quaternion(quaternion)`.
    pub fn from_quaternion(rotation: Quaternion) -> Self {
        Self {
            form: TransformForm::Rotation,
            matrix: rotation.matrix(),
            ..Self::default()
        }
    }
    /// Builds checked native uniform scaling; `Transform3::scaling(center,2.0)?`.
    pub fn scaling(center: Point3, scale: f64) -> Result<Self, Failure> {
        let mut transform = Self::default();
        transform.set_scale(center, scale)?;
        Ok(transform)
    }
    /// A compound transformation with the quaternion matrix and translation, as
    /// `SetTransformation(gp_Quaternion, gp_Vec)`; `Transform3::rotation_translation(q, v)`.
    pub fn rotation_translation(rotation: Quaternion, translation: Vector3) -> Self {
        Self {
            form: TransformForm::Compound,
            matrix: rotation.matrix(),
            offset: *translation.xyz(),
            scale: 1.0,
        }
    }
}
/// `SetDot(direction) * factor` with `shift` added to the diagonal.
fn reflection(direction: Xyz, factor: f64, shift: f64) -> Matrix3 {
    let mut matrix = Matrix3::default();
    matrix.set_dot(direction);
    matrix *= factor;
    let diagonal = matrix.diagonal();
    matrix.set_diagonal(
        diagonal.x() + shift,
        diagonal.y() + shift,
        diagonal.z() + shift,
    );
    matrix
}
impl Transform3 {
    /// Returns the stored OCCT form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Tests the stored form, not matrix equality; `transform.is_identity()`.
    pub fn is_identity(&self) -> bool {
        self.form == TransformForm::Identity
    }
    /// Tests the sign of the scale factor (unlike the planar determinant test);
    /// `transform.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.scale < 0.0
    }
    /// Returns the independent native scale factor; `transform.scale_factor()`.
    pub const fn scale_factor(&self) -> f64 {
        self.scale
    }
    /// Returns the unscaled vectorial part (gp_Trsf::HVectorialPart); `transform.matrix()`.
    pub const fn matrix(&self) -> Matrix3 {
        self.matrix
    }
    /// Returns the translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xyz {
        self.offset
    }
    /// Applies the point-specific shortcuts of gp_Pnt::Transform; `transform.apply(Point3::default())`.
    pub fn apply(self, point: Point3) -> Point3 {
        let xyz = Xyz::from(point);
        match self.form {
            TransformForm::Identity => point,
            TransformForm::Translation => Point3::from(xyz + self.offset),
            TransformForm::Scale => Point3::from(xyz * self.scale + self.offset),
            TransformForm::PointMirror => Point3::from(xyz.reversed() + self.offset),
            _ => Point3::from(self.apply_coordinates(xyz)),
        }
    }
    /// Always executes matrix multiplication as gp_Trsf::Transforms(gp_XYZ&) does;
    /// `transform.apply_coordinates(xyz)`.
    pub fn apply_coordinates(self, coordinates: Xyz) -> Xyz {
        let mut result = self.matrix.apply(coordinates);
        if self.scale != 1.0 {
            result *= self.scale;
        }
        result + self.offset
    }
    /// gp_Trsf::Transforms(double&,double&,double&); `transform.transforms_coordinates(1.0,2.0,3.0)`.
    pub fn transforms_coordinates(self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let result = self.apply_coordinates(Xyz::new(x, y, z));
        (result.x(), result.y(), result.z())
    }
}
