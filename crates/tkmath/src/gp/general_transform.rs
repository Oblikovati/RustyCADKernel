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
// Rust translation of FoundationClasses/TKMath/gp/gp_GTrsf.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! gp_GTrsf: a 3D affine transformation. Non-orthogonal matrices carry the `Other` form and
//! a zero scale; every other form is an embedded gp_Trsf.
use super::{Axis1, Axis2, Failure, Matrix3, Transform3, TransformForm, Xyz, construction};
use rustycad_tkernel::standard::dump::{self, DumpOutput, DumpStream};
use std::io;
mod algebra;
/// An affine transformation allowing independent directional scaling; `GeneralTransform3::default()`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeneralTransform3 {
    matrix: Matrix3,
    offset: Xyz,
    form: TransformForm,
    scale: f64,
}
impl Default for GeneralTransform3 {
    fn default() -> Self {
        Self::from(Transform3::default())
    }
}
/// gp_GTrsf(const gp_Trsf&) and SetTrsf copy all four fields.
impl From<Transform3> for GeneralTransform3 {
    fn from(transform: Transform3) -> Self {
        Self {
            matrix: transform.matrix,
            offset: transform.offset,
            form: transform.form,
            scale: transform.scale,
        }
    }
}
impl GeneralTransform3 {
    /// Assigns the general vectorial and translation parts with form Other and scale 0;
    /// `GeneralTransform3::new(matrix,offset)`.
    pub const fn new(matrix: Matrix3, offset: Xyz) -> Self {
        Self {
            matrix,
            offset,
            form: TransformForm::Other,
            scale: 0.0,
        }
    }
    /// Affinity perpendicular to an oriented line: `(1-r)*D*D^T + r*I`, fixing the line;
    /// `transform.set_affinity_axis(axis,2.0)`.
    pub fn set_affinity_axis(&mut self, axis: Axis1, ratio: f64) {
        self.set_general();
        self.matrix.set_dot(Xyz::from(axis.direction()));
        self.matrix *= 1.0 - ratio;
        self.add_diagonal(ratio);
        let origin = Xyz::from(axis.location());
        self.offset = self.matrix.apply(origin.reversed()) + origin;
    }
    /// Affinity along a plane normal (OCC13963): the offset is taken from `(r-1)*N*N^T` before
    /// the identity is added, so the location is not added back;
    /// `transform.set_affinity_plane(frame,2.0)`.
    pub fn set_affinity_plane(&mut self, frame: Axis2, ratio: f64) {
        self.set_general();
        self.matrix.set_dot(Xyz::from(frame.direction()));
        self.matrix *= ratio - 1.0;
        self.offset = self.matrix.apply(Xyz::from(frame.location()).reversed());
        self.add_diagonal(1.0);
    }
    fn set_general(&mut self) {
        self.form = TransformForm::Other;
        self.scale = 0.0;
    }
    fn add_diagonal(&mut self, value: f64) {
        let diagonal = self.matrix.diagonal();
        self.matrix.set_diagonal(
            diagonal.x() + value,
            diagonal.y() + value,
            diagonal.z() + value,
        );
    }
    /// Sets one coefficient. Column 4 is the translation and only promotes Identity to
    /// Translation; a matrix cell first folds a non-unit similarity scale into the matrix and
    /// then forces form Other; `transform.set_value(1,4,2.0)?`.
    pub fn set_value(&mut self, row: i32, column: i32, value: f64) -> Result<(), Failure> {
        check_cell(row, column)?;
        if column == 4 {
            self.offset.set_coord(row, value)?;
            if self.form == TransformForm::Identity {
                self.form = TransformForm::Translation;
            }
            return Ok(());
        }
        if self.form != TransformForm::Other && self.scale != 1.0 {
            self.matrix *= self.scale;
        }
        self.matrix.set_value(row, column, value)?;
        self.set_general();
        Ok(())
    }
    /// Replaces the matrix, forcing form Other and scale 0; `transform.set_vectorial_part(m)`.
    pub fn set_vectorial_part(&mut self, matrix: Matrix3) {
        self.matrix = matrix;
        self.set_general();
    }
    /// Replaces the translation and re-derives the form; `transform.set_translation_part(xyz)`.
    pub fn set_translation_part(&mut self, offset: Xyz) {
        self.offset = offset;
        self.form = match self.form {
            TransformForm::Compound | TransformForm::Other | TransformForm::Translation => {
                self.form
            }
            TransformForm::Identity => TransformForm::Translation,
            _ => TransformForm::Compound,
        };
    }
    /// Copies a similarity (gp_GTrsf::SetTrsf); `transform.set_trsf(similarity)`.
    pub fn set_trsf(&mut self, transform: Transform3) {
        *self = Self::from(transform);
    }
}
fn check_cell(row: i32, column: i32) -> Result<(), Failure> {
    if !(1..=3).contains(&row) || !(1..=4).contains(&column) {
        return Err(Failure::out_of_range(&format!(
            "gp_GTrsf - cell ({row}, {column}) outside rows [1, 3] and columns [1, 4]"
        )));
    }
    Ok(())
}
impl GeneralTransform3 {
    /// Tests the sign of the matrix determinant; `transform.is_negative()`.
    pub fn is_negative(&self) -> bool {
        self.matrix.determinant() < 0.0
    }
    /// Tests the matrix determinant against gp::Resolution; `transform.is_singular()`.
    pub fn is_singular(&self) -> bool {
        self.matrix.is_singular()
    }
    /// Returns the form; `transform.form()`.
    pub const fn form(&self) -> TransformForm {
        self.form
    }
    /// Returns the translation part; `transform.translation_part()`.
    pub const fn translation_part(&self) -> Xyz {
        self.offset
    }
    /// Returns the stored matrix, unscaled for similarity forms; `transform.vectorial_part()`.
    pub const fn vectorial_part(&self) -> Matrix3 {
        self.matrix
    }
    /// Reads a coefficient (also operator()); similarity forms are scaled, Other is not;
    /// `transform.value(1,1)?`.
    pub fn value(&self, row: i32, column: i32) -> Result<f64, Failure> {
        check_cell(row, column)?;
        Ok(self.coefficient(row as usize, column as usize))
    }
    /// Reads `value` for one-based indices already known to be in range.
    fn coefficient(&self, row: usize, column: usize) -> f64 {
        if column == 4 {
            return self.offset.coordinates()[row - 1];
        }
        let coefficient = self.matrix.coefficients()[row - 1][column - 1];
        if self.form == TransformForm::Other {
            return coefficient;
        }
        self.scale * coefficient
    }
    /// Applies the matrix, the scale of a non-unit similarity, then the translation
    /// (gp_GTrsf::Transforms(gp_XYZ&)); `transform.apply(xyz)`.
    pub fn apply(&self, coordinates: Xyz) -> Xyz {
        let mut result = self.matrix.apply(coordinates);
        if self.form != TransformForm::Other && self.scale != 1.0 {
            result *= self.scale;
        }
        result + self.offset
    }
    /// Updates a coordinate triple in place; `transform.transforms(&mut xyz)`.
    pub fn transforms(&self, coordinates: &mut Xyz) {
        *coordinates = self.apply(*coordinates);
    }
    /// gp_GTrsf::Transforms(double&,double&,double&); `transform.transforms_coordinates(1.0,2.0,3.0)`.
    pub fn transforms_coordinates(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let result = self.apply(Xyz::new(x, y, z));
        (result.x(), result.y(), result.z())
    }
    /// Converts back to a similarity; only the Other form raises (no orthogonality check,
    /// unlike gp_GTrsf2d::Trsf2d); `transform.trsf()?`.
    pub fn trsf(&self) -> Result<Transform3, Failure> {
        if self.form == TransformForm::Other {
            return Err(construction(
                "gp_GTrsf::Trsf() - non-orthogonal GTrsf (form Other)",
            ));
        }
        Ok(self.as_trsf())
    }
    fn as_trsf(&self) -> Transform3 {
        Transform3 {
            form: self.form,
            matrix: self.matrix,
            offset: self.offset,
            scale: self.scale,
        }
    }
    /// Classifies `M / cbrt(det M)`: Other when `N^T N - I` has an entry above 1e-12,
    /// otherwise Other becomes Compound and similarity forms stay; the scale is not changed;
    /// `transform.set_form()?`.
    pub fn set_form(&mut self) -> Result<(), Failure> {
        let determinant = self.matrix.determinant();
        if determinant.abs() < f64::MIN_POSITIVE {
            return Err(construction(&format!(
                "gp_GTrsf::SetForm, null determinant {determinant}"
            )));
        }
        let root = if determinant > 0.0 {
            determinant.powf(1.0 / 3.0)
        } else {
            -(-determinant).powf(1.0 / 3.0)
        };
        let normalized = self.matrix.divided(root)?;
        let mut identity = Matrix3::default();
        identity.set_identity();
        let defect = normalized.transposed() * normalized - identity;
        if self.form == TransformForm::Other {
            self.form = TransformForm::Compound;
        }
        let tolerance = 1.0e-12; // tol:angular - the literal Precision::Angular() of gp_GTrsf::SetForm
        if defect
            .coefficients()
            .iter()
            .flatten()
            .any(|v| v.abs() > tolerance)
        {
            self.form = TransformForm::Other;
        }
        Ok(())
    }
    /// Writes the class name, the nested `matrix` and `loc` fragments (skipped at depth 0),
    /// `shape` and `scale`; `transform.dump_json(&mut stream,-1)?`.
    pub fn dump_json(&self, output: &mut impl DumpOutput, depth: i32) -> io::Result<()> {
        dump::dump_class_begin(output, "gp_GTrsf")?;
        if depth != 0 {
            let mut matrix = DumpStream::default();
            self.matrix.dump_json(&mut matrix)?;
            dump::dump_key_to_class(output, "matrix", matrix.text())?;
            let mut location = DumpStream::default();
            self.offset.dump_json(&mut location)?;
            dump::dump_key_to_class(output, "loc", location.text())?;
        }
        dump::dump_integer(output, "shape", self.form as i32)?;
        dump::dump_real(output, "scale", self.scale)
    }
}
