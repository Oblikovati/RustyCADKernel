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
// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_Trsf2d: a planar similarity kept as form, scale, unscaled matrix and translation.
use super::{Axis2d, Failure, Matrix2, Point2, TransformForm, Vector2, Xy};
use crate::gp::Transform3;
mod compose;
mod forms;
mod power;
mod pre_compose;
mod setters;
/// Planar similarity with gp_Trsf2d's form-specific arithmetic; `Transform2::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2 {
    pub(super) form: TransformForm,
    pub(super) matrix: Matrix2,
    pub(super) offset: Xy,
    pub(super) scale: f64,
}
impl Default for Transform2 {
    fn default() -> Self {
        Self {
            form: TransformForm::Identity,
            matrix: Matrix2::identity(),
            offset: Xy::default(),
            scale: 1.0,
        }
    }
}
/// gp_Trsf2d(const gp_Trsf&) keeps the 3D form, scale, the XY translation and the upper-left
/// 2x2 of the unscaled matrix, even when the form (such as a plane mirror) has no planar meaning.
impl From<Transform3> for Transform2 {
    fn from(transform: Transform3) -> Self {
        let m = *transform.matrix().coefficients();
        let offset = transform.translation_part();
        Self {
            form: transform.form(),
            matrix: Matrix2::new([[m[0][0], m[0][1]], [m[1][0], m[1][1]]]),
            offset: Xy::new(offset.x(), offset.y()),
            scale: transform.scale_factor(),
        }
    }
}
impl Transform2 {
    /// Constructs even a zero displacement with Translation form; `Transform2::translation([1.0,2.0])`.
    pub fn translation(vector: [f64; 2]) -> Self {
        Self::translation_vector(Vector2::new(vector[0], vector[1]))
    }
    /// gp_Trsf2d::SetTranslation(gp_Vec2d); `Transform2::translation_vector(vector)`.
    pub fn translation_vector(vector: Vector2) -> Self {
        Self {
            form: TransformForm::Translation,
            offset: vector.xy(),
            ..Self::default()
        }
    }
    /// gp_Trsf2d::SetTranslation(P1,P2): the offset is `P2 - P1`;
    /// `Transform2::translation_between(from,to)`.
    pub fn translation_between(from: Point2, to: Point2) -> Self {
        Self::translation_vector(Vector2::from(to.xy() - from.xy()))
    }
    /// Uses the native rotation matrix and negated-origin multiplication; `Transform2::rotation(center,angle)`.
    pub fn rotation(center: Point2, angle: f64) -> Self {
        let mut matrix = Matrix2::default();
        matrix.set_rotation(angle);
        let offset = center.xy().reversed().multiplied_matrix(matrix) + center.xy();
        Self {
            form: TransformForm::Rotation,
            matrix,
            offset,
            scale: 1.0,
        }
    }
    /// Point symmetry: identity matrix, scale -1, offset `2*center`; `Transform2::mirror_point(center)`.
    pub fn mirror_point(center: Point2) -> Self {
        Self {
            form: TransformForm::PointMirror,
            matrix: Matrix2::identity(),
            offset: center.xy() * 2.0,
            scale: -1.0,
        }
    }
    /// Uniform scale about a center; no zero check, as in the inline header;
    /// `Transform2::scaling(center,2.0)`.
    pub fn scaling(center: Point2, scale: f64) -> Self {
        Self {
            form: TransformForm::Scale,
            matrix: Matrix2::identity(),
            offset: center.xy() * (1.0 - scale),
            scale,
        }
    }
    /// Constructs the native matrix/negative-scale mirror representation; `Transform2::mirror_axis(axis)`.
    pub fn mirror_axis(axis: Axis2d) -> Self {
        let (vx, vy, x, y) = (
            axis.direction().x(),
            axis.direction().y(),
            axis.location().x(),
            axis.location().y(),
        );
        let matrix = Matrix2::new([
            [1.0 - 2.0 * vx * vx, -2.0 * vx * vy],
            [-2.0 * vx * vy, 1.0 - 2.0 * vy * vy],
        ]);
        let offset = Xy::new(
            -2.0 * ((vx * vx - 1.0) * x + (vx * vy * y)),
            -2.0 * ((vx * vy * x) + (vy * vy - 1.0) * y),
        );
        Self {
            form: TransformForm::AxisMirror,
            matrix,
            offset,
            scale: -1.0,
        }
    }
    /// Maps global coordinates into the axis system (gp_Trsf2d::SetTransformation(gp_Ax2d));
    /// `Transform2::transformation_to(axis)`.
    pub fn transformation_to(axis: Axis2d) -> Self {
        let matrix = axis_rows(axis);
        let offset = axis.location().xy().multiplied_matrix(matrix).reversed();
        Self {
            form: TransformForm::Compound,
            matrix,
            offset,
            scale: 1.0,
        }
    }
    /// Maps coordinates in `from` to coordinates in `to` (gp_Trsf2d::SetTransformation(A1,A2));
    /// `Transform2::transformation_between(from,to)`.
    pub fn transformation_between(from: Axis2d, to: Axis2d) -> Self {
        let mut transform = Self::transformation_to(to);
        let first = from.direction().xy();
        let from_matrix = Matrix2::from_columns(first, Xy::new(-first.y(), first.x()));
        let from_origin = from.location().xy().multiplied_matrix(transform.matrix);
        transform.offset += from_origin;
        transform.matrix.multiply(from_matrix);
        transform
    }
}
/// The axis direction and its quarter turn as rows: SetCol(1/2) followed by Transpose.
fn axis_rows(axis: Axis2d) -> Matrix2 {
    let first = axis.direction().xy();
    Matrix2::new([[first.x(), first.y()], [-first.y(), first.x()]])
}
impl Transform2 {
    /// Returns the retained form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Returns the factored scale; `transform.scale_factor()`.
    pub const fn scale_factor(&self) -> f64 {
        self.scale
    }
    /// Returns the native translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xy {
        self.offset
    }
    /// Returns the unscaled matrix (gp_Trsf2d::HVectorialPart); `transform.matrix()`.
    pub const fn matrix(&self) -> Matrix2 {
        self.matrix
    }
    /// Tests the sign of the unscaled matrix determinant, so a point mirror (identity
    /// matrix, scale -1) is not negative; `transform.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.matrix.determinant() < 0.0
    }
    /// Reads a scaled matrix coefficient or the translation (column 3) by one-based index;
    /// `transform.value(1,3)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        if !(1..=2).contains(&row) || !(1..=3).contains(&column) {
            return Err(Failure::out_of_range(&format!(
                "gp_Trsf2d::Value() - index ({row}, {column}) outside rows [1, 2] and columns [1, 3]"
            )));
        }
        Ok(self.coefficient(row as usize, column as usize))
    }
    /// Reads `value` for indices already known to be in range; `transform.coefficient(1,3)`.
    pub(in crate::gp) fn coefficient(&self, row: usize, column: usize) -> f64 {
        if column < 3 {
            return self.scale * self.matrix.coefficients[row - 1][column - 1];
        }
        self.offset.coordinates()[row - 1]
    }
    /// Applies the unscaled matrix; `transform.apply_unscaled(xy)`.
    pub fn apply_unscaled(self, xy: Xy) -> Xy {
        xy.multiplied_matrix(self.matrix)
    }
    /// Applies the vectorial part without translation; `transform.apply_vector(xy)`.
    pub fn apply_vector(self, xy: Xy) -> Xy {
        let mut result = self.apply_unscaled(xy);
        if self.scale != 1.0 {
            result *= self.scale;
        }
        result
    }
    /// gp_Trsf2d::Transforms(gp_XY&): matrix, scale when not 1, then translation, with no form
    /// shortcuts; `transform.apply_coordinates(xy)`.
    pub fn apply_coordinates(self, xy: Xy) -> Xy {
        self.apply_vector(xy) + self.offset
    }
    /// gp_Trsf2d::Transforms(double&,double&); `transform.transforms_coordinates(1.0,2.0)`.
    pub fn transforms_coordinates(self, x: f64, y: f64) -> (f64, f64) {
        let result = self.apply_coordinates(Xy::new(x, y));
        (result.x(), result.y())
    }
    /// Uses gp_Pnt2d's form shortcuts; `transform.apply(point)`.
    pub fn apply(self, point: Point2) -> Point2 {
        point.transformed(self)
    }
}
