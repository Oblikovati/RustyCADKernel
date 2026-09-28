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
// Rust translation of FoundationClasses/TKMath/gp/gp_Trsf.cxx SetTransformation(gp_Ax3) and SetTransformation(gp_Ax3, gp_Ax3) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::{Axis3, Failure, Matrix3, Transform3, TransformForm, Xyz};
impl Transform3 {
    /// Maps global coordinates into a frame's local system (gp_Trsf::SetTransformation(gp_Ax3));
    /// `Transform3::transformation_to(frame)`.
    pub fn transformation_to(frame: Axis3) -> Self {
        let mut matrix = Matrix3::default();
        matrix.set_rows(
            Xyz::from(frame.x_direction()),
            Xyz::from(frame.y_direction()),
            Xyz::from(frame.direction()),
        );
        let offset = matrix.apply(frame.location().xyz()).reversed();
        Self {
            form: TransformForm::Compound,
            matrix,
            offset,
            scale: 1.0,
        }
    }
    /// Maps coordinates expressed in `from` onto the same coordinates in `to`
    /// (gp_Trsf::SetTransformation(gp_Ax3, gp_Ax3)), keeping the native order: the frame
    /// change into `to` first, then the product with the `from` direction columns;
    /// `Transform3::transformation_between(sphere.position(), Axis3::default())`.
    pub fn transformation_between(from: Axis3, to: Axis3) -> Self {
        let into_to = Self::transformation_to(to);
        let from_columns = Matrix3::from_columns(
            Xyz::from(from.x_direction()),
            Xyz::from(from.y_direction()),
            Xyz::from(from.direction()),
        );
        let from_origin = into_to.matrix.apply(from.location().xyz());
        Self {
            form: TransformForm::Compound,
            matrix: into_to.matrix * from_columns,
            offset: into_to.offset + from_origin,
            scale: 1.0,
        }
    }
    /// Reads a scaled matrix coefficient or the translation (column 4) by one-based index;
    /// `transform.value(1,4)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        if !(1..=3).contains(&row) || !(1..=4).contains(&column) {
            return Err(Failure::out_of_range(&format!(
                "gp_Trsf::Value() - index ({row}, {column}) outside rows [1, 3] and columns [1, 4]"
            )));
        }
        Ok(self.coefficient(row as usize, column as usize))
    }
    /// Reads `value` for indices already known to be in range; `transform.coefficient(1,4)`.
    pub(in crate::gp) fn coefficient(&self, row: usize, column: usize) -> f64 {
        if column < 4 {
            return self.scale * self.matrix.coefficients()[row - 1][column - 1];
        }
        self.offset.coordinates()[row - 1]
    }
    /// Reads the 3x4 `value` table of a frame change in row order; `Transform3::frame_rows(frame)`.
    pub(in crate::gp) fn frame_rows(frame: Axis3) -> [[f64; 4]; 3] {
        let transform = Self::transformation_to(frame);
        std::array::from_fn(|row| {
            std::array::from_fn(|column| transform.coefficient(row + 1, column + 1))
        })
    }
}
