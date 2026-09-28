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

use super::{Axis2d, Failure, Point2, TransformForm, Xy};
/// Planar identity, translation, rotation, and axial mirror transforms; `Transform2::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform2 {
    form: TransformForm,
    matrix: [[f64; 2]; 2],
    offset: Xy,
    scale: f64,
}
impl Default for Transform2 {
    fn default() -> Self {
        Self {
            form: TransformForm::Identity,
            matrix: [[1.0, 0.0], [0.0, 1.0]],
            offset: Xy::default(),
            scale: 1.0,
        }
    }
}
impl Transform2 {
    /// Returns the unscaled matrix by value; `transform.matrix()`.
    pub const fn matrix(&self) -> [[f64; 2]; 2] {
        self.matrix
    }

    /// Returns the native translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xy {
        self.offset
    }

    /// Constructs even a zero displacement with Translation form; `Transform2::translation([1.0,2.0])`.
    pub fn translation(vector: [f64; 2]) -> Self {
        Self {
            form: TransformForm::Translation,
            offset: Xy::new(vector[0], vector[1]),
            ..Self::default()
        }
    }
    /// Uses the native rotation matrix and negated-origin multiplication; `Transform2::rotation(center,angle)`.
    pub fn rotation(center: Point2, angle: f64) -> Self {
        let cosine = angle.cos();
        let sine = angle.sin();
        let matrix = [[cosine, -sine], [sine, cosine]];
        let offset = center.xy().reversed().multiplied_matrix(matrix) + center.xy();
        Self {
            form: TransformForm::Rotation,
            matrix,
            offset,
            scale: 1.0,
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
        let matrix = [
            [1.0 - 2.0 * vx * vx, -2.0 * vx * vy],
            [-2.0 * vx * vy, 1.0 - 2.0 * vy * vy],
        ];
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
        let first = axis.direction().xy();
        let second = Xy::new(-first.y(), first.x());
        // SetCol(1/2) then Transpose leaves the directions as rows.
        let matrix = [[first.x(), first.y()], [second.x(), second.y()]];
        let offset = axis.location().xy().multiplied_matrix(matrix).reversed();
        Self {
            form: TransformForm::Compound,
            matrix,
            offset,
            scale: 1.0,
        }
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
            return self.scale * self.matrix[row - 1][column - 1];
        }
        self.offset.coordinates()[row - 1]
    }
    /// Returns the retained form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Returns the factored scale; `transform.scale_factor()`.
    pub const fn scale_factor(&self) -> f64 {
        self.scale
    }
    /// Applies the unscaled matrix; `transform.apply_unscaled(xy)`.
    pub fn apply_unscaled(self, xy: Xy) -> Xy {
        xy.multiplied_matrix(self.matrix)
    }
    /// Applies the vectorial part without translation; `transform.apply_vector(xy)`.
    pub fn apply_vector(self, xy: Xy) -> Xy {
        let mut result = self.apply_unscaled(xy);
        if self.scale != 1.0 {
            result = result * self.scale;
        }
        result
    }
    /// Applies matrix, scale and translation without form shortcuts; `transform.apply_coordinates(xy)`.
    pub fn apply_coordinates(self, xy: Xy) -> Xy {
        self.apply_vector(xy) + self.offset
    }
    /// Uses gp_Pnt2d's form shortcuts; `transform.apply(point)`.
    pub fn apply(self, point: Point2) -> Point2 {
        match self.form {
            TransformForm::Identity => point,
            TransformForm::Translation => Point2::from(point.xy() + self.offset),
            _ => Point2::from(self.apply_coordinates(point.xy())),
        }
    }
    /// Composes using native rotation and mirror branches (OCC22736); `left.multiplied(right)`.
    pub fn multiplied(mut self, other: Self) -> Self {
        use TransformForm::*;
        match (self.form, other.form) {
            (_, Identity) => return self,
            (Identity, _) => return other,
            (Translation, Translation) => self.offset = self.offset + other.offset,
            (Translation, _) => {
                self.form = Compound;
                self.offset = self.offset + other.offset;
                self.scale = other.scale;
                self.matrix = other.matrix;
            }
            (Rotation, Rotation) => self.compose_rotations(other),
            (_, Translation) => self.offset = self.offset + self.apply_vector(other.offset),
            (AxisMirror, AxisMirror) => {
                self.compose_general(other);
                self.form = Rotation;
            }
            _ => self.compose_general(other),
        }
        self
    }
    fn compose_rotations(&mut self, other: Self) {
        // gp_Trsf2d checks this offset, unlike gp_Trsf's check of the right-hand offset.
        if self.offset != Xy::default() {
            self.offset = self.offset + other.offset.multiplied_matrix(self.matrix);
        }
        self.matrix = multiply_matrix(self.matrix, other.matrix);
    }
    fn compose_general(&mut self, other: Self) {
        self.offset = self.offset + self.apply_vector(other.offset);
        self.scale *= other.scale;
        self.matrix = multiply_matrix(self.matrix, other.matrix);
        self.form = TransformForm::Compound;
    }
}
fn multiply_matrix(left: [[f64; 2]; 2], right: [[f64; 2]; 2]) -> [[f64; 2]; 2] {
    std::array::from_fn(|row| {
        std::array::from_fn(|col| left[row][0] * right[0][col] + left[row][1] * right[1][col])
    })
}
